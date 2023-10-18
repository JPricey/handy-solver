use crate::types::*;

pub const DEFAULT_FONT_SIZE: WindowUnit = 14.0;

pub const BUTTON_SELECTED_COLOUR: &str = "#7eb070";
pub const BUTTON_NON_SELECTED_COLOUR: &str = "#e9f497";

pub const UNDO_BUTTON_COLOUR: &str = "#e6a732";
pub const BUTTON_BORDER_RADIUS_PX: WindowUnit = 2.0;
pub const MENU_BUTTON_COLOUR: &str = "#c9ced6";


pub fn wrap_px(unit: WindowUnit) -> String {
    format!("{unit}px")
}

pub fn wrap_pct(unit: WindowUnit) -> String {
    format!("{unit}%")
}
