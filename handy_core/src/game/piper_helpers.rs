/*
File of helpers to deal with pipers modifier abilities
*/

use crate::game::card_ptr::CardPtrT;
use crate::game::primitives::{Action, ModifierAmount, Pile, Range};
use std::cmp;

pub fn any_card_has_modifiers(pile: &Pile) -> bool {
    for card_ptr in pile {
        if card_ptr.get_active_face().modifier.is_some() {
            return true;
        }
    }

    false
}

pub fn size_with_modifier(amount: usize, modifier: ModifierAmount) -> usize {
    cmp::max(0, (amount as ModifierAmount) + modifier) as usize
}

pub fn range_with_modifier(range: Range, modifier: ModifierAmount) -> Range {
    match range {
        Range::Inf => range,
        Range::Int(amount) => Range::Int(size_with_modifier(amount, modifier)),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModifierRangeType {
    Discrete,
    Infinity,
    None,
}

pub fn modifier_range_type_for_action(action: &Action) -> ModifierRangeType {
    match action {
        Action::Pull(range) | Action::Push(range) | Action::Claws(range) | Action::Hit(range) => {
            match range {
                Range::Inf => ModifierRangeType::Infinity,
                Range::Int(_) => ModifierRangeType::Discrete,
            }
        }
        Action::Quicken(_) | Action::Delay(_) => ModifierRangeType::Discrete,
        Action::Death
        | Action::Void
        | Action::CallAssist
        | Action::CallAssistTwice
        | Action::Inspire
        | Action::Hypnosis
        | Action::Ablaze
        | Action::Fireball
        | Action::FireballTwice
        | Action::Teleport
        | Action::Arrow
        | Action::ArrowTwice
        | Action::Heal
        | Action::Revive
        | Action::Rats
        | Action::Maneuver
        | Action::Backstab
        | Action::BackstabTwice
        | Action::Poison
        | Action::SpacedClaws(_) => ModifierRangeType::None,
    }
}

pub fn action_with_modified_range(action: &Action, modifier: ModifierAmount) -> Action {
    match action {
        Action::Pull(range) => Action::Pull(range_with_modifier(*range, modifier)),
        Action::Push(range) => Action::Push(range_with_modifier(*range, modifier)),
        Action::Hit(range) => Action::Hit(range_with_modifier(*range, modifier)),
        Action::Claws(range) => Action::Claws(range_with_modifier(*range, modifier)),
        Action::Quicken(amount) => Action::Quicken(size_with_modifier(*amount, modifier)),
        Action::Delay(amount) => Action::Delay(size_with_modifier(*amount, modifier)),
        Action::Death
        | Action::Void
        | Action::CallAssist
        | Action::CallAssistTwice
        | Action::Inspire
        | Action::Hypnosis
        | Action::Ablaze
        | Action::Fireball
        | Action::FireballTwice
        | Action::Teleport
        | Action::Arrow
        | Action::ArrowTwice
        | Action::Heal
        | Action::Revive
        | Action::Rats
        | Action::Maneuver
        | Action::Backstab
        | Action::BackstabTwice
        | Action::Poison
        | Action::SpacedClaws(_) => action.clone(),
    }
}

pub fn get_modifier_options(
    pile: &Pile,
    active_idx: usize,
    modifier_range_type: ModifierRangeType,
) -> Vec<(Vec<usize>, ModifierAmount)> {
    let mut results = Vec::new();
    if modifier_range_type == ModifierRangeType::None {
        return results;
    }

    // TODO: instead of only below active_idx, should allow anyone that isn't active
    for target_idx in (active_idx + 1)..pile.len() {
        // if target_idx == active_idx {
        //     continue;
        // }

        let active_card_ptr = pile[target_idx];
        if let Some(modifier) = active_card_ptr.get_active_face().modifier {
            // Don't bother with modifiers on infinity if the modifier has no effect anyway
            if modifier_range_type == ModifierRangeType::Infinity && modifier.mandatory.is_none() {
                continue;
            }

            let pre_results_len = results.len();

            results.push((vec![target_idx], modifier.amount));
            for result_idx in 0..pre_results_len {
                let mut new_result_entry = results[result_idx].clone();
                new_result_entry.0.push(target_idx);
                new_result_entry.1 += modifier.amount;
                results.push(new_result_entry);
            }
        }
    }

    results
}
