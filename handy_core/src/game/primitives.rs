use crate::game::card_ptr::CardPtr;
use arrayvec::ArrayVec;
use bitflags::bitflags;
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, PartialOrd};
use strum_macros;

pub type CardId = u8;
pub type ConditionCountType = u8;
pub type TargetId = usize;
pub type TargetIds = Vec<TargetId>;

// pub type VecPile = Vec<CardPtr>;
// pub type BoxSlicePile = Box<[CardPtr]>;
// pub type ArrayPile = [CardPtr; 9];
pub type ArrayVecPile = ArrayVec<CardPtr, 9>;

pub type Pile = ArrayVecPile;

pub type PayEnergyArrType = ArrayVec<(usize, CardPtr), 4>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Health {
    Empty,
    Half,
    Full,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Ally,
    Enemy,
    Any,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Range {
    Inf,
    Int(usize),
}

#[derive(
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    Enum,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
)]
pub enum Allegiance {
    Hero,
    Baddie,
    Werewolf,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ClawSpaceType {
    Odd,
    Even,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    // Enemy Only
    Pull(Range),
    Push(Range),
    Death,
    Void,
    SpacedClaws(ClawSpaceType),
    // Player Only
    Arrow,
    DoubleArrow,
    Quicken(usize),
    Delay(usize),
    Fireball,
    Ablaze,
    Teleport,
    CallAssist,
    CallAssistTwice,
    // Both
    Hit(Range),
    Inspire,
    Heal,
    Manouver,
    Revive,
    Claws(Range),
    Backstab,
    BackstabTwice,
    Poison,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WrappedAction {
    pub action: Action,
    pub target: Target,
}

#[derive(strum_macros::Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SelfAction {
    Rotate,
    Flip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Energy(ConditionCountType),
    Rage(ConditionCountType),
    Dodge(ConditionCountType),
    ExhaustedAllies(usize),
}

#[derive(strum_macros::Display, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReactionTrigger {
    Block,
    Dodge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProvideAssistReaction {
    pub trigger: ReactionTrigger,
    pub assist_cost: SelfAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StandardReaction {
    pub trigger: ReactionTrigger,
    pub outcome: Option<SelfAction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestAssistReaction {
    pub outcome: Option<SelfAction>,
}

pub type WhenHitType = &'static Row;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reaction {
    Standard(StandardReaction),
    Assist(RequestAssistReaction),
    WhenHit(WhenHitType),
    Roll,
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Features: u8 {
        const NoFeature     = 0b00000000;
        const Weight        = 0b00000001;
        const Trap          = 0b00000010;
        const Web           = 0b00000100;
        const Venom         = 0b00001000;
        const Energy        = 0b00010000;
        const Wall          = 0b00100000;
        const Invulnerable  = 0b01000000;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Row {
    pub is_mandatory: bool,
    pub condition: Option<Condition>,
    pub actions: Vec<WrappedAction>,
    pub mandatory: Option<SelfAction>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FaceDef {
    pub allegiance: Allegiance,
    pub health: Health,
    pub features: Features,
    pub reaction: Option<Reaction>,
    pub reaction_assist: Option<ProvideAssistReaction>,
    pub swarm: Option<Row>,
    pub rows: Vec<Row>,
    pub assists: Vec<Row>,
    pub rage: ConditionCountType,
}

#[derive(
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Enum,
    Ord,
    PartialOrd,
    Deserialize,
    Serialize,
)]
pub enum FaceKey {
    A,
    B,
    C,
    D,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Class {
    Warrior,
    Huntress,
    Pyro,
    Cursed,
    Beastmaster,
    Assassin,
    Ogre,
    Vampire,
    Spider,
    Demon,
    Flora,
    Wall,
}

#[derive(Debug)]
pub struct CardDef {
    pub id: CardId,
    pub class: Class,
    pub faces: EnumMap<FaceKey, FaceDef>,
    pub is_back_start: bool,
}

impl PartialEq for CardDef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CardDef {}

use std::hash::Hash;
use std::hash::Hasher;
impl Hash for CardDef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(strum_macros::Display, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HitType {
    Hit,
    Arrow,
    Trap,
    Claw,
    Ablaze,
    Fireball,
    Backstab,
    Poison,
    Roll,
}

#[derive(strum_macros::Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MoveType {
    Quicken,
    Delay,
}

#[derive(strum_macros::Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndPileMoveType {
    Pull,
    Push,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    // Control flow
    PickRow(usize, usize, CardPtr), // row_idx, card_idx, card_ptr
    SkipTurn(CardPtr),
    SkipHit(HitType),
    BottomCard,

    SkipAction(CardPtr, WrappedAction),
    // StartAction(CardPtr, WrappedAction),
    AttackCard(usize, CardPtr, HitType),
    Damage(usize, CardPtr, HitType, FaceKey),

    // Special Attacks
    Death,
    Void(usize, CardPtr, FaceKey),
    Inspire(usize, CardPtr),

    // Moves
    MoveTarget(usize, CardPtr, MoveType), // index, ptr, type, amount
    MoveBy(usize, CardPtr, MoveType, usize), // anchor_index, anchor_card, move_type, distance
    MoveResult(MoveType, usize),          // type, amount

    Pull(usize, CardPtr),
    Push(usize, CardPtr),
    EndPileMoveResult(EndPileMoveType),

    Teleport(usize, CardPtr, usize, CardPtr),

    // Reported after the event
    Mandatory(CardPtr, SelfAction),

    // Targets
    FireballTarget(usize, CardPtr),
    Ablaze(usize, CardPtr, usize, CardPtr),
    ReactAssistUsed(usize, CardPtr, ReactionTrigger, SelfAction),

    // Heals
    Heal(usize, CardPtr),
    Revive(usize, CardPtr),

    // Reactions
    Block(usize, CardPtr, Option<SelfAction>),
    Dodge(usize, CardPtr, Option<SelfAction>),
    OnHurt(usize, CardPtr),

    // Other
    PayEnergy(PayEnergyArrType),
    Manouver(usize, CardPtr),
    Swarm(usize, CardPtr),
    UseActionAssistCard(usize, CardPtr), // card_idx, card_ptr
    UseActionAssistRow(usize, CardPtr, usize), // card_idx, card_ptr, row
    SkipReactActionAssist,
}

#[derive(strum_macros::Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WinType {
    Win,
    Lose,
    Unresolved,
}

impl WinType {
    pub fn is_over(&self) -> bool {
        *self != WinType::Unresolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_order() {
        assert!(Health::Full == Health::Full);
        assert!(Health::Full > Health::Half);
        assert!(Health::Full > Health::Empty);

        assert!(Health::Half < Health::Full);
        assert!(Health::Half == Health::Half);
        assert!(Health::Half > Health::Empty);

        assert!(Health::Empty < Health::Full);
        assert!(Health::Empty < Health::Half);
        assert!(Health::Empty == Health::Empty);
    }
}
