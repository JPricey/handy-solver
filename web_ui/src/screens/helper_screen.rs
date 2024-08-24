use crate::components::*;
use crate::contexts::*;
use handy_core::game::end_game::GameEndCheckType;
use leptos::*;

use crate::types::WindowUnit;

const H1_FONT_SIZE: WindowUnit = 30.0;
const H2_FONT_SIZE: WindowUnit = 18.0;

#[component]
fn ShortcutRow(cx: Scope, shortcut: String, text: String) -> impl IntoView {
    view! { cx,
        <div>
            <b>{shortcut}:</b>{format!(" {text}")}
        </div>
    }
}

#[component]
pub fn HelperScreen<F, G, H>(
    cx: Scope,
    is_showing_settings_setter: WriteSignal<bool>,
    should_show_new_match: bool,
    new_match_fn: F,
    back_to_menu_fn: G,
    replay_fn: H,
) -> impl IntoView
where
    F: Fn() + 'static,
    G: Fn() + 'static,
    H: Fn() + 'static,
{
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let options = use_options(cx);

    view! { cx,
        <div
            style:position="relative"
            style:width="100%"
            style:height="100%"
            style:display="flex"
            style:justify-content="center"
            style:align-items="center"
        >
            <div
                style:position="absolute"
                style:background="black"
                style:opacity="40%"
                style:width="100%"
                style:height="100%"
                on:click=move |_| is_showing_settings_setter.set(false)
            >
            </div>
            <div
                style:position="absolute"
                style:opacity="100%"
                style:background="white"
                style:width="50%"
                style:height="70%"
                style:border-radius={move || wrap_px(placer_getter.get().scale(10.0))}
            >
                <div
                    style:margin={move || wrap_px(placer_getter.get().scale(20.0))}
                    style:display="flex"
                    style:flex-direction="column"
                    style:position="absolute"
                    style:top="0"
                    style:left="0"
                    style:right="0"
                    style:bottom="0"
                >
                    <div
                        style:width="100%"
                        style:font-size={move || wrap_px(placer_getter.get().scale(H1_FONT_SIZE))}
                        style:display="flex"
                        style:justify-content="center"
                    >
                        Help & Shortcuts
                    </div>

                    <div
                        style:height={move || wrap_px(placer_getter.get().scale(10.0))}
                    />

                    <div>
                        <div
                            style:font-size={move || wrap_px(placer_getter.get().scale(H2_FONT_SIZE))}
                        >
                            Gameplay Shortcuts
                        </div>
                        <ShortcutRow
                            shortcut="1-9".to_owned()
                            text="Execute an action that targets a card in the stack".to_owned()
                        />
                        <ShortcutRow
                            shortcut="0".to_owned()
                            text="Skip an action or turn".to_owned()
                        />
                        <ShortcutRow
                            shortcut="A, B, C, D".to_owned()
                            text="Execute an action that rotates / flips a card".to_owned()
                        />
                        <ShortcutRow
                            shortcut="0".to_owned()
                            text="Skip an action or turn".to_owned()
                        />
                        <ShortcutRow
                            shortcut="Enter".to_owned()
                            text="Execute an action that has already been targetted".to_owned()
                        />

                        <div
                            style:height={move || wrap_px(placer_getter.get().scale(10.0))}
                        />
                        <div
                            style:font-size={move || wrap_px(placer_getter.get().scale(H2_FONT_SIZE))}
                        >
                            Menu Shortcuts
                        </div>
                        <ShortcutRow
                            shortcut="X, ?, /".to_owned()
                            text="Show / Hide this screen".to_owned()
                        />
                        <ShortcutRow
                            shortcut="E".to_owned()
                            text="Show / Hide the engine".to_owned()
                        />
                        <ShortcutRow
                            shortcut="U, ←".to_owned()
                            text="Undo the last action".to_owned()
                        />
                        <ShortcutRow
                            shortcut="→".to_owned()
                            text="Execute an only-move (when auto-move is disabled)".to_owned()
                        />
                    </div>

                    <div
                        style:height={move || wrap_px(placer_getter.get().scale(10.0))}
                    />

                    <div>
                        <div
                            style:font-size={move || wrap_px(placer_getter.get().scale(H2_FONT_SIZE))}
                        >
                            Settings
                        </div>

                        <div>
                            <input
                                type="checkbox"
                                checked = move || options.get().game_end_check_type == GameEndCheckType::PerHeroClass
                                on:click=move |_| {
                                    options.update(|opts| opts.game_end_check_type = flip_end_game_type(opts.game_end_check_type));
                                }
                            />
                            <span>{"Enable loss on any exhausted hero class ("}</span><b>C</b>{")"}
                        </div>

                        <div>
                            <input
                                type="checkbox"
                                checked = move || options.get().is_pick_only_moves
                                on:click=move |_| {
                                    options.update(|opts| opts.is_pick_only_moves = !opts.is_pick_only_moves);
                                }
                            />
                            <span>{"Enable Auto Only-Moves ("}</span><b>O</b>{")"}
                        </div>

                        <div>
                            <input
                                type="checkbox"
                                checked = move || options.get().is_showing_settings_bar
                                on:click=move |_| {
                                    options.update(|opts| opts.is_showing_settings_bar = !opts.is_showing_settings_bar);
                                }
                            />
                            <span>{"Enable Settings Bar Visibility ("}</span><b>H</b>{")"}
                        </div>
                    </div>

                    <div
                        style:flex-grow=1
                    />

                    <div
                        style:width="100%"
                        style:display="flex"
                        style:justify-content="space-around"
                    >
                        { if should_show_new_match { Some(view! { cx,
                            <Button
                                background=Signal::derive(cx, || BUTTON_SELECTED_COLOUR.to_string())
                                width=100.0
                                height=30.0
                                on:click=move |_| {
                                    new_match_fn();
                                    is_showing_settings_setter.set(false);
                                }
                            >
                                New Match
                            </Button>
                            })} else {
                                None
                            }
                        }

                        <Button
                            background=Signal::derive(cx, || BUTTON_SELECTED_COLOUR.to_string())
                            width=100.0
                            height=30.0
                            on:click=move |_| {
                                replay_fn();
                                is_showing_settings_setter.set(false);
                            }
                        >
                            Replay
                        </Button>

                        <Button
                            background=Signal::derive(cx, || BUTTON_NON_SELECTED_COLOUR.to_string())
                            width=100.0
                            height=30.0
                            on:click=move |_| {
                                back_to_menu_fn();
                                is_showing_settings_setter.set(false);
                            }
                        >
                            Back to Menu
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    }
}
