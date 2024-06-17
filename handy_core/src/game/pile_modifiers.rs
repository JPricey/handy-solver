/*
Helpers that mutate an input pile
*/

use crate::game::card_ptr::{CardPtr, CardPtrT};
use crate::game::game_state::EngineGameState;
use crate::game::primitives::{Event, FaceKey, Health, SelfAction};
use strum::IntoEnumIterator;

use super::card_modifiers::perform_card_self_action;

pub fn perform_mandatory_action<T: EngineGameState>(
    state: &mut T,
    self_action: SelfAction,
    active_idx: usize,
) {
    perform_card_self_action(self_action, &mut state.get_pile_mut()[active_idx]);
    state.mut_append_event(Event::Mandatory(state.get_pile()[active_idx], self_action));
}

pub fn bottom_top_card<T: EngineGameState>(state: &mut T) {
    state.get_pile_mut().rotate_left(1);
    state.mut_append_event(Event::BottomCard);
}

pub fn mut_exhaust_card_without_giving_options(card: &mut CardPtr) {
    if card.get_active_face().health == Health::Empty {
        return;
    }
    for key in FaceKey::iter() {
        if card.get_card_def().faces[key].health == Health::Empty {
            card.key = key;
            return;
        }
    }
    panic!("Could not find exhausted face of card");
}
