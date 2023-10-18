use crate::components::*;
use crate::contexts::*;
use leptos::*;

use crate::types::WindowUnit;

const H1_FONT_SIZE: WindowUnit = 30.0;
const H2_FONT_SIZE: WindowUnit = 18.0;

#[component]
pub fn HelperScreen(cx: Scope, is_showing_settings_setter: WriteSignal<bool>) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let is_playing = use_is_playing(cx);

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
                        <div>
                            Reveal a card in the pile (Hold Ctrl + Mouseover)
                        </div>
                        <div>
                            Reveal a card backside in the pile (Hold Shift + Mouseover)
                        </div>
                        <div>
                            Execute an action that targets a card in the stack (1-9)
                        </div>
                        <div>
                            Execute an action that rotates / flips a card (A, B, C, D)
                        </div>
                        <div>
                            Execute an action with no targets (Enter)
                        </div>
                        <div>
                            Skip an action or turn (0)
                        </div>


                        <div
                            style:height={move || wrap_px(placer_getter.get().scale(10.0))}
                        />
                        <div
                            style:font-size={move || wrap_px(placer_getter.get().scale(H2_FONT_SIZE))}
                        >
                            Menu Shortcuts
                        </div>
                        <div>
                            Show / Hide this screen (?, /, X)
                        </div>
                        <div>
                            Show / Hide the engine (E)
                        </div>
                        <div>
                            {format!("Undo the last action (U, ←)")}
                        </div>
                        <div>
                            {format!("Execute an only-move (→)")}
                        </div>
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
                            Toggle Settings Bar Visibility (H)
                        </div>
                        <div>
                            Toggle Auto Only-Moves (O)
                        </div>
                    </div>

                    <div
                        style:flex-grow=1
                    />

                    <div
                        style:width="100%"
                        style:display="flex"
                        style:justify-content="center"
                    >
                        <Button
                            background=Signal::derive(cx, || BUTTON_NON_SELECTED_COLOUR.to_string())
                            width=100.0
                            height=30.0
                            on:click=move |_| {
                                is_playing.update(|s| s.is_playing = false)
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
