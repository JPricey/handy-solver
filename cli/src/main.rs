use handy_core::game_state::*;
use handy_core::game_utils::*;
use handy_core::interface_utils::*;
use handy_core::types::*;
use handy_core::arg_parse::get_starting_pile;

use std::collections::HashSet;

fn collapse_states<T: EngineGameState>(states: Vec<T>) -> Vec<T> {
    let mut result = vec![];
    let mut seen_piles = HashSet::new();

    for state in states {
        if !seen_piles.contains(state.get_pile()) {
            seen_piles.insert(state.get_pile().clone());
            result.push(state);
        }
    }

    result
}

// fn find_next_events_matching_prefix(&states: Vec<GameStateWithEventLog>, prefix: Vec<Event>) = {
//     let mut results: Vec<Some(Event)> = vec![];
//     for state in &states {
//         if prefix == state.events {
//             results.push(None);
//         }
//     }
//     return results
// }
//
// pub fn start_card_play(start_pile: Pile) {
//     let state = GameStateWithEventLog::new(active_pile);
//     let future_states = resolve_top_card(&state);
//
//     let mut event_prefix: Vec<Event> = vec![];
//
//     loop {
//
//     }
// }

pub fn start_cli_game(mut active_pile: Pile) {
    loop {
        if let Some(winner) = is_game_winner(&active_pile) {
            println!("Game is over. {:?} wins!", winner);
            break;
        }

        let state = GameStateWithEventLog::new(active_pile);
        let options = collapse_states(resolve_top_card(&state));

        loop {
            println!("Current State: {:?}", state.get_pile());
            for (i, option) in options.iter().enumerate() {
                println!(
                    "{}: {:?} via {} ({})",
                    i,
                    option.pile,
                    format_multiple_events(&option.events),
                    i
                );
            }

            if options.len() == 1 {
                println!("Making only choice");
                active_pile = options[0].pile.clone();
                break;
            }

            let maybe_choice: Result<usize, _> = text_io::try_read!();
            if let Ok(choice) = maybe_choice {
                if choice < options.len() {
                    active_pile = options[choice].pile.clone();

                    break;
                }
            }
            println!("Invalid row number. Try again");
        }
    }
}

fn main() {
    let start_pile = get_starting_pile();
    start_cli_game(start_pile.into());
}
