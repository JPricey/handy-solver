use std::time::Duration;

use futures::{FutureExt, StreamExt};
use gloo::timers::future::sleep;
use gloo::worker::reactor::{reactor, ReactorScope};
use handy_core::game::*;
use handy_core::solver::a_star::*;
use handy_core::solver::*;
use handy_core::utils::*;
use leptos::log;

use futures::sink::SinkExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ControlSignal {
    SetModel(Model),
    SetRootPiles(Vec<String>),
    ClearRootPiles,
    End,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum OutputSignal {
    Start,
    SolutionCrumb(Vec<String>),
    Init,
    Sleeping,
    Working,
    Done,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SolverState {
    Init,
    Idle,
    Pending,
    Working,
}

struct SolverWorkerState {
    model: Option<Model>,
    root_piles: Vec<Pile>,
    a_star_solver: Option<AStarSolver>,
    state: SolverState,
}

const ITER_BATCH_SIZE: usize = 1000;
const SLEEP_TIME_MS: u64 = 100;

impl SolverWorkerState {
    fn new() -> Self {
        Self {
            a_star_solver: None,
            model: None,
            root_piles: vec![],
            state: SolverState::Init,
        }
    }

    fn clear_solving_state(&mut self) {
        self.state = SolverState::Init;
        self.a_star_solver = None;
    }

    fn set_model(&mut self, model: Model) {
        self.clear_solving_state();
        self.model = Some(model);
        self._check_should_start();
    }

    fn set_root_piles(&mut self, root_piles: Vec<Pile>) {
        self.clear_solving_state();
        self.root_piles = root_piles;
        self._check_should_start();
    }

    fn clear_root_piles(&mut self) {
        self.clear_solving_state();
        self.root_piles = vec![];
    }

    fn _check_should_start(&mut self) {
        if self.root_piles.len() > 0 {
            if let Some(model) = &self.model {
                self.state = SolverState::Working;
                self.a_star_solver = Some(AStarSolver::new(&self.root_piles, model.clone()));
            }
        }
    }

    async fn do_work(&mut self) -> OutputSignal {
        match self.state {
            SolverState::Init => {
                sleep(Duration::from_millis(SLEEP_TIME_MS)).fuse().await;
                return OutputSignal::Init;
            }
            SolverState::Idle | SolverState::Pending => {
                sleep(Duration::from_millis(SLEEP_TIME_MS)).fuse().await;
                return OutputSignal::Sleeping;
            }
            SolverState::Working => {
                if let Some(ref mut a_star_solver) = self.a_star_solver {
                    for _ in 0..ITER_BATCH_SIZE {
                        let iter_result = a_star_solver.single_iter();
                        match iter_result {
                            AStarIterResult::Done(_reason) => {
                                self.state = SolverState::Idle;
                                return OutputSignal::Done;
                            }
                            AStarIterResult::NewBest(tiny_pile) => {
                                let unrolled = a_star_solver.unroll_state(tiny_pile);
                                let strings: Vec<_> =
                                    unrolled.iter().map(|pile| format!("{pile:?}")).collect();
                                return OutputSignal::SolutionCrumb(strings);
                            }
                            AStarIterResult::Continue(_) => {}
                        }
                    }
                    return OutputSignal::Working;
                } else {
                    self.state = SolverState::Idle;
                    return OutputSignal::Sleeping;
                }
            }
        }
    }
}

#[reactor]
pub async fn SolverWorker(mut scope: ReactorScope<ControlSignal, OutputSignal>) {
    scope.send(OutputSignal::Start).await.unwrap();
    log!("Worker started");
    let mut solver_worker_state = SolverWorkerState::new();

    loop {
        futures::select_biased! {
            signal = scope.next() => {
                if let Some(signal) = signal {
                    // log!("worker got signal: {:?}", signal);
                    match signal {
                        ControlSignal::SetModel(model) => {
                            solver_worker_state.set_model(model);
                        }
                        ControlSignal::SetRootPiles(pile_strings) => {
                            let root_piles: Vec<_> =
                                pile_strings.iter().map(|s| string_to_pile(s)).collect();
                            solver_worker_state.set_root_piles(root_piles);
                        }
                        ControlSignal::ClearRootPiles => {
                            solver_worker_state.clear_root_piles();
                        }
                        ControlSignal::End => {
                            break;
                        }
                    }

                    let output_signal = match solver_worker_state.state {
                        SolverState::Working => OutputSignal::Working,
                        SolverState::Idle | SolverState::Pending => OutputSignal::Sleeping,
                        SolverState::Init => OutputSignal::Init,
                    };
                    scope.send(output_signal).await.unwrap();
                }
            },
            _ = sleep(Duration::from_millis(0)).fuse() => {
                let result = solver_worker_state.do_work().await;
                // log!("worker result: {:?}", result);
                scope.send(result).await.unwrap();
            },
        }
    }

    log!("Worker Stopped");
}
