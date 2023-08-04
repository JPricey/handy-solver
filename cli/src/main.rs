use cli_lib::*;
use handy_core::game_state::*;
use handy_core::game_utils::*;
use handy_core::types::*;

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixResult {
    Event(Event, Pile),
    Pile(Pile),
}

fn find_next_events_matching_prefix(
    states: &Vec<GameStateWithPileTrackedEventLog>,
    prefix: &Vec<Event>,
) -> Vec<PrefixResult> {
    let mut results: Vec<PrefixResult> = vec![];
    for state in states {
        if state.events.len() < prefix.len() {
            continue;
        }
        let state_events_prefix: Vec<Event> = state.events[0..prefix.len()]
            .iter()
            .map(|(_, event)| event.clone())
            .collect();
        if prefix == &state_events_prefix {
            if state.events.len() == prefix.len() {
                results.push(PrefixResult::Pile(state.pile.clone()));
            } else {
                let state_event = state.events[prefix.len()].clone();
                let new_event = PrefixResult::Event(state_event.1, state_event.0);
                if !results.contains(&new_event) {
                    results.push(new_event);
                }
            }
        }
    }
    return results;
}

fn format_prefix_result(prefix_result: &PrefixResult) -> String {
    match prefix_result {
        PrefixResult::Event(event, pile) => {
            format!("{} {:?}", format_event_for_cli(event), pile)
        }
        PrefixResult::Pile(pile) => format!("Done Activation: {:?}", pile),
    }
}

fn card_activation_result_via_choices(pile: &Pile) -> Pile {
    let state = GameStateWithPileTrackedEventLog::new(pile.clone());
    let future_states = resolve_top_card(&state);

    let mut current_pile = pile.clone();
    let mut current_events: Vec<Event> = vec![];

    loop {
        let event_options = find_next_events_matching_prefix(&future_states, &current_events);

        loop {
            println!("Current State: {:?}", current_pile);
            for (i, option) in event_options.iter().enumerate() {
                println!("{}: {}", i, format_prefix_result(option));
            }

            let maybe_choice: Result<usize, _> = if event_options.len() == 1 {
                println!("Making only choice");
                Ok(0)
            } else {
                text_io::try_read!()
            };

            if let Ok(choice) = maybe_choice {
                if choice < event_options.len() {
                    let prefix_result = &event_options[choice];
                    match prefix_result {
                        PrefixResult::Event(event, pile) => {
                            current_pile = pile.clone();
                            current_events.push(event.clone());
                        }
                        PrefixResult::Pile(pile) => return pile.clone(),
                    }
                    break;
                }
            }
            println!("Invalid row number. Try again");
        }
    }
}

fn card_activation_result_via_all_outcomes(pile: &Pile) -> Pile {
    let state = GameStateWithEventLog::new(pile.clone());
    let options = collapse_states(resolve_top_card(&state));

    loop {
        println!("Current State: {:?}", pile);
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
            return options[0].pile.clone();
        }

        let maybe_choice: Result<usize, _> = text_io::try_read!();
        if let Ok(choice) = maybe_choice {
            if choice < options.len() {
                return options[choice].pile.clone();
            }
        }
        println!("Invalid row number. Try again");
    }
}

pub fn start_cli_game(mut active_pile: Pile, is_interactive_mode: bool) {
    loop {
        if let Some(winner) = is_game_winner(&active_pile) {
            println!("Game is over. {:?} wins!", winner);
            break;
        }

        if is_interactive_mode {
            active_pile = card_activation_result_via_choices(&active_pile);
        } else {
            active_pile = card_activation_result_via_all_outcomes(&active_pile);
        }
    }
}

fn main() {
    let start_pile = get_starting_pile();
    start_cli_game(start_pile.into(), false);
}
