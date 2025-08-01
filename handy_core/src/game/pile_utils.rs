/*
File of helpers that to answer questions about the state of a pile.
All helpers are on immutable piles or cards.
*/

use crate::game::card_ptr::{CardPtr, CardPtrT};
use crate::game::primitives::{
    Action, Allegiance, Class, Condition, ConditionCostType, Event, FaceKey, Features, Health,
    Pile, Range, RangeType, Reaction, ReactionTrigger, SkipActionReason, StanceType, Target,
    TroupeType, WrappedAction,
};
use strum::IntoEnumIterator;

pub fn does_card_have_energy(card_ptr: &CardPtr) -> bool {
    card_ptr
        .get_active_face()
        .features
        .intersects(Features::Energy)
}

pub fn does_card_have_dodge(card_ptr: &CardPtr) -> bool {
    if let Some(reaction) = card_ptr.get_active_face().reaction {
        if let Reaction::Standard(_, standard_reaction) = reaction {
            if standard_reaction.trigger == ReactionTrigger::Dodge {
                return true;
            }
        }
    }

    false
}

pub fn get_cost_predicate(condition_cost_type: ConditionCostType) -> fn(&CardPtr) -> bool {
    match condition_cost_type {
        ConditionCostType::Dodge => does_card_have_dodge,
        ConditionCostType::Energy => does_card_have_energy,
    }
}

pub fn is_moveable_target(
    card_ptr: &CardPtr,
    active_allegiance: Allegiance,
    target: Target,
) -> bool {
    let target_face = card_ptr.get_active_face();
    let target_allegiance = target_face.allegiance;

    if !is_allegiance_match_for_target(active_allegiance, target_allegiance, target) {
        return false;
    }

    if target_allegiance != active_allegiance
        && target_face
            .features
            .intersects(Features::Weight | Features::Invulnerable)
    {
        return false;
    }

    true
}

pub fn is_allegiance_match_for_target(me: Allegiance, other: Allegiance, spec: Target) -> bool {
    match spec {
        Target::Any => {
            // Neutral cards can only be targetted by other neutral cards, or heros
            // NPC will ignore them
            other != Allegiance::Neutral || me == Allegiance::Neutral || me == Allegiance::Hero
        }
        Target::Ally => me == other,
        // NPC will ignore neutral cards
        Target::Enemy => me != other && (other != Allegiance::Neutral || me == Allegiance::Hero),
    }
}

pub fn is_allegiance_match_for_effect(me: Allegiance, other: Allegiance, spec: Target) -> bool {
    match spec {
        Target::Any => true,
        Target::Ally => me == other,
        Target::Enemy => me != other,
    }
}

pub fn find_heal_target(
    pile: &Pile,
    target_health: Health,
    allegiance: Allegiance,
    target_type: Target,
    starting_idx: usize,
) -> Option<usize> {
    let max_range = pile.len();

    for i in usize::from(starting_idx)..max_range {
        let active_card_ptr = &pile[i];
        if is_allegiance_match_for_target(
            allegiance,
            active_card_ptr.get_active_face().allegiance,
            target_type,
        ) {
            if active_card_ptr.get_active_face().health == target_health {
                return Some(i);
            }
        }
    }

    None
}

// Optimization: enum array?
// Optimization: convert to lookup table?
pub fn find_hurt_faces(card: &CardPtr) -> Vec<FaceKey> {
    let card_def = card.get_card_def();
    let current_health = card_def.faces[card.key].health;
    if current_health == Health::Empty {
        return vec![];
    }

    let mut results = vec![];
    let target_health = one_damage(current_health);
    for key in FaceKey::iter() {
        if key == card.key {
            continue;
        }

        if card_def.faces[key].health == target_health {
            results.push(key);
        }
    }

    results
}

pub fn one_damage(health: Health) -> Health {
    match health {
        Health::Full => Health::Half,
        Health::Half => Health::Empty,
        Health::Empty => {
            panic!("Tried to hurt card with no health")
        }
    }
}

pub fn rotate_key(key: FaceKey) -> FaceKey {
    match key {
        FaceKey::A => FaceKey::B,
        FaceKey::B => FaceKey::A,
        FaceKey::C => FaceKey::D,
        FaceKey::D => FaceKey::C,
    }
}

pub fn flip_key(key: FaceKey) -> FaceKey {
    match key {
        FaceKey::A => FaceKey::C,
        FaceKey::C => FaceKey::A,
        FaceKey::B => FaceKey::D,
        FaceKey::D => FaceKey::B,
    }
}

pub fn exhaust_card(card: &CardPtr) -> Vec<FaceKey> {
    if card.get_active_face().health == Health::Empty {
        panic!("Card is already exhausted")
    }

    let mut results = vec![];
    for key in FaceKey::iter() {
        if card.get_card_def().faces[key].health == Health::Empty {
            results.push(key)
        }
    }
    if results.len() == 0 {
        panic!("Could not find exhausted face of card");
    }

    results
}

pub fn is_action_prevented(
    pile: &Pile,
    feature: Features,
    active_idx: usize,
    active_allegiance: Allegiance,
) -> bool {
    if let Some(infront) = pile.get(active_idx + 1) {
        let active_face = infront.get_active_face();
        active_face.allegiance != active_allegiance && active_face.features.intersects(feature)
    } else {
        false
    }
}

pub fn can_card_be_damaged(pile: &Pile, target_idx: usize) -> bool {
    let target_card = pile[target_idx];
    let target_face = target_card.get_active_face();

    if target_face.features.intersects(Features::Invulnerable) {
        return false;
    }

    if !target_face.features.intersects(Features::United) {
        return true;
    }

    let target_allegiance = target_face.allegiance;

    if target_idx >= 1 {
        let card_above = pile[target_idx - 1];
        let card_above_allegiance = card_above.get_active_face().allegiance;
        if target_allegiance == card_above_allegiance {
            return false;
        }
    }

    if target_idx < pile.len() - 1 {
        let card_below = pile[target_idx + 1];
        let card_below_allegiance = card_below.get_active_face().allegiance;
        if target_allegiance == card_below_allegiance {
            return false;
        }
    }

    return true;
}

pub fn maybe_skip_action_event_for_spider_feature(
    pile: &Pile,
    active_idx: usize,
    active_allegiance: Allegiance,
    wrapped_action: &WrappedAction,
) -> Option<Event> {
    match wrapped_action.action {
        Action::Pull(_)
        | Action::Push(_)
        | Action::Quicken(_)
        | Action::Delay(_)
        | Action::Teleport => {
            if is_action_prevented(pile, Features::Web, active_idx, active_allegiance) {
                Some(Event::SkipAction(
                    pile[active_idx],
                    wrapped_action.clone(),
                    SkipActionReason::Web,
                ))
            } else {
                None
            }
        }
        Action::Hit(_)
        | Action::Claws(_)
        | Action::SpacedClaws(_)
        | Action::Void
        | Action::Ablaze
        | Action::Fireball
        | Action::FireballTwice
        | Action::Arrow
        | Action::ArrowTwice
        | Action::Heal
        | Action::Revive
        | Action::Rats
        | Action::Maneuver
        | Action::Backstab
        | Action::BackstabTwice
        | Action::Poison => {
            if is_action_prevented(pile, Features::Venom, active_idx, active_allegiance) {
                Some(Event::SkipAction(
                    pile[active_idx],
                    wrapped_action.clone(),
                    SkipActionReason::Venom,
                ))
            } else {
                None
            }
        }
        Action::Death
        | Action::CallAssist
        | Action::CallAssistTwice
        | Action::Inspire
        | Action::Hypnosis
        | Action::Key(_) => None,
    }
}

pub fn get_next_troupe(
    pile: &Pile,
    active_idx: usize,
    active_allegiance: Allegiance,
) -> Option<TroupeType> {
    for i in active_idx + 1..pile.len() {
        let other_card = pile[i];
        if other_card.get_active_face().allegiance == active_allegiance
            && other_card.get_card_def().class == Class::Troupe
        {
            return other_card.get_card_def().troupe_type;
        }
    }

    None
}

pub fn get_stance_count(
    pile: &Pile,
    active_idx: usize,
    active_allegiance: Allegiance,
    stance_type: StanceType,
) -> RangeType {
    let mut total = 0;
    for i in active_idx + 1..pile.len() {
        let active_face = pile[i].get_active_face();
        if active_face.allegiance == active_allegiance {
            let is_match = match stance_type {
                StanceType::Open => active_face.features.intersects(Features::Open),
                StanceType::Fist => active_face.features.intersects(Features::Fist),
            };
            if is_match {
                total += 1;
            }
        }
    }
    total
}

pub fn get_integer_range(
    pile: &Pile,
    active_idx: usize,
    active_allegiance: Allegiance,
    range: Range,
) -> RangeType {
    match range {
        Range::Int(x) => x,
        Range::Inf => pile.len(),
        Range::Stance(stance) => get_stance_count(pile, active_idx, active_allegiance, stance),
    }
}

pub fn get_range_cap(
    pile: &Pile,
    active_idx: usize,
    active_allegiance: Allegiance,
    range: Range,
) -> RangeType {
    let int_range = get_integer_range(pile, active_idx, active_allegiance, range);
    std::cmp::min(pile.len(), active_idx + int_range + 1)
}

pub fn is_boolean_condition_met(
    pile: &Pile,
    active_idx: usize,
    active_allegiance: Allegiance,
    condition: Condition,
) -> bool {
    match condition {
        Condition::Stance(stance_type, count) => {
            get_stance_count(pile, active_idx, active_allegiance, stance_type)
                >= RangeType::from(count)
        }
        _ => panic!("unimplemeneted boolean condition {:?}", condition),
    }
}
