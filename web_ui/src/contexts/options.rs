use handy_core::game::end_game::GameEndCheckType;
use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    pub is_showing_settings_bar: bool,
    pub is_pick_only_moves: bool,
    pub game_end_check_type: GameEndCheckType,
}

impl Options {
    pub fn new() -> Self {
        Self {
            is_showing_settings_bar: true,
            is_pick_only_moves: true,
            game_end_check_type: GameEndCheckType::Standard,
        }
    }
}

pub fn provide_options() {
    let options_signal = create_rw_signal(Options::new());

    provide_context(options_signal);
}

pub fn use_options() -> RwSignal<Options> {
    use_context::<RwSignal<Options>>().unwrap()
}

pub fn flip_end_game_type(game_end_check_type: GameEndCheckType) -> GameEndCheckType {
    match game_end_check_type {
        GameEndCheckType::Standard => GameEndCheckType::PerHeroClass,
        GameEndCheckType::PerHeroClass => GameEndCheckType::Standard,
    }
}
