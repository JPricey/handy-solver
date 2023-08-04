use cli::get_starting_pile;
use cli::run_a_star::run_a_star_solver;

fn main() {
    let start_pile = get_starting_pile();
    println!("{start_pile:?}");
    run_a_star_solver(start_pile.into(), None, None);
}
