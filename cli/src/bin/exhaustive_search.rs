use clap::{ArgGroup, Parser};
use handy_core::game::*;
use handy_core::utils::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::mem::swap;

#[derive(Parser, Debug)]
#[clap(group(
            ArgGroup::new("win_type")
                .required(true)
                .args(&["win", "survive_until_top", "exhaust"]),
        ))]
struct ExaustiveSearchArgs {
    #[clap(long, value_parser=string_to_pile_result)]
    pile: Pile,

    #[clap(long)]
    turns: Option<usize>,

    #[clap(long)]
    rev: bool,

    #[clap(long)]
    win: bool,
    #[clap(long, value_parser=string_to_card_id_result)]
    survive_until_top: Option<CardId>,
    #[clap(long, value_parser=string_to_card_id_result)]
    exhaust: Option<CardId>,
}

pub enum VictoryConditionResult {
    Win,
    Loss,
    Continue,
}

trait VictoryConditionChecker {
    fn condition_result(&self, pile: &Pile) -> VictoryConditionResult;
}

#[derive(Clone, Debug)]
struct WinChecker {}
impl VictoryConditionChecker for WinChecker {
    fn condition_result(&self, pile: &Pile) -> VictoryConditionResult {
        match is_game_winner(pile) {
            WinType::Win => VictoryConditionResult::Win,
            WinType::Lose => VictoryConditionResult::Loss,
            WinType::Unresolved => VictoryConditionResult::Continue,
        }
    }
}

#[derive(Clone, Debug)]
struct SurviveUntilTopChecker {
    card_id: CardId,
}
impl VictoryConditionChecker for SurviveUntilTopChecker {
    fn condition_result(&self, pile: &Pile) -> VictoryConditionResult {
        // False if the card is dead anywhere
        for card_ptr in pile {
            if card_ptr.get_card_id() == self.card_id {
                if card_ptr.get_active_face().health == Health::Empty {
                    return VictoryConditionResult::Loss;
                }
                break;
            }
        }

        // If the card is on top and not dead, we win (we know it's not dead from the check above)
        if pile[0].get_card_id() == self.card_id {
            return VictoryConditionResult::Win;
        }

        match is_game_winner(pile) {
            // Hero probably shouldn't be allowed to win these?
            WinType::Win => VictoryConditionResult::Continue,
            WinType::Lose => VictoryConditionResult::Loss,
            WinType::Unresolved => VictoryConditionResult::Continue,
        }
    }
}

#[derive(Clone, Debug)]
struct ExhaustCardChecker {
    card_id: CardId,
}
impl VictoryConditionChecker for ExhaustCardChecker {
    fn condition_result(&self, pile: &Pile) -> VictoryConditionResult {
        // False if the card is dead anywhere
        for card_ptr in pile {
            if card_ptr.get_card_id() == self.card_id {
                if card_ptr.get_active_face().health == Health::Empty {
                    return VictoryConditionResult::Win;
                }
            }
        }

        match is_game_winner(pile) {
            WinType::Win => VictoryConditionResult::Continue,
            WinType::Lose => VictoryConditionResult::Loss,
            WinType::Unresolved => VictoryConditionResult::Continue,
        }
    }
}

fn resolve_condition_checker(args: &ExaustiveSearchArgs) -> Box<dyn VictoryConditionChecker> {
    if args.win {
        Box::new(WinChecker {})
    } else if let Some(card_id) = args.survive_until_top {
        Box::new(SurviveUntilTopChecker { card_id })
    } else if let Some(card_id) = args.exhaust {
        Box::new(ExhaustCardChecker { card_id })
    } else {
        panic!("Could not result condition checker from these args");
    }
}

type ParentStatesByLevel = Vec<HashMap<Pile, Vec<Pile>>>;
fn unwrap_solutions(
    pile: &Pile,
    level: usize,
    mut path: Vec<Pile>,
    parent_map: &ParentStatesByLevel,
) -> Vec<Vec<Pile>> {
    path.push(pile.clone());
    if level == 0 {
        return vec![path];
    }

    let parents = &parent_map[level - 1][pile];
    let mut res: Vec<Vec<Pile>> = vec![];
    for parent in parents {
        res.extend(unwrap_solutions(
            parent,
            level - 1,
            path.clone(),
            parent_map,
        ));
    }

    return res;
}

fn solve(
    init_pile: Pile,
    condition_checker: Box<dyn VictoryConditionChecker>,
    turns: Option<usize>,
) -> Vec<Vec<Pile>> {
    let mut turn_number: usize = 0;
    let mut current_piles = vec![];
    let mut next_piles = vec![init_pile.clone()];

    let mut parent_states_by_level: ParentStatesByLevel = Vec::new();

    loop {
        let mut parent_states: HashMap<Pile, Vec<Pile>> = HashMap::new();
        swap(&mut current_piles, &mut next_piles);
        next_piles.clear();

        let mut victories = vec![];

        while let Some(pile) = current_piles.pop() {
            let child_states = resolve_top_card(&GameStateNoEventLog::new(pile.clone()));

            for child_state in child_states {
                let child_pile = child_state.pile;

                if let Some(entry) = parent_states.get_mut(&child_pile) {
                    entry.push(pile.clone());
                } else {
                    parent_states.insert(child_pile.clone(), vec![pile.clone()]);

                    match condition_checker.condition_result(&child_pile) {
                        VictoryConditionResult::Win => victories.push(child_pile),
                        VictoryConditionResult::Loss => (),
                        VictoryConditionResult::Continue => next_piles.push(child_pile.clone()),
                    }
                }
            }
        }

        parent_states_by_level.push(parent_states);
        turn_number += 1;

        if victories.len() > 0 && turns.map_or(true, |turns| turn_number >= turns) {
            let mut result: Vec<Vec<Pile>> = vec![];
            for victory in victories {
                result.extend(unwrap_solutions(
                    &victory,
                    turn_number,
                    vec![],
                    &parent_states_by_level,
                ));
            }

            return result;
        }

        if turns.map_or(false, |turns| turn_number >= turns) {
            return vec![];
        }
    }
}

fn main() {
    let args = ExaustiveSearchArgs::parse();
    let condition_checker = resolve_condition_checker(&args);

    let mut pile = args.pile;
    if args.rev {
        pile.reverse();
    }

    let mut solutions = solve(pile, condition_checker, args.turns);
    // Filter out duplicate ways to have the same path
    solutions = solutions
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if solutions.len() == 0 {
        println!("No solutions");
    } else {
        println!(
            "Found {} solution after {} turns",
            solutions.len(),
            solutions[0].len() - 1
        );

        for mut solution in solutions {
            solution.reverse();
            println!("~~~");
            let mut last_pile = solution[0].clone();
            println!("{:?}", last_pile);

            for pile in solution[1..].iter().cloned() {
                print_steps_between_piles(&last_pile, &pile, &|x| println!("{}", x));
                println!("{:?}", pile);
                last_pile = pile;
            }
        }
    }
}
