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
        // TODO: incorporate modifiers for stances?
        Range::Stance(stance_type) => Range::Stance(stance_type), 
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
        Action::Pull(range)
        | Action::Push(range)
        | Action::Claws(range)
        | Action::Hit(range)
        | Action::Quicken(range)
        | Action::Delay(range) => match range {
            Range::Inf => ModifierRangeType::Infinity,
            Range::Int(_) => ModifierRangeType::Discrete,
            Range::Stance(_) => ModifierRangeType::Discrete,
        },
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
        Action::Quicken(range) => Action::Quicken(range_with_modifier(*range, modifier)),
        Action::Delay(range) => Action::Delay(range_with_modifier(*range, modifier)),
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

    for target_idx in (active_idx + 1)..pile.len() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::string_to_pile;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_modifier_options() {
        {
            // No modifiers
            let pile = string_to_pile("1 2 3 4 5");
            let results = get_modifier_options(&pile, 0, ModifierRangeType::Discrete);
            assert_eq!(results.len(), 0);
        }

        {
            // With modifiers
            let pile = string_to_pile("55 56 57");
            let results = get_modifier_options(&pile, 0, ModifierRangeType::Discrete);
            assert_eq!(
                results,
                vec![(vec![1], -2), (vec![2], 1), (vec![1, 2], -1),]
            );
        }
    }
}
