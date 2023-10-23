use crate::game::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixResult {
    Event(Event, Pile),
    Pile(Pile),
}

pub fn prefix_result_to_pile(prefix_result: &PrefixResult) -> Pile {
    match prefix_result {
        PrefixResult::Event(_, pile) => pile.clone(),
        PrefixResult::Pile(pile) => pile.clone(),
    }
}

pub fn find_final_piles_matching_prefix(root_pile: &Pile, prefix: &Vec<Event>) -> Vec<Pile> {
    let init_state = GameStateWithEventLog::new(root_pile.clone());
    let new_states = resolve_top_card(&init_state);
    let mut results: Vec<Pile> = Vec::new();

    for state in new_states {
        if state.events.len() < prefix.len() {
            continue;
        }
        let state_events_prefix: Vec<Event> =
            state.events[0..prefix.len()].iter().cloned().collect();

        if prefix == &state_events_prefix {
            results.push(state.pile.clone());
        }
    }
    return results;
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
    states: &Vec<GameStateWithEventLog>,
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
        let state_events_prefix: Vec<Event> =
            state.events[0..prefix.len()].iter().cloned().collect();

        if prefix == &state_events_prefix {
            if res.as_ref().map_or(true, |r| state.events.len() < r.1) {
                res = Some((state.events[prefix.len()].clone(), state.events.len()));
            }
        }
    }
    return res.map(|res| res.0);
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
        Event::SkipAction(card_ptr, wrapped_action) => format!(
            "{:?}: Skip {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        Event::SkipArrow => format!("Skip Arrow"),
        Event::AttackCard(card_idx, card_ptr, hit_type) => {
            format!("{:?}@{}: Targetted ({:?})", card_ptr, card_idx, hit_type)
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
        Event::MoveTarget(card_idx, card_ptr, move_type) => format!("{:?} {:?}@{}", move_type, card_ptr, card_idx),
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
        Event::Block(card_idx, card_ptr, cost) => {
            format!("{:?}@{}: Block({:?})", card_ptr, card_idx, cost)
        }
        Event::Dodge(card_idx, card_ptr, cost) => {
            format!("{:?}@{}: Dodge({:?})", card_ptr, card_idx, cost)
        }
        Event::OnHurt(card_idx, card_ptr) => {
            format!("{:?}@{}: Hurt", card_ptr, card_idx)
        }
        Event::PayEnergy(cards) => {
            let cards = cards
                .iter()
                .map(|(id, ptr)| format!("{ptr:?}@{id}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{cards}: Energy")
        }
        Event::Manouver(card_idx, card_ptr) => {
            format!("{:?}@{}: Manouver", card_ptr, card_idx)
        }
        Event::Swarm(card_idx, card_ptr) => {
            format!("{:?}@{}: Swarm", card_ptr, card_idx)
        }
        Event::UseActionAssistCard(assist_idx, assist_card_ptr) => format!(
            "{:?}@{}: Assist",
            assist_card_ptr, assist_idx
        ),
        Event::UseActionAssistRow(assist_idx, assist_card_ptr, assist_row_idx) => format!(
            "{:?}@{}: Assist Row {}",
            assist_card_ptr, assist_idx, assist_row_idx
        ),
        Event::ReactAssistUsed(card_idx, card_ptr, trigger, cost) => {
            format!("{:?}@{}: {:?} React Assited({:?})", card_ptr, card_idx, trigger, cost)
        }
        Event::SkipReactActionAssist => {
            "Skip React Action".to_string()
        }
        // Event::StartAction(card_ptr, wrapped_action) => format!(
        //     "{:?}: Trigger {}",
        //     card_ptr,
        //     format_wrapped_action(wrapped_action)
        // ),
        // _ => format!("{:?}", event),
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
        Action::DoubleArrow => "Arrowx2".to_owned(),
        Action::Quicken(_) => "Quicken".to_owned(),
        Action::Delay(_) => "Delay".to_owned(),
        Action::Fireball => "Fireball".to_owned(),
        Action::Ablaze => "Ablaze".to_owned(),
        Action::Teleport => "Teleport".to_owned(),
        Action::CallAssist => "CallAssist".to_owned(),
        Action::CallAssistTwice => "Assistx2".to_owned(),
        Action::Hit(_) => "Hit".to_owned(),
        Action::Inspire => "Inspire".to_owned(),
        Action::Heal => "Heal".to_owned(),
        Action::Manouver => "Manouver".to_owned(),
        Action::Revive => "Revive".to_owned(),
        Action::Claws(_) => "Claw".to_owned(),
    }
}

pub fn format_multiple_events(events: &[Event]) -> String {
    events
        .iter()
        .map(format_event_for_cli)
        .collect::<Vec<String>>()
        .join("|")
}
