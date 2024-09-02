use crate::components::*;
use crate::contexts::*;
use crate::types::*;
use closure::closure;
use handy_core::game::Pile;
use leptos::ev::scroll;
use leptos::html::Div;
use leptos::*;
use leptos_use::{use_event_listener};

#[component]
pub fn NewActivationSpan(pile: Pile) -> impl IntoView {
    view! {
        <span
            style:display="flex"
        >
            <span
                style:flex="1"
            >
                <CardIdPill card_ptr=pile[0].clone() />
                Go
            </span>

            <PileSpan pile=pile.clone() />
        </span>

    }
}

#[component]
pub fn HistoryItemWrapper(children: Children) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>().unwrap();
    view! {
        <div
            style:width="100%"
            style:border-bottom="solid"
            style:border-width={move || wrap_px(placer_getter.get().scale(1.0))}
            style:overflow="hidden"
        >
            <div
                style:width="98%"
                style:margin-left="1%"
                style:margin-right="1%"
            >
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn HistoryFrame(frame: GameFrame) -> impl IntoView {
    let events_to_render = frame.events_since_last_fame_this_activation;
    if events_to_render.is_empty() {
        return view! {
            <HistoryItemWrapper>
                <NewActivationSpan pile=frame.current_pile.clone() />
            </HistoryItemWrapper>
        };
    } else {
        return events_to_render
            .into_iter()
            .map(|event| {
                view! {
                    <HistoryItemWrapper>
                        <EventSpan event=event.clone() />
                    </HistoryItemWrapper>
                }
            })
            .collect_view();
    }
}

#[component]
pub fn HistoryPanel(game_history_getter: Signal<GameHistory>, height: WindowUnit) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>().unwrap();
    let (is_bottom_locked, set_bottom_locked) = create_signal(true);
    let (did_scroll_after_new_items, set_did_scroll_after_new_items) = create_signal(true);
    let scroll_el = create_node_ref::<Div>();

    create_effect(move |_| {
        game_history_getter.track();
        set_did_scroll_after_new_items.set(true);

        let Some(el) = scroll_el.get() else {
            return;
        };
        if is_bottom_locked.get_untracked() {
            let scroll_height = el.scroll_height();
            let height = el.client_height();
            let top = scroll_height - height;
            el.set_scroll_top(top);
        }
    });

    let _ = use_event_listener( scroll_el.clone(), scroll, move |_| {
        let Some(el) = scroll_el.get_untracked() else {
            return;
        };

        if did_scroll_after_new_items.get_untracked() {
            set_did_scroll_after_new_items.set(false);
            if is_bottom_locked.get_untracked() {
                let scroll_height = el.scroll_height();
                let height = el.client_height();
                let top = scroll_height - height;
                el.set_scroll_top(top);
            }
            return;
        } else {
            let scroll_height = el.scroll_height();
            let height = el.client_height();
            let top = el.scroll_top();
            let delta = scroll_height - top - height;

            set_bottom_locked.set(delta <= 0);
        }
    });

    view! {
        <div
            style:width="100%"
        >
            <div
                class="select-text"
                node_ref=scroll_el
                style:overflow="auto"
                style:height={move || wrap_px(placer_getter.get().scale(height))}
            >
                <For each=move || game_history_getter.get().all_frames
                    key=move |e| e.clone()
                    children=closure!(| frame| {
                        view! {
                            <HistoryFrame frame=frame />
                        }
                    })
                />
            </div>
        </div>
    }
}
