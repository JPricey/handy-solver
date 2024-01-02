// CNN model. This should rearch 99.1% accuracy.

use anyhow::Result;
use ndarray::{s, Array1, Array2, ArrayView1};
use std::convert::TryInto;
use tch::{
    kind, nn,
    nn::init::DEFAULT_KAIMING_UNIFORM,
    nn::ConvConfig,
    nn::Init::Const,
    nn::ModuleT,
    nn::OptimizerConfig,
    nn::PaddingMode,
    nn::{linear, Module},
    Device, Kind, Tensor,
};

const PILE_SIZE: i64 = 9;
const CARD_SIZE: i64 = 13;
const INPUT_SIZE: i64 = PILE_SIZE * CARD_SIZE;

#[derive(Debug)]
struct Net {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl Net {
    fn new(vs: &nn::Path) -> Net {
        let conv1 = nn::conv2d(vs, 1, 32, 5, Default::default());
        let conv2 = nn::conv2d(vs, 32, 64, 5, Default::default());
        let fc1 = nn::linear(vs, 1024, 1024, Default::default());
        let fc2 = nn::linear(vs, 1024, 10, Default::default());
        Net {
            conv1,
            conv2,
            fc1,
            fc2,
        }
    }
}

impl nn::ModuleT for Net {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        xs.view([-1, 1, 28, 28])
            .apply(&self.conv1)
            .max_pool2d_default(2)
            .apply(&self.conv2)
            .max_pool2d_default(2)
            .view([-1, 1024])
            .apply(&self.fc1)
            .relu()
            .dropout(0.5, train)
            .apply(&self.fc2)
            .relu()
    }
}

pub fn run() -> Result<()> {
    let m = tch::vision::mnist::load_dir("/home/joe/data")?;
    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = Net::new(&vs.root());
    let mut opt = nn::AdamW::default().build(&vs, 1e-4)?;
    for epoch in 1..100 {
        for (bimages, blabels) in m.train_iter(256).shuffle().to_device(vs.device()) {
            let loss = net
                .forward_t(&bimages, true)
                .cross_entropy_for_logits(&blabels);
            opt.backward_step(&loss);
        }
        let test_accuracy =
            net.batch_accuracy_for_logits(&m.test_images, &m.test_labels, vs.device(), 1024);
        println!("epoch: {:4} test acc: {:5.2}%", epoch, 100. * test_accuracy,);
    }
    Ok(())
}

fn net(vs: &nn::Path) -> impl Module {
    let conv_output = 16;
    let l1_output = 32;

    nn::seq().add(nn::conv1d(
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
    .add(nn::linear(vs / "layer2", l1_output, 1, Default::default()))
}

fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4) }
}

fn empty_onehot() -> Tensor {
    let vec_source: Vec<f32> = vec![0.0; INPUT_SIZE as usize];
    let as_bytes = to_byte_slice(&vec_source);
    return Tensor::from_data_size(&as_bytes, &[CARD_SIZE, PILE_SIZE], Kind::Float);
}

fn main() {
    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = net(&vs.root());
    dbg!(&net);

    let inp = empty_onehot().unsqueeze(0);
    dbg!(&inp);
    dbg!(inp.size());

    let res = net.forward_t(&inp, false);
    dbg!(&res);
    dbg!(&res.size());
    // dbg!(&res.f_double_value(&[0]));
    //   // run().unwrap();
}
