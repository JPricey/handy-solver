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
pub fn HistoryFrame<F>(cx: Scope, frame: GameFrame, on_click: F) -> impl IntoView
where
    F: Fn() + 'static,
{
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();

    view! { cx,
        <div
            style:width="100%"
            style:border-bottom="solid"
            style:border-width="1px"
            style:overflow="hidden"
            style:display="flex"
            style:flex-direction="row"
        >
            <div
                style:flex=1
            >
                <FrameSpan frame=frame />
            </div>
            <div
                height="100%"
                style:display="flex"
                style:flex-direction="column"
                style:justify-content="center"
                style:margin-right={move || wrap_px(placer_getter.get().scale(4.0))}
                style:margin-left={move || wrap_px(placer_getter.get().scale(2.0))}
            >
                <Button
                    width=14.0
                    height=14.0
                    background=Signal::derive(cx, || UNDO_BUTTON_COLOUR.to_owned())
                    on:click=move |_| { on_click() }
                    font_size=10.0
                >
                    {format!("←")}
                </Button>
            </div>
        </div>
    }
}

#[component]
pub fn HistoryPanel<F>(
    cx: Scope,
    game_history_getter: Signal<GameHistory>,
    set_history: F,
    width: WindowUnit,
    height: WindowUnit,
) -> impl IntoView
where
    F: Fn(GameHistory) + Clone + 'static,
{
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let (is_bottom_locked, set_bottom_locked) = create_signal(cx, true);
    let (did_scroll_after_new_items, set_did_scroll_after_new_items) = create_signal(cx, true);

    let history_fetcher = move || {
        let result: Vec<(usize, GameFrame)> = game_history_getter
            .get()
            .all_frames
            .into_iter()
            .enumerate()
            .collect();
        return result;
    };

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
        let Some(el) = scroll_el.get() else {
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
            style:background="pink"
            style:width={move || wrap_px(placer_getter.get().scale(width))}
            style:height={move || wrap_px(placer_getter.get().scale(height))}
        >
            <div
                style:width="100%"
                style:display="flex"
                style:flex-direction="row"
                style:justify-content="center"
                style:align-content="center"
            >
                <Button
                    width=(width - 8.0)
                    height=24.0
                    background=Signal::derive(cx, || UNDO_BUTTON_COLOUR.to_owned())
                    disabled=Signal::derive(cx, move || {
                        game_history_getter.with(|history| history.all_frames.len() <= 1)
                    })
                    on:click=closure!(clone set_history,  |_| {
                        let mut new_history = game_history_getter.get();
                        if new_history.all_frames.len() <= 1 {
                            return;
                        }

                        let mut truncate_index = 1;
                        for (i, frame) in new_history.all_frames.iter().enumerate().rev() {
                            if i == new_history.all_frames.len() - 1 {
                                continue;
                            }

                            if frame.available_moves.len() > 1 {
                                truncate_index = i + 1;
                                break;
                            }
                        }

                        new_history.all_frames.truncate(truncate_index);
                        set_history(new_history);
                    })
                >
                    Undo
                </Button>
            </div>

            <div
                style:width="100%"
                style:height={move || wrap_px(placer_getter.get().scale(2.0))}
                style:border-bottom="solid"
                style:border-width="1px"
            />

            <div
                node_ref=scroll_el
                style:overflow="auto"
                style:height={move || wrap_px(placer_getter.get().scale(height - BACK_SECTION_HEIGHT))}
            >
                // History Slider
                <For each=history_fetcher
                    key=move |e| e.0
                    view=closure!(clone set_history, |cx, element| {
                        let (index, frame) = element;
                        let on_click = closure!(clone set_history, || {
                            let mut new_history = game_history_getter.get();
                            new_history.all_frames.truncate(index + 1);
                            set_history(new_history);
                        });
                        view! { cx,
                            <HistoryFrame frame=frame on_click=on_click/>
                        }
                    })
                />
            </div>
        </div>
    }
}
