use leptos::ev::{mouseenter, mouseleave};
use leptos::*;
use leptos_use::core::ElementMaybeSignal;
use leptos_use::use_event_listener;
use web_sys::*;

type IdType = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MaxId {
    id: IdType,
}

impl MaxId {
    fn new(id: IdType) -> Self {
        Self { id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CurrentId {
    id: IdType,
}

impl CurrentId {
    fn new(id: IdType) -> Self {
        Self { id }
    }
}

pub fn provide_single_hover_context() {
    let selected_id_signal = create_rw_signal(CurrentId::new(0));
    let next_id_signal = create_rw_signal(MaxId::new(1));

    provide_context(selected_id_signal);
    provide_context(next_id_signal);
}

pub fn use_single_element_hover<El, T>(el: El) -> Signal<bool>
where
    El: Clone,
    El: Into<ElementMaybeSignal<T, EventTarget>>,
    T: Into<EventTarget> + Clone + 'static,
{
    let next_id_signal = use_context::<RwSignal<MaxId>>().unwrap();
    let selected_id_signal = use_context::<RwSignal<CurrentId>>().unwrap();

    let unique_id = next_id_signal.get_untracked();
    next_id_signal.set(MaxId::new(unique_id.id + 1));

    let (is_hovered, set_hovered) = create_signal(false);

    let _ = use_event_listener(el.clone(), mouseenter, move |_| {
        selected_id_signal.set(CurrentId::new(unique_id.id));
        set_hovered.set(true);
    });

    let _ = use_event_listener(el, mouseleave, move |_| {
        set_hovered.set(false);
    });

    return Signal::derive(move || {
        return is_hovered.get() && unique_id.id == selected_id_signal.get().id;
    })
    .into();
}
