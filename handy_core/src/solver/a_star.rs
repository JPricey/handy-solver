use crate::game::end_game::is_game_winner;
use crate::game::*;
use crate::solver::model_t::ModelT;
use crate::solver::tiny_pile::*;
use crate::utils::*;
use end_game::GameEndCheckType;
use priq::PriorityQueue;
use std::collections::BTreeMap;
use std::fmt::Debug;

// BTree is slower, but memory is more compact, and resize events are gradual
// type SeenMap = HashMap<StoredPileT, SolverState>;
pub type SeenMap<StoredPileT> = BTreeMap<StoredPileT, SolverState<StoredPileT>>;
pub type DepthType = u8;

#[derive(Debug)]
pub struct SolverState<StoredPileT> {
    parent: Option<StoredPileT>,
    depth: DepthType,
}

// Dont enqueue new states if maxdepth + DEFAULT_F_SCORE_END_CUTOFF > new fscore
// Stop if the next state in the queue has fscore > maxdepth + DEFAULT_F_SCORE_END_CUTOFF
const DEFAULT_F_SCORE_END_CUTOFF_FROM_MAX: f32 = 20.0;

pub struct AStarSolver<StoredPileT, StorageConverterT> {
    pub tiny_pile_converter: StorageConverterT,
    pub seen_states: SeenMap<StoredPileT>,
    pub queue: PriorityQueue<f32, StoredPileT>,
    pub model: Box<dyn ModelT>,
    pub total_iters: usize,
    pub max_depth: DepthType,
    pub max_fscore: f32,
    pub fscore_depth_delta: f32,
    pub g_bias: f32,
    pub h_bias: f32,
    pub max_iters: usize,
    pub best_win: Option<StoredPileT>,
    pub game_end_check_type: GameEndCheckType,
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
    NewBest(Pile),
}

impl<StoredPileT, StorageConverterT> AStarSolver<StoredPileT, StorageConverterT>
where
    StoredPileT: StorablePileT,
    StorageConverterT: PileStorageConverter<StoredPileT>,
{
    pub fn new(seed_piles: &[Pile], model: Box<dyn ModelT>) -> Self {
        let start_pile = &seed_piles[0];
        let tiny_pile_converter = StorageConverterT::new_from_pile(start_pile);
        let mut seen_states = SeenMap::<StoredPileT>::new();
        let mut queue: PriorityQueue<f32, StoredPileT> = PriorityQueue::new();

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
            g_bias: 1.0,
            h_bias: 1.0,
            max_depth: default_max_depth,
            max_fscore: default_max_depth as f32 + fscore_depth_delta,
            fscore_depth_delta,
            max_iters: usize::MAX,
            best_win: None,
            game_end_check_type: GameEndCheckType::Standard,
        }
    }

    pub fn set_game_end_check_type(&mut self, game_end_check_type: GameEndCheckType) {
        self.game_end_check_type = game_end_check_type;
    }

    pub fn set_g_bias(&mut self, g_bias: f32) {
        self.g_bias = g_bias;
        self.h_bias = 2.0 - g_bias;
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
            let resolution = is_game_winner(&new_pile, self.game_end_check_type);

            if resolution == WinType::Win {
                if child_depth < self.max_depth {
                    self.best_win = Some(new_tiny_pile.clone());
                    self.set_max_depth(child_depth);
                    self.clear_depth();

                    self.seen_states.insert(
                        new_tiny_pile.clone(),
                        SolverState {
                            parent: Some(tiny_pile),
                            depth: child_depth as DepthType,
                        },
                    );
                    return AStarIterResult::NewBest(new_pile);
                }
            } else if resolution == WinType::Lose {
                continue;
            }

            if let Some(current_child_entry) = self.seen_states.get_mut(&new_tiny_pile) {
                if current_child_entry.depth > child_depth as DepthType {
                    current_child_entry.depth = child_depth as DepthType;
                    current_child_entry.parent = Some(tiny_pile.clone());

                    let new_score = self.g_bias * child_depth as f32
                        + self.h_bias * self.model.score_pile(&new_pile);
                    if new_score <= self.max_fscore {
                        self.queue.put(new_score, new_tiny_pile);
                    }
                }
            } else {
                let new_score = self.g_bias * child_depth as f32
                    + self.h_bias * self.model.score_pile(&new_pile);
                if new_score <= self.max_fscore {
                    self.queue.put(new_score, new_tiny_pile.clone());
                }

                self.seen_states.insert(
                    new_tiny_pile,
                    SolverState {
                        parent: Some(tiny_pile.clone()),
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

    pub fn print_solution_from_tiny(&self, tiny_pile: &StoredPileT) {
        let pile = self.tiny_pile_to_pile(tiny_pile);
        self.print_solution_from_pile(&pile);
    }

    pub fn tiny_pile_to_pile(&self, tiny_pile: &StoredPileT) -> Pile {
        self.tiny_pile_converter.tiny_pile_to_pile(tiny_pile)
    }

    pub fn unroll_state(&self, final_pile: Pile) -> Vec<Pile> {
        let mut result = vec![self.tiny_pile_converter.pile_to_tiny_pile(&final_pile)];

        loop {
            let last_pile = result.last().unwrap();
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

    pub fn convert_to_pile(&self, tiny_pile: StoredPileT) -> Pile {
        self.tiny_pile_converter.tiny_pile_to_pile(&tiny_pile)
    }
}
