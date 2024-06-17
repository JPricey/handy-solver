/*
This file is for functions that modify the state of a CardPtr
*/

use super::pile_utils::{flip_key, rotate_key};
use crate::game::card_ptr::CardPtr;
use crate::game::primitives::SelfAction;

pub fn perform_card_self_action(self_action: SelfAction, active_card: &mut CardPtr) {
    match self_action {
        SelfAction::Rotate => {
            active_card.key = rotate_key(active_card.key);
        }
        SelfAction::Flip => {
            active_card.key = flip_key(active_card.key);
        }
    }
}
