use web_ui::solver::SolverWorker;
use gloo::worker::Registrable;

fn main() {
    console_error_panic_hook::set_once();

    SolverWorker::registrar().register();
}
