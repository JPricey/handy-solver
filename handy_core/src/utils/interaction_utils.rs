use crate::game::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixResult {
    Event(Event, Pile),
    Pile(Pile),
}

pub fn does_event_prefix_fit_sequence_for_state(
    prefix: &Vec<Event>,
    state: &GameStateWithPileTrackedEventLog,
) -> bool {
    let events = &state.events;
    if events.len() < prefix.len() {
        return false;
    }

    for i in 0..prefix.len() {
        if prefix[i] != events[i].1 {
            return false;
        }
    }

    true
}

pub fn should_skip_event_in_prefix_match(event: &Event) -> bool {
    match event {
        Event::StartAction(_, wrapped_action) => {
            match wrapped_action.action {
                Action::Claws(_) => false,
                Action::SpacedClaws(_) => false,
                _ => true,
            }
        },
        _ => false,
    }
}

pub fn get_next_available_events_past_prefix_allowing_skips(
    prefix: &Vec<Event>,
    state: &GameStateWithPileTrackedEventLog,
) -> Option<(Pile, Vec<Event>)> {
    // Must first match the prefix
    if !does_event_prefix_fit_sequence_for_state(prefix, state) {
        return None;
    }

    let state_events = &state.events;
    let mut following_events = Vec::new();
    let mut next_event_index = prefix.len();

    loop {
        let Some(next_choice) = state_events.get(next_event_index) else {
            break;
        };
        let next_event = &next_choice.1;
        following_events.push(next_event.clone());
        next_event_index += 1;

        if !should_skip_event_in_prefix_match(next_event) {
            break;
        }
    }

    let final_pile = state_events[next_event_index - 1].0.clone();

    Some((final_pile, following_events))
}

pub fn prefix_result_to_pile(prefix_result: &PrefixResult) -> Pile {
    match prefix_result {
        PrefixResult::Event(_, pile) => pile.clone(),
        PrefixResult::Pile(pile) => pile.clone(),
    }
}

pub fn find_final_piles_matching_prefix(root_pile: &Pile, prefix: &Vec<Event>) -> Vec<Pile> {
    let init_state = GameStateWithPileTrackedEventLog::new(root_pile.clone());
    let new_states = resolve_top_card_starting_with_prefix_dedupe_excess(&init_state, &prefix);
    let mut results: Vec<Pile> = Vec::new();

    for state in new_states {
        if state.events.len() < prefix.len() {
            continue;
        }
        let mut is_match = true;
        for i in 0..prefix.len() {
            if prefix[i] != state.events[i].1 {
                is_match = false;
                break;
            }
        }

        if is_match {
            results.push(state.pile.clone());
        }
    }

    results
}

pub fn find_next_events_matching_prefix(
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

pub fn find_next_event_matching_prefix_and_with_final_state(
    states: &Vec<GameStateWithPileTrackedEventLog>,
    prefix: &Vec<Event>,
    final_pile: &Pile,
) -> Option<Event> {
    let mut res: Option<(Event, usize)> = None;

    for state in states {
        if &state.pile != final_pile {
            continue;
        }

        if state.events.len() <= prefix.len() {
            continue;
        }
        let maybe_next_events =
            get_next_available_events_past_prefix_allowing_skips(prefix, &state.clone().into());
        let Some((_, following_events)) = maybe_next_events else {
            continue;
        };
        let Some(next_event) = following_events.last() else {
            continue;
        };

        if res.as_ref().map_or(true, |r| state.events.len() < r.1) {
            res = Some((next_event.clone(), state.events.len()));
        }
    }
    res.map(|res| res.0)
}

pub fn compact_pile_string(pile: &Pile, sep: &str) -> String {
    pile.iter()
        .map(|x| format!("{:?}", x))
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn format_wrapped_action(wrapped_action: &WrappedAction) -> String {
    format!("{:?} {:?}", wrapped_action.action, wrapped_action.target)
}

pub fn format_event_for_cli(event: &Event) -> String {
    match event {
        Event::PickRow(row_num, _, card_ptr) => format!("{:?}: Row {}", card_ptr, row_num),
        Event::SkipTurn(_) => "Skip Turn".to_owned(),
        Event::BottomCard => format!("Bottom"),
        Event::SkipAction(card_ptr, wrapped_action, _) => format!(
            "{:?}: Skip {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        Event::SkipHit(hit_type) => format!("Skip {:?}", hit_type),
        Event::AttackCard(card_idx, card_ptr, hit_type) => {
            format!("{:?}@{}: Targeted ({:?})", card_ptr, card_idx, hit_type)
        }
        Event::Damage(card_idx, card_ptr, hit_type, result_face) => format!(
            "{:?}@{}: Damage({:?})=>{:?}",
            card_ptr, card_idx, hit_type, result_face
        ),
        Event::Death => format!("Death"),
        Event::Void(card_idx, card_ptr, face_key) => {
            format!("{:?}@{}: Void=>{:?}", card_ptr, card_idx, face_key)
        }
        Event::Inspire(card_idx, card_ptr) => format!("Inspire {:?}@{}", card_ptr, card_idx),
        Event::Pull(card_idx, card_ptr) => format!("Pull {:?}@{}", card_ptr, card_idx),
        Event::Push(card_idx, card_ptr) => format!("Push {:?}@{}", card_ptr, card_idx),
        Event::EndPileMoveResult(move_type) => format!("Perform {:?}", move_type),

        Event::MoveTarget(card_idx, card_ptr, move_type) => {
            format!("{:?} {:?}@{}", move_type, card_ptr, card_idx)
        }
        Event::MoveBy(_, _, move_type, amount) => format!("{:?} by {}", move_type, amount),
        Event::MoveResult(_, _) => format!("Perform move"),
        Event::Teleport(card_idx1, card_ptr1, card_idx2, card_ptr2) => format!(
            "Teleport {:?}@{} <> {:?}@{}",
            card_ptr1, card_idx1, card_ptr2, card_idx2,
        ),
        Event::Mandatory(card_ptr, self_action) => {
            format!("{:?}: Forced({:?})", card_ptr, self_action)
        }
        Event::FireballTarget(card_idx, card_ptr) => {
            format!("{:?}@{}: Fireball", card_ptr, card_idx)
        }
        Event::Ablaze(card_idx1, card_ptr1, card_idx2, card_ptr2) => format!(
            "Ablaze {:?}@{}<>{:?}@{}",
            card_ptr1, card_idx1, card_ptr2, card_idx2,
        ),
        Event::Heal(card_idx, card_ptr) => {
            format!("{:?}@{}: Heal", card_ptr, card_idx)
        }
        Event::Revive(card_idx, card_ptr) => {
            format!("{:?}@{}: Revive", card_ptr, card_idx)
        }
        Event::Block(card_idx, card_ptr, cost, _) => {
            format!("{:?}@{}: Block({:?})", card_ptr, card_idx, cost)
        }
        Event::Dodge(card_idx, card_ptr, cost, _) => {
            format!("{:?}@{}: Dodge({:?})", card_ptr, card_idx, cost)
        }
        Event::OnHurt(card_idx, card_ptr) => {
            format!("{:?}@{}: Hurt", card_ptr, card_idx)
        }
        Event::PayRowConditionCosts(_, cards) => {
            let cards = cards
                .iter()
                .map(|(id, ptr)| format!("{ptr:?}@{id}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{cards}: Energy")
        }
        Event::Maneuver(card_idx, card_ptr) => {
            format!("{:?}@{}: Maneuver", card_ptr, card_idx)
        }
        Event::Swarm(card_idx, card_ptr) => {
            format!("{:?}@{}: Swarm", card_ptr, card_idx)
        }
        Event::UseActionAssistCard(assist_idx, assist_card_ptr) => {
            format!("{:?}@{}: Assist", assist_card_ptr, assist_idx)
        }
        Event::UseActionAssistRow(assist_idx, assist_card_ptr, assist_row_idx) => format!(
            "{:?}@{}: Assist Row {}",
            assist_card_ptr, assist_idx, assist_row_idx
        ),
        Event::ReactAssistUsed(card_idx, card_ptr, trigger, cost) => {
            format!(
                "{:?}@{}: {:?} React Assisted({:?})",
                card_ptr, card_idx, trigger, cost
            )
        }
        Event::SkipReactActionAssist => "Skip React Action".to_string(),
        Event::StartAction(card_ptr, wrapped_action) => format!(
            "{:?}: Start {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        _ => format!("{:?}", event),
    }
}

pub fn action_simple_name(wrapped_action: &WrappedAction) -> String {
    match wrapped_action.action {
        Action::Pull(_) => "Pull".to_owned(),
        Action::Push(_) => "Push".to_owned(),
        Action::Death => "Death".to_owned(),
        Action::Void => "Void".to_owned(),
        Action::SpacedClaws(space_type) => match space_type {
            ClawSpaceType::Odd => "Claw I".to_owned(),
            ClawSpaceType::Even => "Claw II".to_owned(),
        },
        Action::Arrow => "Arrow".to_owned(),
        Action::ArrowTwice => "Arrow x2".to_owned(),
        Action::Quicken(_) => "Quicken".to_owned(),
        Action::Delay(_) => "Delay".to_owned(),
        Action::Fireball => "Fireball".to_owned(),
        Action::FireballTwice => "Fireball x2".to_owned(),
        Action::Ablaze => "Ablaze".to_owned(),
        Action::Teleport => "Teleport".to_owned(),
        Action::CallAssist => "CallAssist".to_owned(),
        Action::CallAssistTwice => "Assist x2".to_owned(),
        Action::Hit(_) => "Hit".to_owned(),
        Action::Inspire => "Inspire".to_owned(),
        Action::Heal => "Heal".to_owned(),
        Action::Maneuver => "Maneuver".to_owned(),
        Action::Revive => "Revive".to_owned(),
        Action::Claws(_) => "Claw".to_owned(),
        Action::Backstab => "Backstab".to_owned(),
        Action::BackstabTwice => "Backstab x2".to_owned(),
        Action::Poison => "Poison".to_owned(),
        Action::Rats => "Rats".to_owned(),
        Action::Hypnosis => "Hypnosis".to_owned(),
        Action::Key(_) => "Key".to_owned(),
    }
}

pub fn format_multiple_events(events: &[Event]) -> String {
    events
        .iter()
        .map(format_event_for_cli)
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use crate::utils::string_to_pile;

    use super::*;

    fn collect_next_options(
        prefix: &Vec<Event>,
        final_state_options: &Vec<GameStateWithPileTrackedEventLog>,
    ) -> Vec<(Pile, Vec<Event>)> {
        final_state_options
            .iter()
            .filter_map(|s| get_next_available_events_past_prefix_allowing_skips(prefix, s))
            .collect()
    }

    #[test]
    fn test_get_next_available_events_past_prefix_allowing_skips_skips_wrapped_action_events() {
        let start_pile = string_to_pile("4A 6A 5A 3A 9A 8A 1A 7A 2A");
        let start_state = GameStateWithPileTrackedEventLog::new(start_pile.clone());
        let final_state_options = resolve_top_card(&start_state);

        {
            // After we pick row 0, we have several options, and skip the HitAction start prompt
            let prefix = vec![Event::PickRow(0, 0, start_pile[0])];

            let next_options = collect_next_options(&prefix, &final_state_options);
            assert!(next_options.len() == 5);
            let next_options_events = next_options
                .iter()
                .map(|(_, events)| events)
                .cloned()
                .collect::<Vec<_>>();

            // Sees 1 step forward events
            assert!(next_options_events.contains(&vec![Event::SkipAction(
                start_pile[0],
                WrappedAction::new(Action::Hit(Range::Int(4)), Target::Any),
                SkipActionReason::Choice,
            ),]));

            // Sees attack events, and moves forward
            assert!(next_options_events.contains(&vec![
                Event::StartAction(
                    start_pile[0],
                    WrappedAction::new(Action::Hit(Range::Int(4)), Target::Any),
                ),
                Event::AttackCard(3, start_pile[3], HitType::Hit),
            ]));
        }

        {
            // If we skip, the only possible next action is to bottom
            let prefix = vec![
                Event::PickRow(0, 0, start_pile[0]),
                Event::SkipAction(
                    start_pile[0],
                    WrappedAction::new(Action::Hit(Range::Int(4)), Target::Any),
                    SkipActionReason::Choice,
                ),
            ];

            let next_options = collect_next_options(&prefix, &final_state_options);
            assert!(next_options.len() == 1);
            assert!(next_options[0].1 == vec![Event::BottomCard]);
        }
    }
}
