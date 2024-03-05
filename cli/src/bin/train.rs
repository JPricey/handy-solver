use clap::Parser;
use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_jsonlines::json_lines;

const BATCH_SIZE: usize = 5000;

fn elem_wise_mult(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len());
    let mut result = Vec::with_capacity(a.len());
    for i in 0..a.len() {
        result.push(a[i] * b[i])
    }
    result
}

fn train<'a, I>(
    model: &Vec<f32>,
    examples: I,
    num_examples: usize,
    learning_rate: f32,
) -> (Vec<f32>, f32)
where
    I: Iterator<Item = &'a (Vec<f32>, f32, f32)>,
{
    let mut total_loss = 0.0;
    let mut new_model = model.clone();

    let rate = learning_rate / num_examples as f32;

    for (ex_pile, ex_score, ex_weight) in examples {
        let deriv = elem_wise_mult(model, ex_pile);
        let local_score: f32 = deriv.iter().sum();
        let local_diff = ex_score - local_score;
        let local_coeff = local_diff * rate * ex_weight;

        for i in 0..new_model.len() {
            new_model[i] += local_coeff * ex_pile[i];
        }

        let local_loss = local_diff * local_diff;
        total_loss -= local_loss / num_examples as f32;
    }

    (new_model, total_loss)
}

#[derive(Parser, Debug)]
pub struct TrainArgs {
    #[clap(short, long, num_args = 2)]
    pub classes: Vec<Class>,
    #[clap(long, action)]
    pub cont: bool,
    #[clap(short, long)]
    pub rate: Option<f32>,
    #[clap(long)]
    pub suffix: Option<String>,
}

const MAX_WEIGHT: f32 = 100.0;
fn get_weight_for_examples(examples: &Vec<DepthModeTrainingExample>) -> Vec<f32> {
    let mut counts_by_depth = Vec::new();

    for example in examples {
        let StateEval::Win(depth) = example.eval else {
            continue;
        };

        while depth >= counts_by_depth.len() {
            counts_by_depth.push(0);
        }

        counts_by_depth[depth] += 1;
    }

    let weight_per_level = (examples.len() as f32) / (counts_by_depth.len() as f32 - 1.0);

    let weights: Vec<f32> = counts_by_depth
        .iter()
        .map(|count| {
            if *count == 0 {
                return 0.0;
            }

            let weight = weight_per_level / (*count as f32);
            if weight > MAX_WEIGHT {
                MAX_WEIGHT
            } else {
                weight
            }
        })
        .collect();

    weights
}

fn main() {
    let args = TrainArgs::parse();
    let matchup = try_get_matchup_from_classes(&args.classes).expect("Could not parse matchup");
    let relevant_cards = get_relevant_cards_for_matchup(matchup);
    let mut struct_model: Model = if args.cont {
        println!("Using existing Model");
        try_read_model_for_matchup(matchup).expect("Couldn't load existing model")
    } else {
        println!("Using Zeros");
        Model::new_empty_for_cards(&relevant_cards)
    };
    struct_model.trim_to_cards(&relevant_cards);
    let mut model = struct_model.vectorize();
    let learning_rate = args.rate.unwrap_or(0.001);
    println!("Learning rate: {}", learning_rate);

    let suffix_str: String = args
        .suffix
        .map(|s| format!(".{s}"))
        .unwrap_or("".to_owned());

    let examples_path = training_path_for_matchup(matchup);
    println!("Reading examples from {examples_path}");
    let all_examples_base = json_lines(examples_path)
        .unwrap()
        .collect::<Result<Vec<DepthModeTrainingExample>, _>>()
        .unwrap();

    let weights_by_depth = get_weight_for_examples(&all_examples_base);
    for (i, x) in weights_by_depth.iter().enumerate() {
        println!("{:?}", (i, x));
    }

    let all_examples_vec: Vec<_> = all_examples_base
        .iter()
        .filter_map(|ex| {
            let StateEval::Win(depth) = ex.eval else {
                return None;
            };
            Some((
                training_ex_to_model(&ex.pile).vectorize(),
                depth as f32,
                1.0,
                // weights_by_depth[depth],
            ))
        })
        .collect();

    let mut i = 0;
    let mut score;
    let num_examples = all_examples_vec.len();
    let mut rng = thread_rng();
    let mut first = true;

    loop {
        if num_examples > BATCH_SIZE {
            (model, score) = train(
                &model,
                all_examples_vec.choose_multiple(&mut rng, BATCH_SIZE),
                BATCH_SIZE,
                learning_rate,
            );
        } else {
            (model, score) = train(&model, all_examples_vec.iter(), num_examples, learning_rate);
        }
        if first {
            first = false;
            println!("Init score: {matchup:?} {score}");
        }
        i += 1;
        if i > 100 {
            i = 0;
            println!("Writing out model with score: {matchup:?} {score}");
            let result_model = vec_to_model(&model, &relevant_cards);
            write_model_for_matchup_with_custom_suffix(&result_model, matchup, &suffix_str);
        }
    }
}
