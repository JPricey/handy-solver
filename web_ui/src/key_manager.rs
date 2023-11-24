use leptos::leptos_dom::helpers::window_event_listener;
use leptos::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    ControlLeft,
    ShiftLeft,
    ControlRight,
    ShiftRight,
    CapsLock,
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

    pub fn is_control(&self) -> bool {
        self.keys_is_pressed.contains(&Key::ControlLeft)
            || self.keys_is_pressed.contains(&Key::ControlRight)
            || self.keys_is_pressed.contains(&Key::CapsLock)
    }

    pub fn is_shift(&self) -> bool {
        self.keys_is_pressed.contains(&Key::ShiftLeft)
            || self.keys_is_pressed.contains(&Key::ShiftRight)
    }
}

fn try_map_code_to_key(code: &str) -> Option<Key> {
    match code {
        "CapsLock" => Some(Key::CapsLock),
        "ShiftLeft" => Some(Key::ShiftLeft),
        "ControlLeft" => Some(Key::ControlLeft),
        "ShiftRight" => Some(Key::ShiftRight),
        "ControlRight" => Some(Key::ControlRight),
        _ => None,
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
