use clap::Parser;
use cli::*;
use csv::Writer;
use handy_core::game::*;
use handy_core::solver::a_star::*;
use handy_core::solver::*;
use handy_core::utils::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use serde;
use std::fmt::Debug;
use std::time::{Duration, SystemTime};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, value_parser=parse_dot_separated_matchup)]
    pub matchup: Matchup,

    #[clap(short, long, default_value_t = 1.0)]
    g_bias: f32,

    #[clap(short, long, default_value = "0")]
    seed: String,

    #[clap(short, long, default_value_t = 10)]
    num_trials: usize,

    #[clap(short, long, default_value_t = 30)]
    trial_s: usize,

    #[clap(short, long, default_value = "default")]
    prefix: String,
}

#[derive(serde::Serialize, Debug)]
struct SolutionRow {
    trial: usize,
    duration_ms: f32,
    iters: usize,
    depth: DepthType,
}

type Solutions = Vec<SolutionRow>;

fn rate_model(
    start_pile: Pile,
    model: Model,
    g_bias: f32,
    trial_duration: Duration,
    trial: usize,
) -> Solutions {
    let now = SystemTime::now();
    let mut a_star_solver = AStarSolver::new(&vec![start_pile], model);
    a_star_solver.set_g_bias(g_bias);

    let mut solutions: Solutions = Vec::new();
    let mut iters: usize = 0;

    loop {
        let iter_result = a_star_solver.single_iter();
        match iter_result {
            AStarIterResult::Done(_) => {
                // println!("Done early");
                break;
            }
            AStarIterResult::NewBest(_) => {
                iters += 1;
                let duration = now.elapsed().unwrap();
                // println!(
                //     "{:?} {}: New best solution: {}",
                //     duration, iters, a_star_solver.max_depth
                // );
                solutions.push(SolutionRow {
                    depth: a_star_solver.max_depth,
                    duration_ms: duration.as_millis() as f32,
                    iters,
                    trial,
                });
                break;
            }
            AStarIterResult::Continue(_) => {
                iters += 1;
                if now.elapsed().unwrap() > trial_duration {
                    break;
                }
            }
        }
    }

    /*
    if let Some(row) = solutions.last() {
        let new_row = SolutionRow {
            depth: row.depth,
            duration_ms: now.elapsed().unwrap().as_millis() as f32,
            iters,
            trial,
        };
        solutions.push(new_row);
    }
    */

    // println!("Done after {:?}, {}", now.elapsed().unwrap(), iters);
    // println!("{:?}", &solutions);

    solutions
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let (hero, monster) = args.matchup;
    let output_file_name = format!(
        "data/model_charts/{:?}.{:?}-{}-{}-{}-{}.csv",
        hero, monster, args.prefix, &args.seed, args.trial_s, args.g_bias
    );
    println!("Writing to {output_file_name}");
    let mut writer = Writer::from_path(output_file_name).unwrap();

    let trial_duration = Duration::new(args.trial_s as u64, 0);
    let mut rng = Seeder::from(&args.seed).make_rng::<Pcg64>();
    for i in 0..args.num_trials {
        let start_pile = get_start_from_classes(hero, monster, &mut rng);
        let model = get_model_for_pile(&start_pile);

        println!("{}: {:?}", i, &start_pile);
        let solutions = rate_model(start_pile, model, args.g_bias, trial_duration.clone(), i);

        for row in solutions {
            writer.serialize(row).unwrap();
        }
        // writer.flush().unwrap();
    }
}
