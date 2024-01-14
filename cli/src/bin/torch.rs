use chrono::Utc;
use cli::*;
use handy_core::game::*;
use handy_core::utils::*;
use priq::PriorityQueue;
use rand::thread_rng;
use rand::Rng;
use serde_jsonlines::json_lines;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::time::Instant;

use tch::{
    data::Iter2,
    nn,
    nn::{
        init::DEFAULT_KAIMING_UNIFORM, ConvConfig, Init::Const, Module, OptimizerConfig,
        PaddingMode,
    },
    Device, Kind, Tensor,
};

const PILE_SIZE: i64 = 9;
const CARD_SIZE: i64 = 13;
const INPUT_SIZE: i64 = PILE_SIZE * CARD_SIZE;

const HERO: Class = Class::Cursed;
const MONSTER: Class = Class::Spider;
const PATH: &str = "cursed-spider.safetensors";

fn basic_heuristic(pile: &Pile) -> f32 {
    if Some(Allegiance::Hero) == is_game_winner(pile) {
        return 0.0;
    }

    let mut score: f32 = 0.0;
    let mut consecutive_monster_mod = 1.0;

    for card_ptr in pile.iter() {
        let active_face = card_ptr.get_active_face();
        // Vampire master special case
        if card_ptr.card_id == 18 {
            let weight_per_side = 6.0;
            score += match card_ptr.key {
                FaceKey::A => weight_per_side * 3.0,
                FaceKey::B => weight_per_side * 2.0,
                FaceKey::D => weight_per_side * 1.0,
                FaceKey::C => 0.0,
            };
        } else {
            score += match active_face.allegiance {
                Allegiance::Baddie => {
                    let health_score = match active_face.health {
                        Health::Empty => 0.0,
                        Health::Half => 1.0,
                        Health::Full => 2.0,
                    };

                    health_score + consecutive_monster_mod
                }
                Allegiance::Hero => {
                    consecutive_monster_mod = 0.0;

                    match active_face.health {
                        Health::Empty => -0.1,
                        Health::Half => -0.06,
                        Health::Full => 0.0,
                    }
                }
                Allegiance::Werewolf => {
                    consecutive_monster_mod = 0.0;

                    0.0
                }
            }
        }
    }

    return score;
}

fn net(vs: &nn::Path) -> impl Module {
    let conv_output = 16;
    let l1_output = 64;
    let l2_output = 64;
    let l3_output = 8;

    nn::seq()
        .add(nn::conv1d(
            vs / "conv1",
            CARD_SIZE,
            conv_output,
            1,
            ConvConfig {
                stride: 1,
                padding: 0,
                dilation: 1,
                groups: 1,
                bias: true,
                ws_init: DEFAULT_KAIMING_UNIFORM,
                bs_init: Const(0.),
                padding_mode: PaddingMode::Zeros,
            },
        ))
        .add_fn(|xs| xs.flatten(1, 2))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(
            vs / "layer1",
            conv_output * PILE_SIZE as i64,
            l1_output,
            Default::default(),
        ))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(
            vs / "layer2",
            l1_output,
            l2_output,
            Default::default(),
        ))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(
            vs / "layer3",
            l2_output,
            l3_output,
            Default::default(),
        ))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(vs / "layer4", l3_output, 1, Default::default()))
}

fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4) }
}

fn empty_onehot() -> Tensor {
    let vec_source: Vec<f32> = vec![0.0; INPUT_SIZE as usize];
    let as_bytes = to_byte_slice(&vec_source);
    return Tensor::from_data_size(&as_bytes, &[CARD_SIZE, PILE_SIZE], Kind::Float);
}

fn face_to_idx(face_key: FaceKey) -> usize {
    match face_key {
        FaceKey::A => 0,
        FaceKey::B => 1,
        FaceKey::C => 2,
        FaceKey::D => 3,
    }
}

fn pile_onehot(card_map: &CardMap, pile: &Pile) -> Tensor {
    let mut vec_source: Vec<f32> = vec![0.0; INPUT_SIZE as usize];
    for i in 0..9 {
        let card_idx = card_map[&pile[i].card_id];
        let face_idx = PILE_SIZE as usize + face_to_idx(pile[i].get_card_face());
        let card_1d = card_idx * PILE_SIZE as usize + i;
        let face_1d = face_idx * PILE_SIZE as usize + i;
        // dbg!(card_idx, face_idx, i, card_1d, face_1d);
        vec_source[card_1d] = 1.0;
        vec_source[face_1d] = 1.0;
    }

    let as_bytes = to_byte_slice(&vec_source);
    return Tensor::from_data_size(&as_bytes, &[CARD_SIZE, PILE_SIZE], Kind::Float);
}

type CardMap = HashMap<CardId, usize>;

fn make_card_map(hero: Class, monster: Class) -> CardMap {
    let mut cards = CARDS.get_cards_for_class(hero);
    cards.append(&mut CARDS.get_cards_for_class(monster));

    let mut card_ids: Vec<CardId> = cards.iter().map(|c| c.id).collect();
    card_ids.sort();

    let mut card_map = CardMap::new();
    for (idx, card_id) in card_ids.iter().enumerate() {
        card_map.insert(*card_id, idx);
    }
    card_map
}

fn generate_fully_random_basic_heuristic_example<R: Rng>(
    card_map: &CardMap,
    hero: Class,
    monster: Class,
    rng: &mut R,
) -> (Tensor, Tensor) {
    let pile = get_fully_random_pile(hero, monster, rng);
    let x = pile_onehot(card_map, &pile);
    let pile_score = basic_heuristic(&pile);
    let y = Tensor::from_slice(&[pile_score]);

    (x, y)
}

fn generate_won_random_basic_heuristic_example<R: Rng>(
    card_map: &CardMap,
    hero: Class,
    monster: Class,
    rng: &mut R,
) -> (Tensor, Tensor) {
    let pile = get_random_won_pile(hero, monster, rng);
    let x = pile_onehot(card_map, &pile);
    let pile_score = basic_heuristic(&pile);
    let y = Tensor::from_slice(&[pile_score]);

    (x, y)
}

fn generate_basic_example_batch<R: Rng>(
    non_batch_size: usize,
    win_batch_size: usize,
    card_map: &CardMap,
    hero: Class,
    monster: Class,
    rng: &mut R,
) -> (Tensor, Tensor) {
    let tot_batch_size = non_batch_size + win_batch_size;
    let mut v_xs = Vec::with_capacity(tot_batch_size);
    let mut v_ys = Vec::with_capacity(tot_batch_size);

    for _ in 0..non_batch_size {
        let (x, y) = generate_fully_random_basic_heuristic_example(card_map, hero, monster, rng);
        v_xs.push(x);
        v_ys.push(y);
    }

    for _ in 0..win_batch_size {
        let (x, y) = generate_won_random_basic_heuristic_example(card_map, hero, monster, rng);
        v_xs.push(x);
        v_ys.push(y);
    }

    (Tensor::stack(&v_xs, 0), Tensor::stack(&v_ys, 0))
}

fn generate_batch_from_examples(card_map: &CardMap, examples: Vec<Example>) -> (Tensor, Tensor) {
    let mut v_xs = Vec::with_capacity(examples.len());
    let mut v_ys = Vec::with_capacity(examples.len());

    for example in examples {
        v_xs.push(pile_onehot(card_map, &example.0));
        v_ys.push(Tensor::from_slice(&[example.1 as f64]));
    }

    (Tensor::stack(&v_xs, 0), Tensor::stack(&v_ys, 0))
}

fn init_model() {
    let vs = nn::VarStore::new(Device::cuda_if_available());
    net(&vs.root());
    vs.save(PATH).unwrap();
}

fn pre_training() {
    let mut rng = thread_rng();
    let card_map: CardMap = make_card_map(HERO, MONSTER);
    let test_batch_non_size = 40;
    let test_batch_win_size = 10;
    let test_batch_total_size = test_batch_non_size + test_batch_win_size;
    let runs_per_test = 5000;
    let train_batch_non_size = 10;
    let train_batch_win_size = 2;

    let mut vs = nn::VarStore::new(Device::cuda_if_available());
    let net = net(&vs.root());
    vs.load(PATH).unwrap();
    let mut opt = nn::AdamW::default().build(&vs, 1e-4).unwrap();
    dbg!(&net);

    loop {
        {
            vs.save(PATH).unwrap();
            let (xs, ys) = generate_basic_example_batch(
                test_batch_non_size,
                test_batch_win_size,
                &card_map,
                HERO,
                MONSTER,
                &mut rng,
            );
            let res = net.forward(&xs);
            let diffs = &res - &ys;
            let loss = diffs
                .pow_tensor_scalar(2)
                .sum(Kind::Float)
                .double_value(&[]);

            println!("");
            for i in 0..test_batch_total_size {
                let exp = ys.double_value(&[i as i64, 0]);
                let actual = res.double_value(&[i as i64, 0]);
                println!("{:.2} {:.2} / {:.2}", exp, actual, exp - actual);
            }
            println!("avg loss: {}", loss / test_batch_total_size as f64);
        }

        for _ in 0..runs_per_test {
            let (xs, ys) = generate_basic_example_batch(
                train_batch_non_size,
                train_batch_win_size,
                &card_map,
                HERO,
                MONSTER,
                &mut rng,
            );
            let loss = (net.forward(&xs) - ys)
                .pow_tensor_scalar(2)
                .sum(Kind::Float);
            opt.backward_step(&loss);
        }
    }
}

pub type SeenMap = BTreeMap<Pile, SearchNode>;
pub type DepthType = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnownScore {
    Unknown,
    Lost,
    Win,
}

#[derive(Debug)]
pub struct SearchNode {
    lowest_seen_depth: DepthType,
    // children: Option<Vec<Pile>>,
    parents: Vec<Pile>,
    estimated_score: f32,
    known_score: KnownScore,
}

pub struct NNModel {
    // hero: Class,
    // monster: Class,
    card_map: CardMap,
    net: Box<dyn Module>,
}

impl NNModel {
    fn new(hero: Class, monster: Class, net: Box<dyn Module>) -> Self {
        let card_map = make_card_map(hero, monster);
        Self {
            // hero,
            // monster,
            card_map,
            net,
        }
    }

    fn score_pile(&self, pile: &Pile) -> f32 {
        let x = pile_onehot(&self.card_map, &pile);
        let res = self.net.forward(&x.unsqueeze(0));
        return res.double_value(&[0]) as f32;
    }

    fn score_piles(&self, piles: &[Pile]) -> Tensor {
        let vec_x: Vec<Tensor> = piles
            .iter()
            .map(|p| pile_onehot(&self.card_map, p))
            .collect();
        let xs = Tensor::stack(&vec_x, 0);

        let res = self.net.forward(&xs);
        return res;
    }
}

struct SearchState {
    seen_states: BTreeMap<Pile, SearchNode>,
    queue: PriorityQueue<f32, Pile>,
}

impl SearchState {
    fn new() -> Self {
        Self {
            seen_states: BTreeMap::new(),
            queue: PriorityQueue::new(),
        }
    }
}

type Example = (Pile, DepthType);
struct ExamplePool {
    examples: Vec<Example>,
    largest_batch_size: f32,
    batch_size: usize,
    max_proportion: f32,
}

impl ExamplePool {
    fn new(batch_size: usize, max_proportion: f32) -> Self {
        Self {
            examples: Vec::new(),
            largest_batch_size: 0.0,
            batch_size,
            max_proportion,
        }
    }

    fn add_examples(&mut self, mut new_examples: Vec<Example>) {
        let new_examples_len = new_examples.len() as f32;
        if new_examples_len > self.largest_batch_size {
            self.largest_batch_size = new_examples_len;
        }

        self.examples.append(&mut new_examples);
    }

    fn can_train(&self) -> bool {
        if self.examples.len() < self.batch_size {
            return false;
        }

        let current_proportion = self.largest_batch_size / self.examples.len() as f32;

        current_proportion < self.max_proportion
    }

    fn fetch_examples(&mut self) -> Vec<Example> {
        let mut result = Vec::with_capacity(self.batch_size);
        let mut rng = thread_rng();

        self.largest_batch_size = self.largest_batch_size
            * ((self.examples.len() - self.batch_size) as f32 / self.examples.len() as f32);

        for _ in 0..self.batch_size {
            let ex_len = self.examples.len();
            let i = rng.gen_range(0..ex_len);
            if i != ex_len - 1 {
                self.examples.swap(i, ex_len - 1);
            }
            let ex = self.examples.pop().unwrap();
            result.push(ex);
        }

        result
    }
}

fn a_star_one_run(
    model: &NNModel,
    start_pile: Pile,
    max_num_iters: usize,
) -> Vec<(Pile, DepthType)> {
    let start_time = Utc::now().time();
    let mut search_state = SearchState::new();
    let mut max_expanded_depth: usize = 0;

    let mut wins: PriorityQueue<DepthType, Pile> = PriorityQueue::new();

    {
        // Perform search
        // println!("start pile: {:?}", start_pile);
        search_state.seen_states.insert(
            start_pile.clone(),
            SearchNode {
                lowest_seen_depth: 0,
                estimated_score: 0.0,
                known_score: KnownScore::Unknown,
                parents: vec![],
            },
        );
        search_state.queue.put(0.0, start_pile.clone());

        let mut iters = 0;
        loop {
            iters += 1;
            if iters > max_num_iters {
                break;
            }

            let Some((f_score, pile)) = search_state.queue.pop() else {
                break;
            };

            if is_game_winner(&pile).is_some() {
                panic!("popped end game pile: {:?}", &pile);
            }

            let current_seen_state = search_state.seen_states.get_mut(&pile).unwrap();
            max_expanded_depth = std::cmp::max(
                max_expanded_depth,
                current_seen_state.lowest_seen_depth as usize,
            );

            let child_depth = current_seen_state.lowest_seen_depth + 1;

            let cur_state = GameStateNoEventLog::new(pile.clone());
            let child_states = resolve_top_card(&cur_state);
            let child_piles: Vec<_> = child_states.into_iter().map(|s| s.pile).collect();
            let child_scores = model.score_piles(&child_piles);

            for (i, child_pile) in child_piles.into_iter().enumerate() {
                if let Some(child_seen_state) = search_state.seen_states.get_mut(&child_pile) {
                    if !child_seen_state.parents.contains(&pile) {
                        child_seen_state.parents.push(pile.clone());
                    }

                    if child_depth < child_seen_state.lowest_seen_depth {
                        child_seen_state.lowest_seen_depth = child_depth;

                        if child_seen_state.known_score == KnownScore::Unknown {
                            search_state.queue.put(
                                child_depth as f32 + child_seen_state.estimated_score,
                                child_pile.clone(),
                            );
                        } else if KnownScore::Win == child_seen_state.known_score {
                            wins.put(child_depth, child_pile.clone());
                        }
                    }
                } else {
                    let maybe_winner = is_game_winner(&child_pile);

                    if let Some(winner) = maybe_winner {
                        if winner == Allegiance::Hero {
                            // println!("found winner at depth: {} / {:?}", child_depth, &child_pile);
                            search_state.seen_states.insert(
                                child_pile.clone(),
                                SearchNode {
                                    parents: vec![pile.clone()],
                                    lowest_seen_depth: child_depth,
                                    estimated_score: 0.0,
                                    known_score: KnownScore::Win,
                                },
                            );
                            wins.put(child_depth, child_pile.clone());
                        } else {
                            search_state.seen_states.insert(
                                child_pile.clone(),
                                SearchNode {
                                    parents: vec![pile.clone()],
                                    lowest_seen_depth: child_depth,
                                    estimated_score: 0.0,
                                    known_score: KnownScore::Lost,
                                },
                            );
                        }
                    } else {
                        let child_score = child_scores.double_value(&[i as i64]) as f32;
                        search_state.seen_states.insert(
                            child_pile.clone(),
                            SearchNode {
                                parents: vec![pile.clone()],
                                lowest_seen_depth: child_depth,
                                estimated_score: child_score,
                                known_score: KnownScore::Unknown,
                            },
                        );
                        search_state
                            .queue
                            .put(child_depth as f32 + child_score, child_pile.clone());
                    }
                }
            }
        }
    }

    let mut final_examples: HashMap<Pile, DepthType> = HashMap::new();

    // Collect results
    {
        // println!("Num wins: {:?}", wins.len());
        loop {
            let Some((_, pile)) = wins.pop() else {
                break;
            };

            collect_examples(&mut final_examples, &search_state, &pile, 0);
        }
    }

    // for (k, v) in final_examples.iter() {
    //     println!("{:?} {:?}", k, v);
    // }

    let result: Vec<_> = final_examples.into_iter().collect();

    let end_time = Utc::now().time();
    let diff = end_time - start_time;
    println!("Done in {}s", diff.num_seconds());

    return result;
}

fn collect_examples(
    final_examples: &mut HashMap<Pile, DepthType>,
    search_state: &SearchState,
    pile: &Pile,
    pile_known_score: DepthType,
) {
    if let Some(d) = final_examples.get(pile) {
        if *d <= pile_known_score {
            return;
        }
    }

    final_examples.insert(pile.clone(), pile_known_score);

    let this_node = search_state.seen_states.get(pile).unwrap();
    for parent in &this_node.parents {
        collect_examples(final_examples, search_state, parent, pile_known_score + 1)
    }
}

type KnownExamplesType = (Tensor, Tensor, usize);

fn compare_to_dataset(model: &NNModel, known_examples: &KnownExamplesType) {
    let (xs, ys, count) = known_examples;
    let res = model.net.forward(&xs);
    let diffs = &res - ys;
    let loss = diffs
        .pow_tensor_scalar(2)
        .sum(Kind::Float)
        .double_value(&[]);

    println!("Avg loss: {}", loss / *count as f64);
}

fn a_star_train() {
    let mut rng = thread_rng();
    let mut vs = nn::VarStore::new(Device::cuda_if_available());
    let net = net(&vs.root());
    vs.load(PATH).unwrap();

    let mut opt = nn::AdamW::default().build(&vs, 1e-4).unwrap();
    let model = NNModel::new(HERO, MONSTER, Box::new(net));

    let known_examples = load_known_examples(HERO, MONSTER);

    // let a_star_iters: usize = 100000;
    let a_star_iters: usize = 250000;
    let training_batch_size = 10;
    let epochs = 10;
    let mut example_pool = ExamplePool::new(training_batch_size, 0.25);

    compare_to_dataset(&model, &known_examples);

    loop {
        {
            let start_pile = get_start_from_classes(HERO, MONSTER, &mut rng);
            // let start_pile = string_to_pile("[27A, 31A, 28A, 25A, 26A, 30A, 29A, 32A, 24A]");
            let examples = a_star_one_run(&model, start_pile, a_star_iters);
            println!("Adding {} new examples", examples.len());
            example_pool.add_examples(examples);
        }

        {
            if example_pool.can_train() {
                println!("Training.");
                while example_pool.can_train() {
                    let training_examples = example_pool.fetch_examples();
                    let (xs, ys) = generate_batch_from_examples(&model.card_map, training_examples);

                    for _ in 0..epochs {
                        let loss = (model.net.forward(&xs) - &ys)
                            .pow_tensor_scalar(2)
                            .sum(Kind::Float);
                        opt.backward_step(&loss);
                    }
                }

                compare_to_dataset(&model, &known_examples);
            }
        }

        println!(
            "Examples: {:.2} / {} = {:.2}",
            example_pool.largest_batch_size,
            example_pool.examples.len(),
            example_pool.largest_batch_size / (example_pool.examples.len() as f32 + 0.0001),
        );
    }
}

fn load_examples_from_path(
    hero: Class,
    monster: Class,
    examples_path: &str,
) -> (Tensor, Tensor, usize) {
    let card_map = make_card_map(hero, monster);
    let all_examples = json_lines(examples_path)
        .unwrap()
        .collect::<Result<Vec<DepthModeTrainingExample>, _>>()
        .unwrap();

    let mut vec_xs = Vec::new();
    let mut vec_ys = Vec::new();

    for example in all_examples {
        if let StateEval::Win(win) = example.eval {
            vec_xs.push(pile_onehot(&card_map, &example.pile));
            vec_ys.push(Tensor::from_slice(&[win as f32]));
        }
    }

    let count = vec_xs.len();

    return (Tensor::stack(&vec_xs, 0), Tensor::stack(&vec_ys, 0), count);
}

fn load_known_examples(hero: Class, monster: Class) -> (Tensor, Tensor, usize) {
    let examples_path = training_path_for_matchup((hero, monster));
    return load_examples_from_path(hero, monster, &examples_path);
}

fn load_validation_set(hero: Class, monster: Class) -> (Tensor, Tensor, usize) {
    let examples_path = format!("{}.old", training_path_for_matchup((hero, monster)));
    return load_examples_from_path(hero, monster, &examples_path);
}

fn known_examples_train() {
    let hero = HERO;
    let monster = MONSTER;
    let batch_size = 16;

    println!("started");
    let mut vs = nn::VarStore::new(Device::cuda_if_available());
    let net = net(&vs.root());
    vs.load(PATH).unwrap();
    let mut opt = nn::AdamW::default().build(&vs, 1e-4).unwrap();
    let model = NNModel::new(hero, monster, Box::new(net));
    println!("loaded model");

    let (train_xs, train_ys, train_size) = load_known_examples(hero, monster);
    // let (train_xs, train_ys, train_size) = load_validation_set(hero, monster);
    println!("loaded training set: {}", train_size);

    let validation_dataset = load_validation_set(hero, monster);
    println!("loaded validation set: {}", validation_dataset.2);

    loop {
        let before = Instant::now();
        let mut iter = Iter2::new(&train_xs, &train_ys, 16);
        iter.shuffle();

        for (i, (xs, ys)) in iter.enumerate() {
            if i % 1024 == 0 {
                print!(
                    "\r{i}: {:.2}%      ",
                    (i * batch_size * 100) as f32 / (train_size) as f32
                );
                io::stdout().flush().unwrap();
            }
            let loss = (model.net.forward(&xs) - &ys)
                .pow_tensor_scalar(2)
                .sum(Kind::Float);
            opt.backward_step(&loss);
        }
        println!("");

        vs.save(PATH).unwrap();
        compare_to_dataset(&model, &validation_dataset);
        println!("Elapsed time: {:.2?}", before.elapsed());
    }
}

fn main() {
    // init_model();
    // pre_training();

    // a_star_train();
    known_examples_train();
}
