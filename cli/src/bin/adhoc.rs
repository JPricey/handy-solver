use cli::*;
use handy_core::game::*;
use handy_core::solver::a_star::*;
use handy_core::utils::*;
use tch::{
    nn,
    nn::{
        init::DEFAULT_KAIMING_UNIFORM, ConvConfig, Init::Const, Module,
        PaddingMode,
    },
    Device,
};

const PATH: &str = "cursed-spider.safetensors";
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

fn main() {
    let hero = Class::Cursed;
    let monster = Class::Spider;

    let mut vs = nn::VarStore::new(Device::cuda_if_available());
    let net = net(&vs.root());
    vs.load(PATH).unwrap();
    let model = NNModel::new(hero, monster, Box::new(net));
    // let model = try_read_model_for_matchup((hero, monster)).unwrap();

    let start_pile = string_to_pile("27A 26A 31A 30A 32A 29A 28A 24A 25A ");

    let mut a_star_solver = AStarSolver::new(&vec![start_pile.clone()], Box::new(model));

    loop {
        let iter_result = a_star_solver.single_iter();
        match iter_result {
            AStarIterResult::Done(_) => {
                panic!("Could not solve pile: {:?}", start_pile);
            }
            AStarIterResult::NewBest(result) => {
                let path = a_star_solver.unroll_state(result);
                println!("found soln: {} iters / length {}", a_star_solver.total_iters, path.len());
                // return;
            }
            AStarIterResult::Continue(_) => {}
        }
    }
}
