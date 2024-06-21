use cli::get_starting_pile;
use cli::paths::*;
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use std::collections::HashSet;

fn format_prefix_result(prefix_result: &PrefixResult) -> String {
    match prefix_result {
        PrefixResult::Event(event, pile) => {
            format!("{} {:?}", format_event_for_cli(event), pile)
        }
        PrefixResult::Pile(pile) => format!("Done Activation: {:?}", pile),
    }
}

fn card_activation_result_via_choices(pile: &Pile) -> Pile {
    let mut current_pile = pile.clone();
    let mut current_events: Vec<Event> = vec![];

    loop {
        let state = GameStateWithPileTrackedEventLog::new(pile.clone());
        let future_states =
            resolve_top_card_starting_with_prefix_dedupe_excess(&state, &current_events);
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
    //let options = collapse_states(resolve_top_card(&state));
    let options = resolve_top_card(&state);
    let matchup = try_get_matchup_from_pile(pile).unwrap();
    let model = try_read_model_for_matchup(matchup).unwrap();

    loop {
        println!("Current State: {:?}", pile);
        for (i, option) in options.iter().enumerate() {
            println!(
                "{}: {:?} via {} ({}) ({})",
                i,
                option.pile,
                format_multiple_events(&option.events),
                i,
                model.score_pile(&option.pile)
            );
        }
        let unique_choices: HashSet<Pile> =
            options.iter().map(|option| option.pile.clone()).collect();
        println!("Unique Choices: {}", unique_choices.len());

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
        let resolution = is_game_winner(&active_pile);
        if resolution.is_over() {
            println!("You {:?}", resolution);
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
    start_cli_game(start_pile.into(), true);
}
