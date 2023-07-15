use super::types::*;
use regex::Regex;

pub fn string_to_card_ptr_result(input: &str) -> Result<CardPtr, ()> {
    let re = Regex::new(r"(\d+)([a-dA-D]?)").unwrap();
    let Some(captures) = re.captures(input) else { return Err(()) };
    let (_, [id_str, key_str]) = captures.extract();
    let key: FaceKey = key_str
        .to_uppercase()
        .to_owned()
        .parse()
        .unwrap_or(FaceKey::A);

    let Ok(id) = id_str.parse() else { return Err(()) };
    Ok(CardPtr::new_from_id(id, key))
}

pub fn string_to_card_ptr(input: &str) -> CardPtr {
    string_to_card_ptr_result(input).unwrap()
}

pub fn string_to_pile_result(input: &str) -> Result<Pile, String> {
    let mut result = vec![];

    let re = Regex::new(r"(\d+[a-dA-D]?)").unwrap();

    for (_, [card_ptr_str]) in re.captures_iter(input).map(|c| c.extract()) {
        let Ok(card_ptr) = string_to_card_ptr_result(card_ptr_str) else { return Err(card_ptr_str.to_owned()) };
        result.push(card_ptr);
    }

    if result.len() == 0 {
        Err("Could not parse any cards".to_owned())
    } else {
        Ok(result.into())
    }
}

pub fn string_to_pile(input: &str) -> Pile {
    string_to_pile_result(input).unwrap()
}

pub fn format_wrapped_action(wrapped_action: WrappedAction) -> String {
    format!("{:?} {:?}", wrapped_action.action, wrapped_action.target)
}

pub fn format_event_for_cli(event: Event) -> String {
    match event {
        Event::PickRow(row_num, card_ptr) => format!("{:?}: Row {}", card_ptr, row_num),
        Event::SkipTurn => "Skip".to_owned(),
        Event::BottomCard(card_ptr) => format!("{:?}: Bottomed", card_ptr),
        Event::SkipAction(card_ptr, wrapped_action) => format!(
            "{:?}: Skip {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        Event::StartAction(card_ptr, wrapped_action) => format!(
            "{:?}: Trigger {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        Event::AttackCard(card_ptr, wrapped_action) => {
            format!("{} {:?}", format_wrapped_action(wrapped_action), card_ptr)
        }
        Event::UseActionAssist(assist_idx, assist_card_ptr, assist_row_idx) => format!(
            "{:?}@{}: Assist Row {}",
            assist_card_ptr, assist_idx, assist_row_idx
        ),
        Event::Damage(card_idx, card_ptr, hit_type, result_face) => format!(
            "{:?}@{}: Damage({:?})=>{:?}",
            card_ptr, card_idx, hit_type, result_face
        ),
        Event::Death => format!("Death"),
        Event::Void(card_idx, card_ptr, face_key) => {
            format!("{:?}@{}: Void=>{:?}", card_ptr, card_idx, face_key)
        }
        Event::Inspire(card_idx, card_ptr) => format!("Inspire {:?}@{}", card_ptr, card_idx),
        Event::Quicken(card_idx, card_ptr, amount) => {
            format!("Quicken {:?}@{}+{}", card_ptr, card_idx, amount)
        }
        Event::Delay(card_idx, card_ptr, amount) => {
            format!("Delay {:?}@{}-{}", card_ptr, card_idx, amount)
        }
        Event::Pull(card_idx, card_ptr) => format!("Pull {:?}@{}", card_ptr, card_idx),
        Event::Push(card_idx, card_ptr) => format!("Push {:?}@{}", card_ptr, card_idx),
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
        Event::ReactAssistUsed(card_idx, card_ptr, cost) => {
            format!("{:?}@{}: React Assited({:?})", card_ptr, card_idx, cost)
        }
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
        Event::PayEnergy(card_idx, card_ptr) => {
            format!("{:?}@{}: Energy", card_ptr, card_idx)
        }
        Event::Manouver(card_idx, card_ptr) => {
            format!("{:?}@{}: Manouver", card_ptr, card_idx)
        }
        Event::Swarm(card_idx, card_ptr) => {
            format!("{:?}@{}: Swarm", card_ptr, card_idx)
        } // _ => format!("{:?}", event),
    }
}

pub fn format_multiple_events(events: &[Event]) -> String {
    events
        .iter()
        .cloned()
        .map(format_event_for_cli)
        .collect::<Vec<String>>()
        .join("|")
}
