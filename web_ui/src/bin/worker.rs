use gloo::worker::Registrable;
use web_ui::solver::SolverWorker;

fn main() {
    console_error_panic_hook::set_once();

    SolverWorker::registrar().register();
}
