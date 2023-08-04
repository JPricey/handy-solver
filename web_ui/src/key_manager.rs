use leptos::leptos_dom::helpers::window_event_listener;
use leptos::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Control,
    Shift,
    CapsLock,
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyManagerState {
    keys_is_pressed: HashSet<Key>,
}

impl KeyManagerState {
    fn new() -> KeyManagerState {
        KeyManagerState {
            keys_is_pressed: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.keys_is_pressed.clear()
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.keys_is_pressed.contains(&key)
    }
}

fn try_map_code_to_key(code: &str) -> Option<Key> {
    match code {
        "CapsLock" => Some(Key::CapsLock),
        "ShiftLeft" => Some(Key::Shift),
        "ControlLeft" => Some(Key::Control),
        "Digit0" => Some(Key::N0),
        "Digit1" => Some(Key::N1),
        "Digit2" => Some(Key::N2),
        "Digit3" => Some(Key::N3),
        "Digit4" => Some(Key::N4),
        "Digit5" => Some(Key::N5),
        "Digit6" => Some(Key::N6),
        "Digit7" => Some(Key::N7),
        "Digit8" => Some(Key::N8),
        "Digit9" => Some(Key::N9),
        _ => {
            // log!("Unknown Code {code:?}");
            None
        }
    }
}

pub fn register_key_manager(cx: Scope) -> impl IntoView {
    let (key_manager_getter, key_manager_setter) = create_signal(cx, KeyManagerState::new());
    provide_context(cx, key_manager_getter);

    window_event_listener(ev::keydown, move |ev| {
        let code = ev.code();
        if let Some(key) = try_map_code_to_key(&code) {
            key_manager_setter.update(|manager| {
                manager.keys_is_pressed.insert(key);
            });
        }
    });

    window_event_listener(ev::keyup, move |ev| {
        let code = ev.code();
        if let Some(key) = try_map_code_to_key(&code) {
            key_manager_setter.update(|manager| {
                manager.keys_is_pressed.remove(&key);
            });
        }
    });

    window_event_listener(ev::blur, move |_| {
        key_manager_setter.update(|manager| manager.clear())
    });

    window_event_listener(ev::focus, move |_| {
        key_manager_setter.update(|manager| manager.clear())
    });

    window_event_listener(ev::focusout, move |_| {
        key_manager_setter.update(|manager| manager.clear())
    });

    window_event_listener(ev::focusin, move |_| {
        key_manager_setter.update(|manager| manager.clear())
    });
}

pub fn get_key_manager_getter(cx: Scope) -> ReadSignal<KeyManagerState> {
    let key_manager_getter = use_context::<ReadSignal<KeyManagerState>>(cx).unwrap();
    key_manager_getter
}
