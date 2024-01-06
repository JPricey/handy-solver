use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;

use tch::{
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

fn net(vs: &nn::Path) -> impl Module {
    let conv_output = 16;
    let l1_output = 64;
    let l2_output = 64;

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
        .add(nn::linear(vs / "layer3", l2_output, 1, Default::default()))
        .add_fn(|xs| xs.sigmoid())
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

fn _init_card_map(card_map: &mut CardMap, pile: &Pile) {
    let mut card_ids: Vec<CardId> = pile.iter().map(|c| c.card_id).collect();
    card_ids.sort();

    for (idx, card_id) in card_ids.iter().enumerate() {
        card_map.insert(*card_id, idx);
    }
}

pub fn get_random_pile_with_no_winner<R: Rng>(hero: Class, monster: Class, rng: &mut R) -> Pile {
    let mut pile = get_start_from_classes(hero, monster, rng);
    randomize_sides(&mut pile, rng);
    while is_game_winner(&pile).is_some() {
        randomize_sides(&mut pile, rng);
    }
    return pile;
}

pub fn get_random_won_pile<R: Rng>(hero: Class, monster: Class, rng: &mut R) -> Pile {
    let mut pile = get_start_from_classes(hero, monster, rng);
    randomize_sides(&mut pile, rng);
    for card in &mut pile {
        if card.get_active_face().allegiance == Allegiance::Baddie {
            card.key = get_random_exhausted_face(rng, card.get_card_def());
        }
    }
    return pile;
}

pub fn randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        card_ptr.key = get_random_face(rng);
    }
}

fn main() {
    let hero = Class::Cursed;
    let monster = Class::Spider;
    let mut rng = thread_rng();
    let mut card_map: CardMap = HashMap::new();

    {
        let pile = get_start_from_classes(hero, monster, &mut rng);
        _init_card_map(&mut card_map, &pile);
    }

    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = net(&vs.root());
    let mut opt = nn::AdamW::default().build(&vs, 1e-4).unwrap();
    dbg!(&net);

    loop {
        for batch in 1..1000 {
            {
                let pile = get_random_pile_with_no_winner(hero, monster, &mut rng);
                let xs = pile_onehot(&card_map, &pile).unsqueeze(0);
                let ys = Tensor::from_slice(&[1.0]).unsqueeze(0);
                let loss = (net.forward(&xs) - ys)
                    .pow_tensor_scalar(2)
                    .sum(Kind::Float);
                opt.backward_step(&loss);
            }

            {
                let pile = get_random_won_pile(hero, monster, &mut rng);
                let xs = pile_onehot(&card_map, &pile).unsqueeze(0);
                let ys = Tensor::from_slice(&[0.0]).unsqueeze(0);
                let loss = (net.forward(&xs) - ys)
                    .pow_tensor_scalar(2)
                    .sum(Kind::Float);
                opt.backward_step(&loss);
            }
        }

        {
            {
                let pile = get_random_pile_with_no_winner(hero, monster, &mut rng);
                let xs = pile_onehot(&card_map, &pile).unsqueeze(0);
                let ys = Tensor::from_slice(&[1.0]).unsqueeze(0);
                let non_res = net.forward(&xs);
                let non_acc = non_res.accuracy_for_logits(&ys);
                dbg!((non_res.f_double_value(&[0, 0]), non_acc,));

                let pile = get_random_won_pile(hero, monster, &mut rng);
                let xs = pile_onehot(&card_map, &pile).unsqueeze(0);
                let ys = Tensor::from_slice(&[0.0]).unsqueeze(0);
                let won_res = net.forward(&xs);
                let won_acc = won_res.accuracy_for_logits(&ys);

                dbg!((won_res.f_double_value(&[0, 0]), won_acc));
                // println!(
                //     "test acc: {:5.2}%",
                //     100. * f64::from(&test_accuracy),
                // );
            }
        }
    }

    // let inp = empty_onehot().unsqueeze(0);
    // dbg!(&inp);
    // dbg!(inp.size());

    // let res = net.forward_t(&inp, false);
    // dbg!(&res);
    // dbg!(&res.size());
    // dbg!(&res.f_double_value(&[0]));
    //   // run().unwrap();
}
