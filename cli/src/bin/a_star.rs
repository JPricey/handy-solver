use clap::Parser;
use cli::run_a_star::run_a_star_solver;
use cli::{get_starting_pile_from_args, StandardArgs};
use handy_core::game::end_game::GameEndCheckType;

fn main() {
    let args = StandardArgs::parse();
    let start_pile = get_starting_pile_from_args(&args);
    println!("{start_pile:?}");
    run_a_star_solver(
        start_pile.into(),
        None,
        None,
        args.g_bias,
        Some(GameEndCheckType::Standard),
        true,
    );
}
