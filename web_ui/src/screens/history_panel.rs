use crate::components::*;
use crate::contexts::*;
use crate::types::*;
use closure::closure;
use leptos::ev::scroll;
use leptos::html::Div;
use leptos::*;
use leptos_use::*;

const BACK_SECTION_HEIGHT: WindowUnit = 40.0;

#[component]
pub fn HistoryFrame(cx: Scope, frame: GameFrame) -> impl IntoView {
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
                <FrameSpan frame=frame />
            </div>
        </div>
    }
}

#[component]
pub fn HistoryPanel<F>(
    cx: Scope,
    game_history_getter: Signal<GameHistory>,
    do_undo: F,
    width: WindowUnit,
    height: WindowUnit,
) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
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
            // History Panel
            style:width={move || wrap_px(placer_getter.get().scale(width))}
            style:height={move || wrap_px(placer_getter.get().scale(height))}
        >
            <div
                style:width="100%"
                style:display="flex"
                style:flex-direction="row"
                style:justify-content="center"
                style:align-content="center"
                style:margin-top={move || wrap_px(placer_getter.get().scale(3.0))}
            >
                <Button
                    width=(width - 4.0)
                    height=24.0
                    background=Signal::derive(cx, || UNDO_BUTTON_COLOUR.to_owned())
                    disabled=Signal::derive(cx, move || {
                        game_history_getter.with(|history| history.all_frames.len() <= 1)
                    })
                    on:click=move |_| do_undo()
                >
                    Undo (U)
                </Button>
            </div>

            <div
                style:width="100%"
                style:height={move || wrap_px(placer_getter.get().scale(2.0))}
                style:border-bottom="solid"
                style:border-width={move || wrap_px(placer_getter.get().scale(1.0))}
            />

            <div
                class="select-text"
                node_ref=scroll_el
                style:overflow="auto"
                style:height={move || wrap_px(placer_getter.get().scale(height - BACK_SECTION_HEIGHT))}
            >
                // History Slider
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
