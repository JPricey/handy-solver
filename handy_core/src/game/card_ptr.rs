pub mod card_ptr_as_id;

use crate::game::card_ptr::card_ptr_as_id::CardPtrAsId;
use crate::game::primitives::*;
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CardPtrAsTuple(CardId, FaceKey);

pub trait CardPtrT: Copy + Clone + PartialEq + Eq + Hash + Ord + PartialOrd + Debug {
    fn new_from_id(id: CardId, key: FaceKey) -> Self;

    fn new_from_tuple(tuple: CardPtrAsTuple) -> Self {
        Self::new_from_id(tuple.0, tuple.1)
    }

    fn get_card_id(&self) -> CardId;

    fn get_card_face(&self) -> FaceKey;

    fn get_card_def(&self) -> &CardDef;

    fn get_active_face(&self) -> &FaceDef {
        &self.get_card_def().faces[self.get_card_face()]
    }

    fn to_tuple(&self) -> CardPtrAsTuple {
        CardPtrAsTuple(self.get_card_id(), self.get_card_face())
    }

    fn to_display_string(&self) -> ColoredString {
        let string = format!("{}{}", self.get_card_id(), self.get_card_face());
        match self.get_active_face().allegiance {
            Allegiance::Hero => string.blue(),
            Allegiance::Monster => string.red(),
            Allegiance::Werewolf => string.yellow(),
            Allegiance::Rat => string.yellow(),
            Allegiance::Quest => string.white(),
        }
    }
}

// Default CardPtr type
pub type CardPtr = CardPtrAsId;
