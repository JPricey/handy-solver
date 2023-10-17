use crate::get_model_for_pile;
use handy_core::game::*;
use handy_core::solver::a_star::*;

pub fn run_a_star_solver(
    start_pile: Pile,
    max_depth: Option<DepthType>,
    max_iters: Option<usize>,
) -> Vec<Pile> {
    let model = get_model_for_pile(&start_pile);

    let mut a_star_solver = AStarSolver::new(&vec![start_pile], model);
    if let Some(def_max_iters) = max_iters {
        a_star_solver.set_max_iters(def_max_iters);
    }
    if let Some(def_max_depth) = max_depth {
        a_star_solver.set_max_depth(def_max_depth);
    }
    let mut count: usize = 0;
    loop {
        count += 1;
        let iter_result = a_star_solver.single_iter();
        match iter_result {
            AStarIterResult::Done(reason) => {
                println!("Stopping Solver: {:?}", reason);
                if let Some(best_win) = a_star_solver.best_win {
                    return a_star_solver.unroll_state(best_win);
                } else {
                    return vec![];
                }
            }
            AStarIterResult::NewBest(tiny_pile) => {
                println!("New best solution: {}", a_star_solver.max_depth);
                a_star_solver.print_solution_from_tiny(&tiny_pile);
            }
            AStarIterResult::Continue(_) => {
                if let Some(def_max_iter) = max_iters {
                    if a_star_solver.queue.len() > def_max_iter * 2 {
                        println!("Trimming queue");
                        a_star_solver.reset_queue_and_fscore();
                    }
                }
                if count > 100_000 {
                    println!(
                        "Iteration: {:?}, queue_size: {}, seen_size: {}, f_score: {}, best_len: {} max_fscore: {}",
                        a_star_solver.total_iters,
                        a_star_solver.queue.len(),
                        a_star_solver.seen_states.len(),
                        a_star_solver.queue.peek().map_or(0.0, |e| e.0),
                        a_star_solver.max_depth,
                        a_star_solver.max_fscore,
                    );
                    count = 0;
                }
            }
        }
    }
}
