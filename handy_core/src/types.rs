use super::card_defs::CARDS;
use bitflags::bitflags;
use colored::Colorize;
use enum_map::{Enum, EnumMap};
use std::cmp::{Ord, PartialOrd};
use std::fmt;
use strum_macros;

pub type CardId = u8;
pub type FaceValue = f32;
pub type ConditionCountType = u8;
pub type EnergyId = usize;
pub type EnergyIds = Vec<EnergyId>;

pub type VecPile = Vec<CardPtr>;
pub type BoxSlicePile = Box<[CardPtr]>;

// pub type Pile = VecPile;
pub type Pile = BoxSlicePile;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Health {
    Full,
    Half,
    Empty,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WrappedAction {
    pub action: Action,
    pub target: Target,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SelfAction {
    Rotate,
    Flip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Energy(ConditionCountType),
    Rage(ConditionCountType),
    ExhaustedAllies(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Features: u8 {
        const NoFeature = 0b00000000;
        const Weight    = 0b00000001;
        const Trap      = 0b00000010;
        const Web       = 0b00000100;
        const Venom     = 0b00001000;
        const Energy    = 0b00010000;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Row {
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
    // Should be about ~2 for the main side of a card
    pub value: FaceValue,
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
    Paladin,
    Huntress,
    Pyro,
    Werewolf,
    Beastmaster,
    Ogre,
    Vampire,
    Spider,
    Demon,
    Verdancy,
}

#[derive(Debug)]
pub struct CardDef {
    pub id: CardId,
    pub class: Class,
    pub faces: EnumMap<FaceKey, FaceDef>,
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CardPtrAsId {
    pub card_id: CardId,
    pub key: FaceKey,
}

impl CardPtrAsId {
    pub fn new_from_id(id: usize, key: FaceKey) -> CardPtrAsId {
        CardPtrAsId {
            card_id: id as u8,
            key,
        }
    }

    pub fn get_card_id(&self) -> CardId {
        self.card_id
    }

    pub fn get_card_face(&self) -> FaceKey {
        self.key
    }

    pub fn get_card_def(&self) -> &CardDef {
        &CARDS.get_card(self.card_id as usize)
    }

    pub fn get_active_face(&self) -> &FaceDef {
        &CARDS.get_card(self.card_id as usize).faces[self.key]
    }
}

impl fmt::Debug for CardPtrAsId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{}{}", self.get_card_id(), self.key);
        let colored = match self.get_active_face().allegiance {
            Allegiance::Hero => string.blue(),
            Allegiance::Baddie => string.red(),
            Allegiance::Werewolf => string.yellow(),
        };
        write!(fmt, "{}", colored)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CardPtrAsRef {
    pub card_def: &'static CardDef,
    pub key: FaceKey,
}

impl CardPtrAsRef {
    pub fn new_from_id(id: usize, key: FaceKey) -> CardPtrAsRef {
        CardPtrAsRef {
            card_def: CARDS.get_card(id),
            key,
        }
    }

    pub fn get_card_id(&self) -> CardId {
        self.card_def.id
    }

    pub fn get_card_face(&self) -> FaceKey {
        self.key
    }

    pub fn get_card_def(&self) -> &CardDef {
        &self.card_def
    }

    pub fn get_active_face(&self) -> &FaceDef {
        &self.card_def.faces[self.key]
    }
}

impl fmt::Debug for CardPtrAsRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{}{}", self.get_card_id(), self.key);
        let colored = match self.get_active_face().allegiance {
            Allegiance::Hero => string.blue(),
            Allegiance::Baddie => string.red(),
            Allegiance::Werewolf => string.yellow(),
        };
        write!(fmt, "{}", colored)
    }
}

pub type CardPtr = CardPtrAsId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HitType {
    Hit,
    Arrow,
    Trap,
    Claw,
    Ablaze,
    Fireball,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    // Control flow
    PickRow(usize, CardPtr),
    SkipTurn,
    BottomCard(CardPtr),

    // AdjustCard TODO
    SkipAction(CardPtr, WrappedAction),
    // Reported before the event
    StartAction(CardPtr, WrappedAction),
    AttackCard(CardPtr, WrappedAction),
    UseActionAssist(usize, CardPtr, usize),

    // Attacks
    Damage(usize, CardPtr, HitType, FaceKey),

    // Special Attacks
    Death,
    Void(usize, CardPtr, FaceKey),
    Inspire(usize, CardPtr),

    // Moves
    Quicken(usize, CardPtr, usize),
    Delay(usize, CardPtr, usize),
    Pull(usize, CardPtr),
    Push(usize, CardPtr),

    Teleport(usize, CardPtr, usize, CardPtr),

    // Reported after the event
    Mandatory(CardPtr, SelfAction),

    // Targets
    FireballTarget(usize, CardPtr),
    Ablaze(usize, CardPtr, usize, CardPtr),
    ReactAssistUsed(usize, CardPtr, SelfAction),

    // Heals
    Heal(usize, CardPtr),
    Revive(usize, CardPtr),

    // Reactions
    Block(usize, CardPtr, Option<SelfAction>),
    Dodge(usize, CardPtr, Option<SelfAction>),
    OnHurt(usize, CardPtr),

    // Other
    PayEnergy(usize, CardPtr),
    Manouver(usize, CardPtr),
    Swarm(usize, CardPtr),
}
