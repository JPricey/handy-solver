use glam::{DQuat, EulerRot};
use handy_core::game::*;
use leptos::*;
use std::f64::consts::PI;
use std::ops::{Add, Mul, Sub};
use std::time::Duration;

pub type WindowUnit = f64;
pub type WindowSize = (WindowUnit, WindowUnit);

pub static GOLDEN_MAX_WIDTH: WindowUnit = 1920.0;
pub static GOLDEN_MIN_WIDTH: WindowUnit = 1280.0;
pub static GOLDEN_HEIGHT: WindowUnit = 800.0;

pub fn quat_for_face(face_key: FaceKey) -> DQuat {
    match face_key {
        FaceKey::A => DQuat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
        FaceKey::B => DQuat::from_euler(EulerRot::XYZ, 0.0, 0.0, PI),
        FaceKey::C => DQuat::from_euler(EulerRot::XYZ, 0.0, PI, 0.0),
        FaceKey::D => DQuat::from_euler(EulerRot::XYZ, 0.0, PI, PI),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderCard {
    pub card_id: CardId,
    pub active_face: RwSignal<FaceKey>,

    pub point: RwSignal<(Point2D, Duration)>,
    pub animated_point: Signal<Point2D>,

    pub quat: RwSignal<(DQuat, Duration)>,
    pub animated_quat: Signal<DQuat>,

    pub position_in_pile: RwSignal<(WindowUnit, Duration)>,
    pub animated_position_in_pile: Signal<WindowUnit>,

    pub is_important: RwSignal<bool>,
    pub z_index: Signal<i32>,

    pub is_clickable: RwSignal<bool>,
}

pub fn scalar_mult(point: WindowSize, scale: WindowUnit) -> WindowSize {
    (point.0 * scale, point.1 * scale)
}

pub fn point_sub(a: WindowSize, b: WindowSize) -> WindowSize {
    (a.0 - b.0, a.1 - b.1)
}

pub fn point_add(a: WindowSize, b: WindowSize) -> WindowSize {
    (a.0 + b.0, a.1 + b.1)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point2D {
    pub x: WindowUnit,
    pub y: WindowUnit,
}

impl Point2D {
    pub fn default() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn new(x: WindowUnit, y: WindowUnit) -> Self {
        Self { x, y }
    }

    pub fn length_squared(self) -> WindowUnit {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> WindowUnit {
        self.length_squared().sqrt()
    }
}

impl Add<Point2D> for Point2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub<Point2D> for Point2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Point2D {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MoveOption {
    pub events: Vec<Event>,
    pub next_pile: Pile,
}

impl MoveOption {
    pub fn new(events: Vec<Event>, next_pile: Pile) -> Self {
        Self { events, next_pile }
    }

    /// Returns the event that will be used to decide affordances for this action
    pub fn get_primary_event(&self) -> &Event {
        self.events.last().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameFrame {
    pub root_pile: Pile,
    pub current_pile: Pile,
    pub event_history: Vec<Event>,
    pub available_moves: Vec<MoveOption>,
    pub resolution: WinType,
    pub is_definite_win: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameHistory {
    pub all_frames: Vec<GameFrame>,
}
