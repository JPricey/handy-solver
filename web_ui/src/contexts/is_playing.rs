use leptos::*;

pub struct IsPlaying {
    pub is_playing: bool,
}

impl IsPlaying {
    pub fn new(is_playing: bool) -> Self {
        Self { is_playing }
    }
}

pub fn provide_is_playing(cx: Scope) {
    let is_playing_signal = create_rw_signal(cx, IsPlaying::new(false));

    provide_context(cx, is_playing_signal);
}

pub fn use_is_playing(cx: Scope) -> RwSignal<IsPlaying> {
    use_context::<RwSignal<IsPlaying>>(cx).unwrap()
}
