use crate::components::*;
use crate::contexts::*;
use crate::game_card::*;
use crate::game_player_state::*;
use crate::game_player_types::*;
use crate::oracle_panel::*;
use crate::screens::*;
use crate::types::*;
use closure::closure;
use handy_core::game::*;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::*;

fn get_combined_interaction_buttons(
    interaction_options: &InteractionOptions,
) -> Vec<InteractionButton> {
    let mut a = interaction_options.interaction_buttons.clone();
    let b = interaction_options.valid_selection_buttons.clone();

    a.extend(b);

    return a;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hotkey {
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
    A,
    B,
    C,
    D,
    Ent,
    Up,
    Down,
    Left,
    Right,
}

fn code_to_hotkeyable(code: &str) -> Option<Hotkey> {
    match code {
        "Digit0" => Some(Hotkey::N0),
        "Digit1" => Some(Hotkey::N1),
        "Digit2" => Some(Hotkey::N2),
        "Digit3" => Some(Hotkey::N3),
        "Digit4" => Some(Hotkey::N4),
        "Digit5" => Some(Hotkey::N5),
        "Digit6" => Some(Hotkey::N6),
        "Digit7" => Some(Hotkey::N7),
        "Digit8" => Some(Hotkey::N8),
        "Digit9" => Some(Hotkey::N9),
        "KeyA" => Some(Hotkey::A),
        "KeyB" => Some(Hotkey::B),
        "KeyC" => Some(Hotkey::C),
        "KeyD" => Some(Hotkey::D),
        "Enter" => Some(Hotkey::Ent),
        "ArrowUp" => Some(Hotkey::Up),
        "ArrowDown" => Some(Hotkey::Down),
        "ArrowLeft" => Some(Hotkey::Left),
        "ArrowRight" => Some(Hotkey::Right),
        _ => None,
    }
}

fn hotkey_to_number(hotkey: Hotkey) -> Option<usize> {
    match hotkey {
        Hotkey::N0 => Some(0),
        Hotkey::N1 => Some(1),
        Hotkey::N2 => Some(2),
        Hotkey::N3 => Some(3),
        Hotkey::N4 => Some(4),
        Hotkey::N5 => Some(5),
        Hotkey::N6 => Some(6),
        Hotkey::N7 => Some(7),
        Hotkey::N8 => Some(8),
        Hotkey::N9 => Some(9),
        _ => None,
    }
}

fn hotkey_to_face_key(hotkey: Hotkey) -> Option<FaceKey> {
    match hotkey {
        Hotkey::A => Some(FaceKey::A),
        Hotkey::B => Some(FaceKey::B),
        Hotkey::C => Some(FaceKey::C),
        Hotkey::D => Some(FaceKey::D),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionOption {
    MoveOption(MoveOption),
    CardSelection(CardId),
}

fn hotkey_to_outcome(
    hotkey: Hotkey,
    interaction_options: &InteractionOptions,
) -> Option<ActionOption> {
    if let Some(num) = hotkey_to_number(hotkey) {
        if num > 0 {
            for row in &interaction_options.row_options {
                if row.row_index + 1 == num {
                    return Some(ActionOption::MoveOption(row.move_option.clone()));
                }
            }

            if let Some(card_ptr) = interaction_options.pile.get(num - 1) {
                if let Some(reason) = interaction_options
                    .clickable_cards
                    .get(&card_ptr.get_card_id())
                {
                    match reason {
                        ClickableCardReason::Move(move_option) => {
                            return Some(ActionOption::MoveOption(move_option.clone()));
                        }
                        ClickableCardReason::Select => {
                            return Some(ActionOption::CardSelection(card_ptr.get_card_id()));
                        }
                    }
                }
            }
        }
    }

    if let Some(face_key) = hotkey_to_face_key(hotkey) {
        for option in &interaction_options.damage_card_options {
            if option.card_ptr.key == face_key {
                return Some(ActionOption::MoveOption(option.move_option.clone()));
            }
        }
    }

    if hotkey == Hotkey::N0 {
        if let Some(skip) = interaction_options.skip_button.last() {
            return Some(ActionOption::MoveOption(skip.move_option.clone()));
        }
    }

    if hotkey == Hotkey::Ent {
        if let Some(selection) = interaction_options.valid_selection_buttons.first() {
            return Some(ActionOption::MoveOption(selection.move_option.clone()));
        }

        if let Some(selection) = interaction_options.interaction_buttons.first() {
            return Some(ActionOption::MoveOption(selection.move_option.clone()));
        }
    }

    None
}

#[component]
pub fn action_button(cx: Scope, text: String, is_skip: bool) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let hotkey_text = if is_skip { "(0)" } else { "(Enter)" };

    let colour = if is_skip {
        BUTTON_NON_SELECTED_COLOUR
    } else {
        BUTTON_SELECTED_COLOUR
    };

    view! {cx,
        <Button
            width=CHOICE_BUTTON_WIDTH_PX
            height=CHOICE_BUTTON_HEIGHT_PX
            background=Signal::derive(cx, || colour.to_owned())
        >
            <div
                width="100%"
                height="100%"
                style:display="flex"
                style:justify-content="center"
                style:align-content="center"
                style:font-size={move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE))}
            >
                {text}
            </div>
            <div
                style:position="absolute"
                style:right="0%"
                style:bottom="0%"
                style:font-size={move || wrap_px(placer_getter.get().scale(8.0))}
            >
                {hotkey_text}
            </div>
        </Button>
    }
}

#[component]
pub fn GamePlayer(cx: Scope, init_pile: Pile) -> impl IntoView {
    let is_playing = use_is_playing(cx);
    let game_state = GamePlayerState::new(cx, init_pile.clone());
    let game_history_getter = game_state.game_history_getter;
    let render_card_map_getter = game_state.render_card_map_getter;

    let current_state = create_memo(cx, move |_| {
        game_history_getter.get().all_frames.last().unwrap().clone()
    });
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let interaction_getter = game_state.interaction_getter;

    let render_cards_getter = move || {
        let mut result: Vec<RenderCard> = render_card_map_getter.get().values().copied().collect();
        result.sort_by(|a, b| {
            b.animated_position_in_pile
                .get()
                .partial_cmp(&a.animated_position_in_pile.get())
                .unwrap()
        });
        result
    };

    let hint_string = move || {
        let hints = interaction_getter.get().hints;
        return hints.into_iter().collect::<Vec<_>>().join(" or ");
    };

    let end_game_text = move || {
        let cs = current_state.get();
        match cs.winner {
            Some(Allegiance::Hero) => "You Win!!",
            Some(Allegiance::Baddie) => "You Lost :(",
            _ => "Something went wrong",
        }
    };

    window_event_listener(ev::keydown, move |ev| {
        let code = ev.code();
        if let Some(hotkey) = code_to_hotkeyable(&code) {
            let hotkey_outcome = hotkey_to_outcome(hotkey, &interaction_getter.get());
            if let Some(outcome) = hotkey_outcome {
                match outcome {
                    ActionOption::MoveOption(move_option) => {
                        game_state.apply_option(&move_option);
                    }
                    ActionOption::CardSelection(card_id) => {
                        game_state.select_card(card_id);
                    }
                }
            }
        }
    });

    let init_animation = game_state.do_render_pile_update();
    game_state.maybe_schedule_next_move(init_animation);

    on_cleanup(
        cx,
        closure!(clone mut game_state, || {
            game_state.clear_animation();
        }),
    );

    view! { cx,
        <div
            // Parent container
            style:display="flex"
        >
            <div
                // Play Zone
                style:width={move || wrap_px(placer_getter.get().scale(*CARD_ZONE_WIDTH_PX))}
                style:height={move || wrap_px(placer_getter.get().scale(GOLDEN_HEIGHT))}
                style:background="rgb(248, 238, 226)"
                style:display="flex"
                style:flex-direction="column"
            >
                <div
                    // Hint Zone
                    style:width="100%"
                    style:font-size={move || wrap_px(placer_getter.get().scale(24.0))}
                    style:margin={move || wrap_px(placer_getter.get().scale(12.0))}
                    style:display="flex"
                    style:justify-content="center"
                    style:align-content="center"
                >
                    {hint_string}
                </div>

                <div
                    // Options header zone
                    style:width="100%"
                    style:display="flex"
                    style:height=wrap_pct(*OPTIONS_HEADER_ZONE_HEIGHT_PCT)
                >
                    <div
                        // Button Zone
                        style:display="flex"
                        style:justify-content="center"
                        style:flex-grow=1.0
                    >
                        <For each=move || get_combined_interaction_buttons(&interaction_getter.get()) key=|e| e.move_option.clone() view=move |cx, option| {
                            let on_click = move |_| {
                                game_state.apply_option(&option.move_option);
                            };

                            view! { cx,
                                <ActionButton
                                    text=option.text.to_owned()
                                    is_skip=false
                                    on:click=on_click
                                />
                            }
                        }/>
                        <For each=move || interaction_getter.get().skip_button key=|e| e.move_option.clone() view=move |cx, option| {
                            let on_click = move |_| {
                                game_state.apply_option(&option.move_option);
                            };

                            view! { cx,
                                <ActionButton
                                    text=option.text.to_owned()
                                    is_skip=true
                                    on:click=on_click
                                />
                            }
                        }/>
                    </div>
                    <div
                        // Card selections
                        style:display="flex"
                        style:justify-content="center"
                        style:flex-grow=1.0
                    >
                        <For each=move || interaction_getter.get().damage_card_options key=|e| e.card_ptr view=move |cx, damage_option| {
                            view! { cx,
                                <StaticGameCard
                                    card_id=damage_option.card_ptr.get_card_id()
                                    face_key=damage_option.card_ptr.get_card_face()
                                    is_clickable=true
                                    scale=1.0
                                    on:click= move |_| { game_state.apply_option(&damage_option.move_option) }
                                />
                            }
                        }/>
                    </div>
                </div>

                // Cards
                <For each=render_cards_getter key=|e| e.card_id view=move |cx, render_card| {
                    let get_row_options= move || {
                            interaction_getter.with(|interactions| {
                            interactions.row_options.iter().filter(|option| {
                                option.card_id == render_card.card_id
                            }).cloned().collect::<Vec<RenderRowOption>>()
                        })
                    };

                    view! { cx,
                        <InPlayGameCard
                            render_card={render_card}
                            on:click = move |_| {
                                let maybe_click_result = interaction_getter.get().clickable_cards.get(&render_card.card_id).cloned();
                                if let Some(click_result) = maybe_click_result {
                                    match click_result {
                                        ClickableCardReason::Move(move_option) => {
                                            game_state.apply_option(&move_option);
                                        }
                                        ClickableCardReason::Select => {
                                            game_state.select_card(render_card.card_id);
                                        }
                                    }
                                }
                            }
                        >
                            <div
                                style:width="100%"
                                style:height="100%"
                                style:transform=move || {
                                    match render_card.active_face.get() {
                                        FaceKey::A | FaceKey::C => None,
                                        FaceKey::B | FaceKey::D => Some("rotate(180deg)"),
                                    }
                                }
                            >
                                <For each=get_row_options key=|e: &RenderRowOption| (e.row_index, e.move_option.clone()) view=move |cx, row_option| {
                                    view! { cx,
                                        <div
                                            style:position="absolute"
                                            style:top={move || wrap_px(placer_getter.get().scale(RENDER_CARD_SIZE.1 * row_option.placement_pct))}
                                            style:width={move || wrap_px(placer_getter.get().scale(*ROW_OPTION_WIDTH_PX))}
                                            style:height={move || wrap_px(placer_getter.get().scale(*ROW_OPTION_HEIGHT_PX))}
                                            on:click= move |_| { game_state.apply_option(&row_option.move_option) }
                                        >
                                            <button
                                                tabindex=-1
                                                style:position="absolute"
                                                style:border="none"
                                                style:border-radius={move || wrap_px(placer_getter.get().scale(BUTTON_BORDER_RADIUS_PX))}
                                                style:background-color="white"
                                                style:cursor="pointer"
                                                style:opacity="0.4"
                                                style:width="100%"
                                                style:height="100%"
                                            />
                                            <div
                                                style:position="absolute"
                                                style:top="50%"
                                                style:left="7%"
                                                style:transform="translateY(-50%)"
                                            >
                                                {row_option.row_index + 1}
                                            </div>
                                        </div>
                                    }
                                }/>
                            </div>
                        </InPlayGameCard>
                    }
                }/>

        </div>

        <HistoryPanel
            game_history_getter=game_history_getter.into()
            set_history=move |new_history| {
                game_state.set_history(new_history.clone());
                game_state.do_render_pile_update();
            }
            width=*HISTORY_ZONE_WIDTH_PX
            height=GOLDEN_HEIGHT
        />

        // Special placements
        <div
            style:position="absolute"
            style:left="0%"
            style:bottom="0%"
        >
            <OraclePanel width=ORACLE_ZONE_WIDTH_PX height=ORACLE_ZONE_HEIGHT_PX current_frame=current_state.into()/>
        </div>

        <Show
            when=move || current_state.get().winner.is_some()
            fallback=move |_| ()
        >
            <div
                style:position="absolute"
                style:width={move || wrap_px(placer_getter.get().scale(END_WINDOW_WIDTH_PX))}
                style:height={move || wrap_px(placer_getter.get().scale(END_WINDOW_HEIGHT_PX))}
                style:left={move || wrap_px(placer_getter.get().scale(*END_WINDOW_LEFT_PX))}
                style:top={move || wrap_px(placer_getter.get().scale(*END_WINDOW_TOP_PX))}
                style:border-radius={move || wrap_px(placer_getter.get().scale(4.0))}
                style:border-width={move || wrap_px(placer_getter.get().scale(1.0))}
                style:border-style="solid"
                style:background="white"

                style:display="flex"
                style:flex-direction="column"
                style:justify-content="space-evenly"
                style:align-items="center"
            >
                <div>
                    {end_game_text}
                </div>


                <div>
                    <Button
                        background=Signal::derive(cx, || BUTTON_SELECTED_COLOUR.to_string())
                        width=100.0
                        height=30.0
                        on:click=move |_| {
                            let mut new_history = game_history_getter.get();
                            new_history.all_frames.truncate(1);
                            game_state.set_history(new_history.clone());
                            game_state.do_render_pile_update();
                        }
                    >
                        Replay
                    </Button>

                    <div
                        style:height={move || wrap_px(placer_getter.get().scale(2.0))}
                    />

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
        </Show>
    </div>
    }
}
