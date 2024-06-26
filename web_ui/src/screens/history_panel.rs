use crate::components::*;
use crate::contexts::*;
use crate::types::*;
use closure::closure;
use handy_core::game::Pile;
use leptos::ev::scroll;
use leptos::html::Div;
use leptos::*;
use leptos_use::*;

#[component]
pub fn NewActivationSpan(cx: Scope, pile: Pile) -> impl IntoView {
    view! {cx,
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
pub fn HistoryItemWrapper(cx: Scope, children: Children) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    view! { cx,
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
                {children(cx)}
            </div>
        </div>
    }
}

#[component]
pub fn HistoryFrame(cx: Scope, frame: GameFrame) -> impl IntoView {
    let events_to_render = frame.events_since_last_fame_this_activation;
    if events_to_render.is_empty() {
        return view! { cx,
            <HistoryItemWrapper>
                <NewActivationSpan pile=frame.current_pile.clone() />
            </HistoryItemWrapper>
        };
    } else {
        return events_to_render
            .into_iter()
            .map(|event| {
                view! { cx,
                    <HistoryItemWrapper>
                        <EventSpan event=event.clone() />
                    </HistoryItemWrapper>
                }
            })
            .collect_view(cx);
    }
}

#[component]
pub fn HistoryPanel(
    cx: Scope,
    game_history_getter: Signal<GameHistory>,
    height: WindowUnit,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let (is_bottom_locked, set_bottom_locked) = create_signal(cx, true);
    let (did_scroll_after_new_items, set_did_scroll_after_new_items) = create_signal(cx, true);
    let scroll_el = create_node_ref::<Div>(cx);

    create_effect(cx, move |_| {
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

    let _ = use_event_listener(cx, scroll_el.clone(), scroll, move |_| {
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

    view! { cx,
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
                    view=closure!(|cx, frame| {
                        view! { cx,
                            <HistoryFrame frame=frame />
                        }
                    })
                />
            </div>
        </div>
    }
}
