use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    pub is_showing_settings_bar: bool,
    pub is_pick_only_moves: bool,
}

impl Options {
    pub fn new() -> Self {
        Self {
            is_showing_settings_bar: true,
            is_pick_only_moves: true,
        }
    }
}

pub fn provide_options(cx: Scope) {
    let options_signal = create_rw_signal(cx, Options::new());

    provide_context(cx, options_signal);
}

pub fn use_options(cx: Scope) -> RwSignal<Options> {
    use_context::<RwSignal<Options>>(cx).unwrap()
}
