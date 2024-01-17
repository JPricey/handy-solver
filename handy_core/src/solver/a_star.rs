use crate::game::*;
use crate::solver::tiny_pile::*;
use crate::utils::*;
use priq::PriorityQueue;
use std::collections::BTreeMap;
// use std::collections::HashMap;
use crate::solver::model_t::ModelT;
use std::fmt::Debug;

// BTree is slower, but memory is more compact, and resize events are gradual
// type SeenMap = HashMap<TinyPile, SolverState>;
pub type SeenMap = BTreeMap<TinyPile, SolverState>;
pub type DepthType = u8;

#[derive(Debug)]
pub struct SolverState {
    parent: Option<TinyPile>,
    depth: DepthType,
}

// Dont enqueue new states if maxdepth + DEFAULT_F_SCORE_END_CUTOFF > new fscore
// Stop if the next state in the queue has fscore > maxdepth + DEFAULT_F_SCORE_END_CUTOFF
const DEFAULT_F_SCORE_END_CUTOFF_FROM_MAX: f32 = 20.0;

pub struct AStarSolver {
    pub tiny_pile_converter: TinyPileConverter,
    pub seen_states: SeenMap,
    pub queue: PriorityQueue<f32, TinyPile>,
    pub model: Box<dyn ModelT>,
    pub total_iters: usize,
    pub max_depth: DepthType,
    pub max_fscore: f32,
    pub fscore_depth_delta: f32,
    pub max_iters: usize,
    pub best_win: Option<TinyPile>,
}

#[derive(Debug)]
pub enum AStarDoneReason {
    MaxIters,
    FScoreCutoff,
    EmptyQueue,
}

#[derive(Debug)]
pub enum DoneIterResult {
    DepthCutoff,
    ClearedFromState,
    Computed,
}

#[derive(Debug)]
pub enum AStarIterResult {
    Done(AStarDoneReason),
    Continue(DoneIterResult),
    NewBest(TinyPile),
}

impl AStarSolver {
    pub fn new(seed_piles: &[Pile], model: Box<dyn ModelT>) -> Self {
        let start_pile = &seed_piles[0];
        let tiny_pile_converter = TinyPileConverter::new_from_pile(start_pile);
        let mut seen_states = SeenMap::new();
        let mut queue: PriorityQueue<f32, TinyPile> = PriorityQueue::new();

        for pile in seed_piles {
            seen_states.insert(
                tiny_pile_converter.pile_to_tiny_pile(&pile),
                SolverState {
                    parent: None,
                    depth: 0,
                },
            );
            queue.put(0.0, tiny_pile_converter.pile_to_tiny_pile(&pile));
        }

        let fscore_depth_delta = DEFAULT_F_SCORE_END_CUTOFF_FROM_MAX;
        let default_max_depth = 250;

        Self {
            tiny_pile_converter,
            seen_states,
            queue,
            model,
            total_iters: 0,
            max_depth: default_max_depth,
            max_fscore: default_max_depth as f32 + fscore_depth_delta,
            fscore_depth_delta,
            max_iters: usize::MAX,
            best_win: None,
        }
    }

    pub fn set_max_iters(&mut self, max_iters: usize) {
        self.max_iters = max_iters;
    }

    pub fn set_max_fscore(&mut self, max_fscore: f32) {
        self.max_fscore = max_fscore;
    }

    pub fn clamp_max_fscore(&mut self, max_fscore: f32) {
        if max_fscore < self.max_fscore {
            self.max_fscore = max_fscore;
        }
    }

    pub fn fscore_depth_delta(&mut self, delta: f32) {
        self.fscore_depth_delta = delta;
    }

    pub fn set_max_depth(&mut self, max_depth: DepthType) {
        self.max_depth = max_depth;
        self.clamp_max_fscore(max_depth as f32 + self.fscore_depth_delta);
    }

    pub fn single_iter(&mut self) -> AStarIterResult {
        self.total_iters += 1;
        if self.total_iters > self.max_iters {
            return AStarIterResult::Done(AStarDoneReason::MaxIters);
        }
        let Some((f_score, tiny_pile)) = self.queue.pop() else {
            return AStarIterResult::Done(AStarDoneReason::EmptyQueue);
        };

        if f_score > self.max_fscore {
            return AStarIterResult::Done(AStarDoneReason::FScoreCutoff);
        }

        let pile = self.tiny_pile_converter.tiny_pile_to_pile(&tiny_pile);

        let Some(current_seen_state) = self.seen_states.get(&tiny_pile) else {
            return AStarIterResult::Continue(DoneIterResult::ClearedFromState);
        };

        let child_depth = current_seen_state.depth + 1;
        if child_depth > self.max_depth - 1 {
            return AStarIterResult::Continue(DoneIterResult::DepthCutoff);
        }

        let init_state = GameStateNoEventLog::new(pile.clone());
        let new_states = resolve_top_card(&init_state);

        for state in new_states {
            let new_pile = state.get_pile().clone();
            let new_tiny_pile = self.tiny_pile_converter.pile_to_tiny_pile(&new_pile);
            let maybe_winner = is_game_winner(&new_pile);

            if let Some(winner) = maybe_winner {
                if winner == Allegiance::Hero {
                    if child_depth < self.max_depth {
                        self.best_win = Some(new_tiny_pile);
                        self.set_max_depth(child_depth);
                        self.clear_depth();

                        self.seen_states.insert(
                            new_tiny_pile,
                            SolverState {
                                parent: Some(tiny_pile),
                                depth: child_depth as DepthType,
                            },
                        );
                        return AStarIterResult::NewBest(new_tiny_pile);
                    }
                } else {
                    continue;
                }
            }

            if let Some(current_child_entry) = self.seen_states.get_mut(&new_tiny_pile) {
                if current_child_entry.depth > child_depth as DepthType {
                    current_child_entry.depth = child_depth as DepthType;
                    current_child_entry.parent = Some(tiny_pile);

                    let new_score = child_depth as f32 + self.model.score_pile(&new_pile);
                    if new_score <= self.max_fscore {
                        self.queue.put(new_score, new_tiny_pile);
                    }
                }
            } else {
                let new_score = child_depth as f32 + self.model.score_pile(&new_pile);
                if new_score <= self.max_fscore {
                    self.queue.put(new_score, new_tiny_pile);
                }

                self.seen_states.insert(
                    new_tiny_pile,
                    SolverState {
                        parent: Some(tiny_pile),
                        depth: child_depth as DepthType,
                    },
                );
            };
        }
        return AStarIterResult::Continue(DoneIterResult::Computed);
    }

    fn clear_depth(&mut self) {
        self.seen_states.retain(|_, v| v.depth < self.max_depth);
    }

    pub fn reset_queue_and_fscore(&mut self) {
        let remaining_iters = self.max_iters - self.total_iters;
        if self.queue.len() <= remaining_iters {
            return;
        }

        let mut new_queue = PriorityQueue::new();
        for _ in 0..remaining_iters {
            let Some((k, v)) = self.queue.pop() else {
                break;
            };

            if k > self.max_fscore {
                break;
            }

            new_queue.put(k, v);
        }

        self.clamp_max_fscore(self.queue.peek().unwrap().0);
        self.queue = new_queue;
    }

    pub fn print_solution_custom_logger(&self, pile: &Pile, log: &dyn Fn(&str) -> ()) {
        let tiny_pile = self.tiny_pile_converter.pile_to_tiny_pile(pile);
        let solve_state = self.seen_states.get(&tiny_pile).unwrap();

        if let Some(tiny_parent) = &solve_state.parent {
            let parent = self.tiny_pile_converter.tiny_pile_to_pile(&tiny_parent);
            self.print_solution_custom_logger(&parent, &log);

            let possible_paths = resolve_top_card(&GameStateWithEventLog::new(parent.clone()));
            for path in &possible_paths {
                if &path.pile == pile {
                    for e in &path.events {
                        let line = format!("\t{}", format_event_for_cli(e));
                        log(&line);
                    }
                    let line = format!("{:?} / {}", pile, self.model.score_pile(pile));
                    log(&line);
                    return;
                }
            }
            let line = format!("{:?}: ?Unknown Path. This is a bug?", pile);
            log(&line);
        } else {
            let line = format!("{:?}", pile);
            log(&line)
        }
    }

    pub fn print_solution_from_pile(&self, pile: &Pile) {
        self.print_solution_custom_logger(&pile, &|x| println!("{}", x));
    }

    pub fn print_solution_from_tiny(&self, tiny_pile: &TinyPile) {
        let pile = self.tiny_pile_converter.tiny_pile_to_pile(tiny_pile);
        self.print_solution_from_pile(&pile);
    }

    pub fn unroll_state(&self, final_tiny_pile: TinyPile) -> Vec<Pile> {
        let mut result = vec![final_tiny_pile];

        loop {
            let last_pile = result.last().unwrap();
            // let last_tiny_pile = converter.pile_to_tiny_pile(last_pile);
            let entry = self.seen_states.get(&last_pile).unwrap();
            if let Some(parent) = &entry.parent {
                result.push(parent.clone());
            } else {
                break;
            }
        }

        result.reverse();
        result
            .iter()
            .map(|tiny| self.tiny_pile_converter.tiny_pile_to_pile(tiny))
            .collect()
    }

    pub fn convert_to_pile(&self, tiny_pile: TinyPile) -> Pile {
        self.tiny_pile_converter.tiny_pile_to_pile(&tiny_pile)
    }
}
