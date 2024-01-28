use crate::game_card::*;
use crate::types::*;
use handy_core::game::*;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

pub const FONT_BASE_SIZE: WindowUnit = 14.0;

pub const CARD_WINDOW_BUFFER_PX: WindowUnit = 20.0;
pub const ORACLE_ZONE_WIDTH_PX: WindowUnit = 220.0;
pub const ORACLE_ZONE_HEIGHT_PX: WindowUnit = 160.0;

pub const CHOICE_BUTTON_WIDTH_PX: WindowUnit = 120.0;
pub const CHOICE_BUTTON_HEIGHT_PX: WindowUnit = 80.0;
pub const END_WINDOW_WIDTH_PX: WindowUnit = 400.0;
pub const END_WINDOW_HEIGHT_PX: WindowUnit = 300.0;
pub const SELECTED_Y_DELTA_PX: WindowUnit = -40.0;

lazy_static! {
    pub static ref HISTORY_ZONE_WIDTH_PX: WindowUnit = 368.0;
    pub static ref CARD_ZONE_WIDTH_PX: WindowUnit = GOLDEN_WIDTH - *HISTORY_ZONE_WIDTH_PX;
    pub static ref CARD_ZONE_HEIGHT_PX: WindowUnit = GOLDEN_HEIGHT;
    pub static ref CARD_ZONE_BUFFER_WIDTH: WindowUnit = 24.0;
    pub static ref TOP_CARD_Y_OFFSET_PX: WindowUnit =
        GOLDEN_HEIGHT - RENDER_CARD_SIZE.1 - CARD_WINDOW_BUFFER_PX;
    pub static ref BOTTOM_CARD_Y_OFFSET_PX: WindowUnit = *TOP_CARD_Y_OFFSET_PX - 220.0;
    pub static ref ROW_OPTION_WIDTH_PX: WindowUnit = RENDER_CARD_SIZE.0;
    pub static ref ROW_OPTION_HEIGHT_PX: WindowUnit = RENDER_CARD_SIZE.1 * 0.105;
    pub static ref TOP_CARD_LEFT_PX: WindowUnit =
        *CARD_ZONE_WIDTH_PX - *CARD_ZONE_BUFFER_WIDTH - RENDER_CARD_SIZE.0;
    pub static ref END_WINDOW_LEFT_PX: WindowUnit =
        (*CARD_ZONE_WIDTH_PX - END_WINDOW_WIDTH_PX) / 2.0;
    pub static ref END_WINDOW_TOP_PX: WindowUnit =
        (*CARD_ZONE_HEIGHT_PX - END_WINDOW_HEIGHT_PX) / 2.0;
    pub static ref OPTIONS_HEADER_ZONE_HEIGHT_PCT: WindowUnit =
        (CHOICE_BUTTON_HEIGHT_PX + 10.0) * 100.0 / GOLDEN_HEIGHT;
}

pub type RenderCardMap = HashMap<CardId, RenderCard>;

#[derive(Clone, Debug, PartialEq)]
pub struct FallbackButton {
    pub move_option: MoveOption,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InteractionButton {
    pub move_option: MoveOption,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SkipButton {
    pub move_option: MoveOption,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderRowOption {
    pub card_index: usize,
    pub card_id: CardId,
    pub row_index: usize,
    pub move_option: MoveOption,
    pub placement_pct: WindowUnit,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DamageCardOption {
    pub card_ptr: CardPtr,
    pub move_option: MoveOption,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClickableCard {
    pub card_id: CardId,
    pub card_index: u8,
    pub reason: ClickableCardReason,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClickableCardReason {
    Move(MoveOption),
    Select,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompleteSelectionOption {
    pub selected_cards: HashSet<CardId>,
    pub move_option: MoveOption,
    pub completed_text: String,
}

impl CompleteSelectionOption {
    pub fn new(
        selected_cards: HashSet<CardId>,
        move_option: MoveOption,
        completed_text: String,
    ) -> Self {
        Self {
            selected_cards,
            move_option,
            completed_text,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InteractionOptions {
    pub pile: Pile,

    pub row_options: Vec<RenderRowOption>,
    pub interaction_buttons: Vec<InteractionButton>,
    pub valid_selection_buttons: Vec<InteractionButton>,
    // TODO: should be optional, not a vec
    pub skip_button: Vec<SkipButton>,
    pub clickable_cards: HashMap<CardId, ClickableCardReason>,
    pub damage_card_options: Vec<DamageCardOption>,

    pub selected_cards: HashSet<CardId>,
    pub selection_options: Vec<CompleteSelectionOption>,

    pub important_cards: HashSet<CardId>,

    pub hints: HashSet<String>,
}

impl InteractionOptions {
    pub fn new(pile: Pile) -> Self {
        InteractionOptions {
            pile,

            row_options: vec![],
            interaction_buttons: vec![],
            valid_selection_buttons: vec![],
            skip_button: vec![],
            clickable_cards: HashMap::new(),
            damage_card_options: vec![],

            selected_cards: HashSet::new(),
            selection_options: Vec::new(),

            important_cards: HashSet::new(),

            hints: HashSet::new(),
        }
    }

    pub fn total_buttons_available(&self) -> usize {
        self.row_options.len()
            + self.interaction_buttons.len()
            + self.valid_selection_buttons.len()
            + self.skip_button.len()
            + self.damage_card_options.len()
    }
}
