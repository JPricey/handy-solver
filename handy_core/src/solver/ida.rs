use crate::game::*;
use crate::solver::model::*;
use std::fmt::Debug;

type DepthType = u8;
const F_CUTOFF: f32 = 3.0;

#[derive(Debug)]
enum NodeResult {
    Loss,
    Win(Vec<Pile>),
}

pub struct IdaSolver {
    pub model: Model,
    pub total_iters: usize,
    pub max_depth: DepthType,
    pub f_score_cutoff: f32,
}

impl IdaSolver {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            total_iters: 0,
            max_depth: 250,
            f_score_cutoff: 0.0,
        }
    }

    fn _solve_recursive(&mut self, pile: &Pile, depth: DepthType) -> NodeResult {
        self.total_iters += 1;
        let resolution = is_game_winner(pile);
        if resolution == WinType::Win {
            self.max_depth = depth;
            println!("found solution at depth {}", depth);
            self.f_score_cutoff = self.max_depth as f32 + F_CUTOFF;
            return NodeResult::Win(vec![pile.clone()]);
        } else if resolution == WinType::Lose {
            return NodeResult::Loss;
        }

        let child_depth = depth + 1;
        if child_depth >= self.max_depth {
            return NodeResult::Loss;
        }

        let init_state = GameStateNoEventLog::new(pile.clone());
        let mut child_states: Vec<_> = resolve_top_card(&init_state)
            .into_iter()
            .map(|s| (self.model.score_pile(&s.pile), s.pile))
            .collect();
        child_states.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        child_states.dedup();

        let mut best_win = vec![];
        for (child_score, child_pile) in child_states {
            if child_depth as f32 + child_score > self.f_score_cutoff {
                continue;
            }

            // println!(
            //     "Checking pile {:?} at depth {}, score {}",
            //     child_pile, child_depth, child_score
            // );

            let child_result = self._solve_recursive(&child_pile, child_depth);
            match child_result {
                NodeResult::Loss => (), // Do nothing
                NodeResult::Win(mut win_path) => {
                    win_path.push(pile.clone());
                    if self.max_depth == child_depth {
                        return NodeResult::Win(win_path);
                    }
                    best_win = win_path;
                }
            }
        }

        if best_win.len() > 0 {
            return NodeResult::Win(best_win);
        }

        NodeResult::Loss
    }

    pub fn solve(&mut self, pile: Pile) {
        self.f_score_cutoff = self.model.score_pile(&pile) + F_CUTOFF;
        let result = self._solve_recursive(&pile, 0);
        dbg!(result);
    }
}
