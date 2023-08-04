use cli::get_starting_pile;
use cli::paths::*;
use handy_core::game::*;
use handy_core::solver::ida::*;
use handy_core::solver::*;

pub fn run(start_pile: Pile) {
    let matchup = try_get_matchup_from_pile(&start_pile).unwrap();
    let model = try_read_model_for_matchup(matchup).unwrap();
    let mut solver = IdaSolver::new(model);
    solver.solve(start_pile);
}

fn main() {
    let start_pile = get_starting_pile();
    run(start_pile)
}
