use crate::types::*;
use handy_core::game::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IconType {
    Unknown,
    Hit,
    Delay,
    Quicken,
    Arrow,
    Ablaze,
    Fireball,
}

pub fn action_to_icon_type_and_range(action: Action) -> (IconType, Option<Range>) {
    match action {
        Action::Hit(range) => (IconType::Hit, Some(range)),
        Action::Delay(range) => (IconType::Delay, Some(Range::Int(range))),
        Action::Quicken(range) => (IconType::Quicken, Some(Range::Int(range))),
        Action::Arrow => (IconType::Arrow, None),
        Action::Ablaze => (IconType::Ablaze, None),
        Action::Fireball => (IconType::Fireball, None),
        _ => (IconType::Unknown, None),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IconSpec {
    pub filename: &'static str,
    pub native_size: WindowSize,
    pub background_point: WindowSize,
    pub background_rad: WindowUnit,
    // pub number_point;
}

impl IconType {
    pub fn value(&self) -> IconSpec {
        match *self {
            IconType::Unknown => IconSpec {
                filename: "unknown.svg",
                native_size: (8.254282, 8.6261272),
                background_point: (-84.145866 + 88.273007, 76.085236 - 71.58625),
                background_rad: 4.127141,
            },
            IconType::Hit => IconSpec {
                filename: "hit.svg",
                native_size: (8.254282, 8.6261272),
                background_point: (-84.145866 + 88.273007, 76.085236 - 71.58625),
                background_rad: 4.127141,
            },
            IconType::Delay => IconSpec {
                filename: "delay.svg",
                native_size: (9.851387, 10.11192),
                background_point: (88.479691 - 83.168211, 96.920692 - 91.589043),
                background_rad: 4.5399084,
            },
            IconType::Quicken => IconSpec {
                filename: "quicken.svg",
                native_size: (9.0798168, 9.4093494),
                background_point: (169.48396 - 164.94405, 198.15073 - 193.28129),
                background_rad: 4.5399084,
            },
            IconType::Arrow => IconSpec {
                filename: "arrow.svg",
                native_size: (8.5797405, 8.254282),
                background_point: (-6.1495671 + 10.276708, -2.245064 + 6.372205),
                background_rad: 4.127141,
            },
            IconType::Ablaze => IconSpec {
                filename: "ablaze.svg",
                native_size: (8.259758, 8.2717972),
                background_point: (353.959058783 + 119.37744, 48.1475468383 - 208.18399),
                background_rad: 4.1271409694,
            },
            IconType::Fireball => IconSpec {
                filename: "fireball.svg",
                native_size: (8.254282, 8.254282),
                background_point: (353.959058783 + 39.651483, 48.1475468383 - 124.33878),
                background_rad: 4.1271409694,
            },
        }
    }
}
