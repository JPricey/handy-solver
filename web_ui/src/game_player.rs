use crate::components::*;
use crate::contexts::*;
use crate::game_card::*;
use crate::game_player_state::*;
use crate::game_player_types::*;
use crate::init_pile_provider::*;
use crate::oracle_panel::*;
use crate::screens::*;
use crate::types::*;
use crate::constants::*;
use closure::closure;
use handy_core::game::*;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::*;

const ACTION_ROW_MARGIN_PX: WindowUnit = 4.0;

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
    E,
    F,
    H,
    O,
    U,
    X,
    Slash,
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
        "KeyE" => Some(Hotkey::E),
        "KeyF" => Some(Hotkey::F),
        "KeyH" => Some(Hotkey::H),
        "KeyU" => Some(Hotkey::U),
        "KeyO" => Some(Hotkey::O),
        "KeyX" => Some(Hotkey::X),
        "Slash" => Some(Hotkey::Slash),
        "Enter" => Some(Hotkey::Ent),
        "ArrowUp" => Some(Hotkey::Up),
        "ArrowDown" => Some(Hotkey::Down),
        "ArrowLeft" => Some(Hotkey::Left),
        "ArrowRight" => Some(Hotkey::Right),
        _ => {
            // log!("{code}");
            None
        }
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
    ToggleSettings,
    ToggleSettingsBar,
    ToggleEngine,
    ToggleOnlyMoves,
    Undo,
    AnimationSkip,
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

    if hotkey == Hotkey::X || hotkey == Hotkey::Slash {
        return Some(ActionOption::ToggleSettings);
    }

    if hotkey == Hotkey::H {
        return Some(ActionOption::ToggleSettingsBar);
    }

    if hotkey == Hotkey::E {
        return Some(ActionOption::ToggleEngine);
    }

    if hotkey == Hotkey::O {
        return Some(ActionOption::ToggleOnlyMoves);
    }

    if hotkey == Hotkey::U || hotkey == Hotkey::Left {
        return Some(ActionOption::Undo);
    }

    if hotkey == Hotkey::F || hotkey == Hotkey::Right {
        return Some(ActionOption::AnimationSkip);
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
pub fn GamePlayer(cx: Scope, init_pile_provider: Box<dyn InitPileProvider>, is_playing: RwSignal<bool>) -> impl IntoView {
    let game_state = GamePlayerState::new(cx, init_pile_provider.get_init_pile());
    let game_history_getter = game_state.game_history_getter;
    let render_card_map_getter = game_state.render_card_map_getter;
    let options = use_options(cx);

    let (pile_provider_getter, _) = create_signal(cx, init_pile_provider.clone());

    let current_state = create_memo(cx, move |_| {
        game_history_getter.get().all_frames.last().unwrap().clone()
    });
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let interaction_getter = game_state.interaction_getter;

    let (is_showing_settings_getter, is_showing_settings_setter) = create_signal(cx, false);
    let is_oracle_enabled = create_rw_signal(cx, false);

    let render_cards_getter = move || {
        let mut result: Vec<RenderCard> = render_card_map_getter.get().values().copied().collect();
        result.sort_by(|a, b| {
            let important_cmp = interaction_getter.with(|i| {
                i.important_cards
                    .contains(&a.card_id)
                    .cmp(&i.important_cards.contains(&b.card_id))
            });

            if !important_cmp.is_eq() {
                return important_cmp;
            }

            b.animated_position_in_pile
                .get()
                .partial_cmp(&a.animated_position_in_pile.get())
                .unwrap()
        });
        result
    };

    let maybe_animation_queue = game_state.maybe_animation_queue;
    let is_animating = create_memo(cx, move |_| maybe_animation_queue.get().is_some());

    let undo = move || {
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
        game_state.set_history(new_history);
        game_state.do_render_pile_update();
    };

    let hint_string = move || {
        if is_animating.get() {
            return "".to_owned();
        }

        let hints = interaction_getter.get().hints;
        return hints.into_iter().collect::<Vec<_>>().join(" or ");
    };

    let end_game_text = move || {
        let cs = current_state.get();
        match cs.resolution {
            WinType::Win => "You Win!!",
            WinType::Lose => "You Lost :(",
            WinType::Unresolved => "Something went wrong",
        }
    };

    window_event_listener(ev::keydown, move |ev| {
        let code = ev.code();
        if let Some(hotkey) = code_to_hotkeyable(&code) {
            let hotkey_outcome = hotkey_to_outcome(hotkey, &interaction_getter.get());
            if let Some(outcome) = hotkey_outcome {
                ev.prevent_default();

                match outcome {
                    ActionOption::MoveOption(move_option) => {
                        game_state.apply_option(&move_option);
                    }
                    ActionOption::CardSelection(card_id) => {
                        game_state.select_card(card_id);
                    }
                    ActionOption::ToggleSettings => {
                        is_showing_settings_setter.set(!is_showing_settings_getter.get())
                    }
                    ActionOption::ToggleSettingsBar => {
                        options.update(|opts| {
                            opts.is_showing_settings_bar = !opts.is_showing_settings_bar
                        });
                    }
                    ActionOption::ToggleOnlyMoves => {
                        options.update(|opts| opts.is_pick_only_moves = !opts.is_pick_only_moves);
                    }
                    ActionOption::ToggleEngine => {
                        is_oracle_enabled.set(!is_oracle_enabled.get());
                    }
                    ActionOption::Undo => {
                        undo();
                    }
                    ActionOption::AnimationSkip => {
                        game_state.try_do_only_move();
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
                style:background="#f8eee2"
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
                        style:margin-top={move || wrap_px(placer_getter.get().scale(22.0))}
                    >
                        <For each=move || {
                            if is_animating.get() {
                                Vec::new()
                            } else {
                                get_combined_interaction_buttons(&interaction_getter.get())
                            }
                        }
                            key=|e| e.move_option.clone() view=move |cx, option| {
                            let on_click = move |_| {
                                game_state.apply_option(&option.move_option);
                            };

                            view! { cx,
                                <div
                                    style:margin={move || wrap_px(placer_getter.get().scale(ACTION_ROW_MARGIN_PX))}
                                >
                                    <ActionButton
                                        text=option.text.to_owned()
                                        is_skip=false
                                        on:click=on_click
                                    />
                                </div>
                            }
                        }/>
                        <For each=move || {
                                if is_animating.get() {
                                    Vec::new()
                                } else {
                                    interaction_getter.get().skip_button
                                }
                            }
                            key=|e| e.move_option.clone() view=move |cx, option| {
                            let on_click = move |_| {
                                game_state.apply_option(&option.move_option);
                            };

                            view! { cx,
                                <div
                                    style:margin={move || wrap_px(placer_getter.get().scale(ACTION_ROW_MARGIN_PX))}
                                >
                                    <ActionButton
                                        text=option.text.to_owned()
                                        is_skip=true
                                        on:click=on_click
                                    />
                                </div>
                            }
                        }/>
                    </div>
                    <div
                        // Card selections
                        style:display="flex"
                        style:justify-content="center"
                        style:flex-grow=1.0
                    >
                        <For each=move || {
                            if is_animating.get() {
                                Vec::new()
                            } else {
                                interaction_getter.get().damage_card_options
                            }
                        }
                        key=|e| e.clone() view=move |cx, damage_option| {
                            view! { cx,
                                <div
                                    style:margin-left={move || wrap_px(placer_getter.get().scale(ACTION_ROW_MARGIN_PX))}
                                    style:margin-right={move || wrap_px(placer_getter.get().scale(ACTION_ROW_MARGIN_PX))}
                                >
                                    <StaticGameCard
                                        card_id=damage_option.card_ptr.get_card_id()
                                        face_key=damage_option.card_ptr.get_card_face()
                                        is_clickable=true
                                        scale=1.0
                                        on:click= move |_| { game_state.apply_option(&damage_option.move_option) }
                                    />
                                </div>
                            }
                        }/>
                    </div>
                </div>

                // Cards
                <For each=render_cards_getter key=|e| e.card_id view=move |cx, render_card| {
                    let get_row_options= move || {
                            if is_animating.get() {
                                return Vec::new();
                            }

                            interaction_getter.with(|interactions| {
                                interactions.row_options.iter().filter(|option| {
                                    option.card_id == render_card.card_id
                                }).cloned().collect::<Vec<RenderRowOption>>()
                            }
                        )
                    };

                    view! { cx,
                        <InPlayGameCard
                            render_card=render_card
                            is_animating={is_animating.into()}
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
                                                class="clickable-option-overlay"
                                                style:position="absolute"
                                                style:border-radius={move || wrap_px(placer_getter.get().scale(BUTTON_BORDER_RADIUS_PX))}
                                                style:border-width={move || wrap_px(placer_getter.get().scale(SELECTABLE_BUTTON_WIDTH_PX))}
                                            />
                                            <div
                                                style:position="absolute"
                                                style:top="49%"
                                                style:left="5.2%"
                                                style:transform="translateY(-50%)"
                                            >
                                                <RowIndexBadge number=Signal::derive(cx, move || row_option.row_index + 1) scale=1.0/>
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
            do_undo = move || undo()
            width=*HISTORY_ZONE_WIDTH_PX
            height=GOLDEN_HEIGHT
        />

        // Special placements
        <div
            style:position="absolute"
            style:left="0.8%"
            style:bottom="0.8%"
            style:display="flex"
            style:align-items="end"
            style:visibility=move || if options.get().is_showing_settings_bar { "visible" } else { "hidden" }
            options
        >
            <OraclePanel
                width=ORACLE_ZONE_WIDTH_PX
                height=ORACLE_ZONE_HEIGHT_PX
                current_frame=current_state.into()
                is_enabled=is_oracle_enabled
            />

            <div
                style:width=move || wrap_px(placer_getter.get().scale(4.0))
            />

            <Button
                width=ORACLE_ZONE_WIDTH_PX
                height=40.0
                background=Signal::derive(cx, || MENU_BUTTON_COLOUR.to_owned())
                on:click =move |_| is_showing_settings_setter.set(true)
            >
                Help & Shortcuts (?)
            </Button>
        </div>

        <Show
            when=move || current_state.get().resolution.is_over()
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
                    <Show
                        when=move || pile_provider_getter.get().is_pile_random()
                        fallback=|_| ()
                    >
                        <Button
                            background=Signal::derive(cx, || BUTTON_SELECTED_COLOUR.to_string())
                            width=100.0
                            height=30.0
                            on:click=move |_| {
                                let init_pile = pile_provider_getter.get().get_init_pile();
                                game_state.set_init_pile(init_pile);
                                let init_animation = game_state.do_render_pile_update();
                                game_state.maybe_schedule_next_move(init_animation);
                            }
                        >
                            New Match
                        </Button>

                        <div
                            style:height={move || wrap_px(placer_getter.get().scale(2.0))}
                        />
                    </Show>

                    <Button
                        background=Signal::derive(cx, || BUTTON_SELECTED_COLOUR.to_string())
                        width=100.0
                        height=30.0
                        on:click=move |_| {
                            let mut new_history = game_history_getter.get();
                            new_history.all_frames.truncate(1);
                            game_state.set_history(new_history.clone());
                            let init_animation = game_state.do_render_pile_update();
                            game_state.maybe_schedule_next_move(init_animation);
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
                            is_playing.set(false)
                        }
                    >
                        Back to Menu
                    </Button>
                </div>
            </div>
        </Show>

        <Show
            when=move || is_showing_settings_getter.get()
            fallback=move |_| ()
        >
            <div
                style:position="absolute"
                style:width="100%"
                style:height="100%"
            >
                <HelperScreen
                    is_showing_settings_setter=is_showing_settings_setter
                    should_show_new_match={init_pile_provider.is_pile_random()}
                    new_match_fn=closure!(clone init_pile_provider, || {
                        let init_pile = init_pile_provider.get_init_pile();
                        game_state.set_init_pile(init_pile);
                        let init_animation = game_state.do_render_pile_update();
                        game_state.maybe_schedule_next_move(init_animation);
                    })
                    back_to_menu_fn=move || {
                        is_playing.set(false)
                    }
                    replay_fn=move || {
                        let mut new_history = game_history_getter.get();
                        new_history.all_frames.truncate(1);
                        game_state.set_history(new_history.clone());
                        let init_animation = game_state.do_render_pile_update();
                        game_state.maybe_schedule_next_move(init_animation);
                    }
                />
            </div>
        </Show>
    </div>
    }
}
