use crate::{
    game::card_defs::CARDS,
    game::card_ptr::{CardPtrAsTuple, CardPtrT},
    game::primitives::{CardDef, CardId, FaceKey},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CardPtrAsId {
    pub card_id: CardId,
    pub key: FaceKey,
}

impl CardPtrT for CardPtrAsId {
    fn new_from_id(id: CardId, key: FaceKey) -> Self {
        Self { card_id: id, key }
    }

    fn get_card_id(&self) -> CardId {
        self.card_id
    }

    fn get_card_face(&self) -> FaceKey {
        self.key
    }

    fn get_card_def(&self) -> &CardDef {
        &CARDS.get_card(self.card_id as usize)
    }
}

impl Debug for CardPtrAsId {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_display_string())
    }
}

impl Serialize for CardPtrAsId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_tuple().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CardPtrAsId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tuple = CardPtrAsTuple::deserialize(deserializer)?;
        Ok(Self::new_from_tuple(tuple))
    }
}
