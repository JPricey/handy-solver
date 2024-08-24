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
pub const MAX_PILE_LEN: usize = 12;

pub type ArrayVecPile = ArrayVec<CardPtr, MAX_PILE_LEN>;

pub type Pile = ArrayVecPile;

pub type PayCostArrType = ArrayVec<(usize, CardPtr), 4>;
pub type ModifierArrType = ArrayVec<(usize, CardPtr), 4>;
pub type RangeType = usize;

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
    Int(RangeType),
    Stance(StanceType),
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
    Rat,
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
    ArrowTwice,
    Quicken(Range),
    Delay(Range),
    Fireball,
    FireballTwice,
    Ablaze,
    Teleport,
    CallAssist,
    CallAssistTwice,
    Backstab,
    BackstabTwice,
    Poison,
    Rats,
    Hypnosis,
    // Both
    Hit(Range),
    Inspire,
    Heal,
    Maneuver,
    Revive,
    Claws(Range),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WrappedAction {
    pub action: Action,
    pub target: Target,
}

impl WrappedAction {
    pub fn new(action: Action, target: Target) -> Self {
        Self { action, target }
    }
}

#[derive(strum_macros::Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SelfAction {
    Rotate,
    Flip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConditionCostType {
    Energy,
    Dodge,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StanceType {
    Open,
    Fist,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Cost(ConditionCostType, ConditionCountType),
    Rage(ConditionCountType),
    Troupe(TroupeType),
    Stance(StanceType, ConditionCountType),
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
    Standard(Option<Condition>, StandardReaction),
    Assist(RequestAssistReaction),
    WhenHit(WhenHitType),
    Roll(Option<SelfAction>),
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Features: u16 {
        const NoFeature     = 0b0000000000;
        const Weight        = 0b0000000001;
        const Trap          = 0b0000000010;
        const Web           = 0b0000000100;
        const Venom         = 0b0000001000;
        const Energy        = 0b0000010000;
        const Wall          = 0b0000100000;
        const Invulnerable  = 0b0001000000;
        const Open          = 0b0010000000;
        const Fist          = 0b0100000000;
        const United        = 0b1000000000;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Row {
    pub is_mandatory: bool,
    pub condition: Option<Condition>,
    pub actions: Vec<WrappedAction>,
    pub mandatory: Option<SelfAction>,
}

pub type ModifierAmount = i8;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Modifier {
    pub amount: ModifierAmount,
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
    pub modifier: Option<Modifier>,
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Enum, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Class {
    Dummy,
    Warrior,
    Huntress,
    Pyro,
    Cursed,
    Beastmaster,
    Assassin,
    Monk,
    Ogre,
    Vampire,
    Spider,
    Demon,
    Flora,
    Wall,
    Piper,
    Troupe,
    Ooze,
}

pub const HEROS: [Class; 8] = [
    Class::Warrior,
    Class::Huntress,
    Class::Pyro,
    Class::Cursed,
    Class::Beastmaster,
    Class::Assassin,
    Class::Piper,
    Class::Monk,
];

pub const BADDIES: [Class; 8] = [
    Class::Ogre,
    Class::Vampire,
    Class::Spider,
    Class::Demon,
    Class::Flora,
    Class::Wall,
    Class::Troupe,
    Class::Ooze,
];

impl Class {
    pub fn is_hero(self) -> bool {
        match self {
            Class::Warrior
            | Class::Huntress
            | Class::Pyro
            | Class::Cursed
            | Class::Beastmaster
            | Class::Assassin
            | Class::Piper
            | Class::Monk => true,

            Class::Dummy
            | Class::Ogre
            | Class::Vampire
            | Class::Spider
            | Class::Demon
            | Class::Flora
            | Class::Wall
            | Class::Troupe
            | Class::Ooze => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TroupeType {
    Spade,
    Club,
    Diamond,
    Heart,
}

#[derive(Debug)]
pub struct CardDef {
    pub id: CardId,
    pub class: Class,
    pub faces: EnumMap<FaceKey, FaceDef>,
    pub is_back_start: bool,
    pub troupe_type: Option<TroupeType>,
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

#[derive(strum_macros::Display, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SkipActionReason {
    Web,
    Venom,
    NoOption,
    Choice,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    // Control flow
    PickRow(usize, usize, CardPtr), // row_idx, card_idx, card_ptr
    SkipTurn(CardPtr),
    SkipHit(HitType),
    BottomCard,

    SkipAction(CardPtr, WrappedAction, SkipActionReason),
    StartAction(CardPtr, WrappedAction),
    AttackCard(usize, CardPtr, HitType),
    Damage(usize, CardPtr, HitType, FaceKey),
    WhiffHit(usize, CardPtr, HitType),

    // Special Attacks
    Death,
    Void(usize, CardPtr, FaceKey),
    Inspire(usize, CardPtr),
    Hypnosis(usize, CardPtr),

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
    Rat(usize, CardPtr),

    // Reactions
    Block(usize, CardPtr, Option<SelfAction>, FaceKey),
    Dodge(usize, CardPtr, Option<SelfAction>, FaceKey),
    OnHurt(usize, CardPtr),

    // Other
    PayRowConditionCosts(ConditionCostType, PayCostArrType),
    UseCardModifiers(ModifierArrType, ModifierAmount, WrappedAction),
    Maneuver(usize, CardPtr),
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
