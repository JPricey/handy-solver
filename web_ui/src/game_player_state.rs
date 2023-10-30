use crate::contexts::*;
use crate::game_player_types::*;
use crate::types::*;
use core::time::Duration;
use glam::DQuat;
use handy_core::game::*;
use handy_core::utils::*;
use leptos::*;
use leptos_animation::*;
use std::cmp::max;
use std::f64::consts::PI;

const CARD_MOVE_SPEED: WindowUnit = 1000.0;
const CARD_ROTATE_SPEED: WindowUnit = 2.0 * PI;

fn selectable_row_offset(face_def: &FaceDef) -> usize {
    let mut result = 0;
    if face_def.swarm.is_some() {
        result += 1;
    }
    result += face_def.assists.len();
    result
}

fn get_only_damage_card_id(move_options: &Vec<MoveOption>) -> Option<CardId> {
    let mut result = None;

    for option in move_options {
        match option.event {
            Event::Damage(_, card_ptr, _, _) => {
                if let Some(id) = result {
                    if id != card_ptr.get_card_id() {
                        return None;
                    };
                } else {
                    result = Some(card_ptr.get_card_id());
                }
            }
            _ => (),
        }
    }

    result
}

fn placement_pct_for_row_option(row_index: usize) -> WindowUnit {
    return 0.165 + 0.11 * (row_index as f64);
}

pub fn calculate_interaction_options(game_frame: &GameFrame) -> InteractionOptions {
    let mut new_interaction_options = InteractionOptions::new(game_frame.current_pile.clone());
    let only_damage_option = get_only_damage_card_id(&game_frame.available_moves);

    let mut add_clickable_card_option =
        |card_id: CardId, clickable_card_reason: ClickableCardReason| {
            new_interaction_options
                .clickable_cards
                .insert(card_id, clickable_card_reason);
        };

    for available_move in &game_frame.available_moves {
        match &available_move.event {
            Event::Teleport(_, card1, _, card2) => {
                new_interaction_options
                    .selection_options
                    .push(CompleteSelectionOption::new(
                        vec![card1.get_card_id(), card2.get_card_id()]
                            .into_iter()
                            .collect(),
                        available_move.clone(),
                        "Teleport".to_owned(),
                    ));
                new_interaction_options
                    .hints
                    .insert("Pick Teleport Targets".to_owned());
            }
            Event::PayEnergy(cards) => {
                new_interaction_options
                    .selection_options
                    .push(CompleteSelectionOption::new(
                        cards.iter().map(|(_, ptr)| ptr.get_card_id()).collect(),
                        available_move.clone(),
                        "Pay Energy".to_owned(),
                    ));
                new_interaction_options
                    .hints
                    .insert("Pick Energy".to_owned());
            }
            Event::PickRow(row_index, card_index, card_ptr) => {
                let active_face = card_ptr.get_active_face();
                let row_offset = selectable_row_offset(active_face);
                let placement_pct: WindowUnit =
                    placement_pct_for_row_option(row_offset + *row_index);
                new_interaction_options.row_options.push(RenderRowOption {
                    placement_pct,
                    row_index: *row_index,
                    card_index: *card_index,
                    card_id: card_ptr.get_card_id(),
                    move_option: available_move.clone(),
                });
                new_interaction_options
                    .hints
                    .insert("Pick a Row to Activate".to_owned());
            }
            Event::SkipTurn(_) => {
                assert!(new_interaction_options.skip_button.len() == 0);
                new_interaction_options.skip_button.push(SkipButton {
                    move_option: available_move.clone(),
                    text: "Skip Turn".to_owned(),
                });
                new_interaction_options
                    .hints
                    .insert("Skip This Activation".to_owned());
            }
            Event::Ablaze(_, card1, _, card2) => {
                new_interaction_options
                    .selection_options
                    .push(CompleteSelectionOption::new(
                        vec![card1.get_card_id(), card2.get_card_id()]
                            .into_iter()
                            .collect(),
                        available_move.clone(),
                        "Ablaze".to_owned(),
                    ));
                new_interaction_options
                    .hints
                    .insert("Pick Ablaze Ends".to_owned());
            }
            Event::BottomCard => {
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: "Finish Activation".to_owned(),
                    });
                new_interaction_options
                    .hints
                    .insert("Finish Activation".to_owned());
            }
            Event::SkipAction(_, wrapped_action) => {
                assert!(new_interaction_options.skip_button.len() == 0);
                new_interaction_options.skip_button.push(SkipButton {
                    move_option: available_move.clone(),
                    text: format!("Skip {}", action_simple_name(&wrapped_action)),
                });
                new_interaction_options
                    .hints
                    .insert("Skip This Action".to_owned());
            }
            Event::SkipArrow => {
                assert!(new_interaction_options.skip_button.len() == 0);
                new_interaction_options.skip_button.push(SkipButton {
                    move_option: available_move.clone(),
                    text: format!("Skip Second Arrow"),
                });
                new_interaction_options
                    .hints
                    .insert("Skip This Action".to_owned());
            }
            Event::Inspire(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Inspire".to_owned());
            }
            Event::Pull(_, card_ptr) => {
                // Allow both targets and interaction button options
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: "Pull".to_owned(),
                    });
                new_interaction_options
                    .hints
                    .insert("Pull".to_owned());
            }
            Event::Push(_, card_ptr) => {
                // Allow both targets and interaction button options
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: "Push".to_owned(),
                    });
                new_interaction_options
                    .hints
                    .insert("Push".to_owned());
            }
            Event::Heal(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Heal".to_owned());
            }
            Event::Revive(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Revive".to_owned());
            }
            Event::OnHurt(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Trigger Hurt Response".to_owned());
            }
            Event::Manouver(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Manouver".to_owned());
            }
            Event::Mandatory(_, self_action) => {
                let text = format!("Mandatory {:?}", self_action);
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: text.clone(),
                    });

                new_interaction_options.hints.insert(text);
            }
            Event::Swarm(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Activate Swarm".to_owned());
            }
            Event::MoveTarget(_, card_ptr, move_type) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert(format!("Target {:?}", move_type));
            }
            Event::MoveBy(_, card_ptr, move_type, _) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                let hint_string = match move_type {
                    MoveType::Quicken => format!("Target Card to Move Above"),
                    MoveType::Delay => format!("Target Card to Move Below"),
                };
                new_interaction_options.hints.insert(hint_string);
            }
            Event::MoveResult(move_type, _) => {
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: format!("{:?}", move_type),
                    });
                new_interaction_options
                    .hints
                    .insert("Perform Move".to_string());
            }
            Event::AttackCard(_, card_ptr, _) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Attack".to_owned());
            }
            Event::Block(_, card_ptr, self_action) => {
                // If this card is getting damaged, show the block as both a damage and card option
                if Some(card_ptr.get_card_id()) == only_damage_option {
                    let mut new_card_ptr = card_ptr.clone();
                    if let Some(definite_action) = self_action {
                        perform_card_self_action(*definite_action, &mut new_card_ptr);
                    };

                    new_interaction_options
                        .damage_card_options
                        .push(DamageCardOption {
                            move_option: available_move.clone(),
                            card_ptr: new_card_ptr,
                        });

                    add_clickable_card_option(
                        card_ptr.get_card_id(),
                        ClickableCardReason::Move(available_move.clone()),
                    );

                    new_interaction_options.hints.insert("Block".to_owned());
                } else {
                    add_clickable_card_option(
                        card_ptr.get_card_id(),
                        ClickableCardReason::Move(available_move.clone()),
                    );
                    new_interaction_options
                        .hints
                        .insert("Pick Blocker".to_owned());
                }
            }
            Event::FireballTarget(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Fireball".to_owned());
            }
            Event::Damage(_, card_ptr, _, face_key) => {
                let damaged_card_ptr = CardPtr::new_from_id(card_ptr.get_card_id(), *face_key);
                new_interaction_options
                    .damage_card_options
                    .push(DamageCardOption {
                        move_option: available_move.clone(),
                        card_ptr: damaged_card_ptr,
                    });
                new_interaction_options
                    .hints
                    .insert("Take Damage".to_owned());
            }
            Event::Dodge(_, card_ptr, self_action) => {
                let mut new_card_ptr = card_ptr.clone();
                if let Some(definite_action) = self_action {
                    perform_card_self_action(*definite_action, &mut new_card_ptr);
                };

                new_interaction_options
                    .damage_card_options
                    .push(DamageCardOption {
                        move_option: available_move.clone(),
                        card_ptr: new_card_ptr,
                    });
                new_interaction_options.hints.insert("Dodge".to_owned());
            }
            Event::Void(_, card_ptr, face_key) => {
                let damaged_card_ptr = CardPtr::new_from_id(card_ptr.get_card_id(), *face_key);
                new_interaction_options
                    .damage_card_options
                    .push(DamageCardOption {
                        move_option: available_move.clone(),
                        card_ptr: damaged_card_ptr,
                    });
                new_interaction_options.hints.insert("Void".to_owned());
            }
            Event::Death => {
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: "Death".to_owned(),
                    });
                new_interaction_options.hints.insert("Death".to_owned());
            }
            Event::UseActionAssistCard(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Pick Pet to Assist".to_owned());
            }
            Event::UseActionAssistRow(card_index, card_ptr, row_index) => {
                let placement_pct: WindowUnit = placement_pct_for_row_option(*row_index);
                new_interaction_options.row_options.push(RenderRowOption {
                    placement_pct,
                    row_index: *row_index,
                    card_index: *card_index,
                    card_id: card_ptr.get_card_id(),
                    move_option: available_move.clone(),
                });
                new_interaction_options
                    .hints
                    .insert("Pick a Row to Activate".to_owned());
                new_interaction_options
                    .important_cards
                    .insert(card_ptr.get_card_id());
            }
            Event::ReactAssistUsed(_, card_ptr, _, _) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Use Reaction".to_owned());
            }
            Event::SkipReactActionAssist => {
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: "Skip React Action".to_owned(),
                    });
                new_interaction_options
                    .hints
                    .insert("Skip react action".to_owned());
            }
        }
    }

    if new_interaction_options.selection_options.len() == 1 {
        let only_selection = &new_interaction_options.selection_options[0];
        new_interaction_options.selected_cards = only_selection.selected_cards.clone();
    }

    calculate_selection_options(&mut new_interaction_options);

    return new_interaction_options;
}

pub fn calculate_selection_options(interaction_options: &mut InteractionOptions) {
    // Clear out all clickable cards
    interaction_options.clickable_cards.retain(|_, v| match v {
        ClickableCardReason::Select => false,
        _ => true,
    });

    let mut add_clickable_card_option = |card_id: CardId| {
        interaction_options
            .clickable_cards
            .insert(card_id, ClickableCardReason::Select);
    };

    interaction_options.valid_selection_buttons.clear();

    // Can always deselect
    for card_id in &interaction_options.selected_cards {
        add_clickable_card_option(*card_id);
    }

    for complete_selection_option in &interaction_options.selection_options {
        let remaining_options =
            &complete_selection_option.selected_cards - &interaction_options.selected_cards;
        // If choosable cards for this option is a superset of selected cards, add the
        // remaining cards as selectablw options
        if remaining_options.len()
            == complete_selection_option.selected_cards.len()
                - interaction_options.selected_cards.len()
        {
            for option in remaining_options {
                add_clickable_card_option(option)
            }
        }

        if interaction_options.selected_cards == complete_selection_option.selected_cards {
            interaction_options
                .valid_selection_buttons
                .push(InteractionButton {
                    move_option: complete_selection_option.move_option.clone(),
                    text: complete_selection_option.completed_text.clone(),
                });
        }
    }
}

fn place_within_inclusive(
    index: usize,
    max: usize,
    small_amount: WindowUnit,
    big_amount: WindowUnit,
) -> WindowUnit {
    if max == 0 {
        return small_amount;
    }
    let delta = big_amount - small_amount;
    return small_amount + delta * (1.0 - (index as WindowUnit / max as WindowUnit));
}

pub fn get_card_position(card_index: usize, pile_len: usize, is_selected: bool) -> Point2D {
    let mut y = place_within_inclusive(
        card_index,
        pile_len - 1,
        *BOTTOM_CARD_Y_OFFSET_PX,
        *TOP_CARD_Y_OFFSET_PX,
    );
    if is_selected {
        y += SELECTED_Y_DELTA_PX;
    }
    let x = place_within_inclusive(
        card_index,
        pile_len - 1,
        *CARD_ZONE_BUFFER_WIDTH,
        *TOP_CARD_LEFT_PX,
    );

    Point2D::new(x, y)
}

pub fn render_pile_update(
    render_card_map: &mut RenderCardMap,
    interaction_options: &InteractionOptions,
    pile: &Pile,
) -> Duration {
    let pile_len = pile.len();

    let mut max_applied_duration = Duration::new(0, 0);
    for (i, card) in pile.iter().enumerate() {
        let render_card = render_card_map.get_mut(&card.get_card_id()).unwrap();

        let is_selected = interaction_options
            .selected_cards
            .contains(&card.get_card_id());

        render_card.active_face.set(card.get_card_face());

        let current_pos = render_card.animated_point.get_untracked();
        let desired_pos = get_card_position(i, pile_len, is_selected);

        if current_pos != desired_pos {
            let position_s = (current_pos - desired_pos).length() / CARD_MOVE_SPEED;
            let distance_duration = Duration::from_secs_f64(position_s);
            max_applied_duration = max(max_applied_duration, distance_duration);
            render_card.point.set((desired_pos, distance_duration));
            render_card
                .position_in_pile
                .set((i as f64, distance_duration));
        }

        let current_quat = render_card.animated_quat.get_untracked();
        let desired_quat = quat_for_face(card.get_card_face());

        if current_quat != desired_quat {
            let rotation_s = current_quat.angle_between(desired_quat) / CARD_ROTATE_SPEED;
            let rotation_duration = Duration::from_secs_f64(rotation_s);

            max_applied_duration = max(max_applied_duration, rotation_duration);
            render_card.quat.set((desired_quat, rotation_duration));
        }

        let is_clickable = interaction_options
            .clickable_cards
            .contains_key(&card.get_card_id());
        render_card.is_clickable.set(is_clickable);
    }

    return max_applied_duration;
}

pub fn is_state_longer_event_prefix(
    prefix: &Vec<Event>,
    state: &GameStateWithPileTrackedEventLog,
) -> bool {
    let events = &state.events;
    if events.len() < prefix.len() {
        return false;
    }

    for i in 0..prefix.len() {
        if prefix[i] != events[i].1 {
            return false;
        }
    }

    return true;
}

pub fn find_next_moves(pile: &Pile, prefix: &Vec<Event>) -> Vec<MoveOption> {
    let states = resolve_top_card(&GameStateWithPileTrackedEventLog::new(pile.clone()));

    let mut results: Vec<MoveOption> = vec![];
    for state in states {
        if is_state_longer_event_prefix(prefix, &state) {
            let (new_pile, new_event) = state.events[prefix.len()].clone();
            let move_option = MoveOption {
                next_pile: new_pile,
                event: new_event,
            };
            if !results.contains(&move_option) {
                results.push(move_option);
            }
        }
    }

    return results;
}

pub fn get_frame_from_root_pile(pile: Pile) -> GameFrame {
    let winner = is_game_winner(&pile);
    let available_moves = if winner.is_some() {
        Vec::new()
    } else {
        find_next_moves(&pile, &Vec::new())
    };

    GameFrame {
        root_pile: pile.clone(),
        current_pile: pile.clone(),
        event_history: Vec::new(),
        available_moves,
        winner,
    }
}

pub fn get_frame_from_option(last_frame: &GameFrame, option: &MoveOption) -> GameFrame {
    let mut new_history = last_frame.event_history.clone();
    new_history.push(option.event.clone());

    let available_moves = find_next_moves(&last_frame.root_pile, &new_history);

    GameFrame {
        root_pile: last_frame.root_pile.clone(),
        event_history: new_history,
        current_pile: option.next_pile.clone(),
        available_moves,
        winner: None,
    }
}

pub fn dquat_tween(from: &DQuat, to: &DQuat, progress: f64) -> DQuat {
    let res = from.slerp(to.clone(), progress);
    res
}

fn get_init_render_pile(cx: Scope, init_pile: &Pile) -> RenderCardMap {
    init_pile
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let active_face = create_rw_signal(cx, FaceKey::A);

            let point = create_rw_signal(cx, (Point2D::default(), Duration::new(0, 0)));
            let animated_point =
                create_animated_signal(cx, move || point.get().into(), tween_default);

            let quat = create_rw_signal(cx, (quat_for_face(FaceKey::A), Duration::new(0, 0)));
            let animated_quat = create_animated_signal(cx, move || quat.get().into(), dquat_tween);

            let position_in_pile = create_rw_signal(cx, (i as WindowUnit, Duration::new(0, 0)));
            let animated_position_in_pile = create_animated_signal(
                cx,
                move || {
                    <(f64, Duration) as Into<AnimationTarget<f64>>>::into(position_in_pile.get())
                },
                tween_default,
            );

            let render_card = RenderCard {
                card_id: card.get_card_id(),
                active_face,

                point,
                animated_point,

                quat,
                animated_quat,

                position_in_pile,
                animated_position_in_pile,

                is_clickable: create_rw_signal(cx, false),
            };

            (card.card_id, render_card)
        })
        .collect()
}

#[derive(Clone, Copy)]
pub struct GamePlayerState {
    game_history: RwSignal<GameHistory>,
    interaction_setter: WriteSignal<InteractionOptions>,
    options: RwSignal<Options>,

    pub maybe_animation_queue: RwSignal<Option<TimeoutHandle>>,
    pub game_history_getter: Signal<GameHistory>,
    pub render_card_map_getter: Signal<RenderCardMap>,
    pub interaction_getter: ReadSignal<InteractionOptions>,
}

impl GamePlayerState {
    pub fn new(cx: Scope, init_pile: Pile) -> Self {
        let initial_frame = get_frame_from_root_pile(init_pile.clone());
        let init_state = GameHistory {
            all_frames: vec![initial_frame.clone()],
        };
        let game_history = create_rw_signal(cx, init_state);
        let game_history_getter = game_history.into();

        let (render_card_map_getter, _) = create_signal(cx, get_init_render_pile(cx, &init_pile));

        let (interaction_getter, interaction_setter) = create_signal::<InteractionOptions>(
            cx,
            InteractionOptions::new(initial_frame.current_pile.clone()),
        );

        let maybe_animation_queue = create_rw_signal(cx, None);

        Self {
            game_history,
            game_history_getter,
            render_card_map_getter: render_card_map_getter.into(),
            interaction_getter,
            interaction_setter,
            maybe_animation_queue,
            options: use_options(cx),
        }
    }

    pub fn set_init_pile(self, pile: Pile) {
        let initial_frame = get_frame_from_root_pile(pile.clone());
        let init_state = GameHistory {
            all_frames: vec![initial_frame.clone()],
        };
        self.clear_animation();
        self.game_history.set(init_state);
        self.do_render_pile_update();
    }

    pub fn clear_animation(self) {
        if let Some(animation_handle) = self.maybe_animation_queue.get_untracked() {
            animation_handle.clear();
            self.maybe_animation_queue.set(None);
        }
    }

    pub fn set_history(self, game_history: GameHistory) {
        self.clear_animation();
        self.game_history.set(game_history)
    }

    fn get_current_frame(self) -> GameFrame {
        self.game_history_getter
            .get_untracked()
            .all_frames
            .last()
            .unwrap()
            .clone()
    }

    pub fn do_render_pile_update(self) -> Duration {
        let game_frame = self.get_current_frame();
        let new_interaction_options = calculate_interaction_options(&game_frame);
        let mut render_card_map = self.render_card_map_getter.get_untracked();
        let result = render_pile_update(
            &mut render_card_map,
            &new_interaction_options,
            &game_frame.current_pile,
        );
        self.interaction_setter.set(new_interaction_options);
        return result;
    }

    pub fn apply_single_option(self, move_option: &MoveOption) -> Duration {
        let mut game_history = self.game_history_getter.get();
        let available_moves = game_history
            .all_frames
            .last()
            .unwrap()
            .available_moves
            .clone();

        if !available_moves.contains(move_option) {
            return self.do_render_pile_update();
        }

        if move_option.event == Event::BottomCard {
            game_history
                .all_frames
                .push(get_frame_from_root_pile(move_option.next_pile.clone()));
        } else {
            let next_frame =
                get_frame_from_option(game_history.all_frames.last().unwrap(), &move_option);
            game_history.all_frames.push(next_frame.clone());
        }

        self.set_history(game_history.clone());
        return self.do_render_pile_update();
    }

    pub fn apply_option(self, move_option: &MoveOption) {
        let animation_duration = self.apply_single_option(move_option);
        self.maybe_schedule_next_move(animation_duration);
    }

    pub fn try_do_only_move(self) {
        let game_frame = self.get_current_frame();
        if game_frame.available_moves.len() != 1 {
            return;
        }
        let only_move = game_frame.available_moves[0].clone();
        self.apply_option(&only_move);
    }

    pub fn maybe_schedule_next_move(self, animation_duration: Duration) {
        if !self.options.get_untracked().is_pick_only_moves {
            return;
        }

        let game_frame = self.get_current_frame();
        if game_frame.available_moves.len() != 1 {
            return;
        }

        let only_move = game_frame.available_moves[0].clone();

        let animation_result =
            set_timeout_with_handle(move || self.apply_option(&only_move), animation_duration);
        self.clear_animation();
        self.maybe_animation_queue.set(animation_result.ok());
    }

    pub fn select_card(self, card_id: CardId) {
        let mut new_interaction_options = self.interaction_getter.get();
        let already_contains = new_interaction_options.selected_cards.contains(&card_id);
        if already_contains {
            new_interaction_options.selected_cards.remove(&card_id);
        } else {
            new_interaction_options.selected_cards.insert(card_id);
        }

        calculate_selection_options(&mut new_interaction_options);

        let mut render_card_map = self.render_card_map_getter.get();
        render_pile_update(
            &mut render_card_map,
            &new_interaction_options,
            &self
                .game_history_getter
                .get()
                .all_frames
                .last()
                .unwrap()
                .current_pile,
        );
        self.interaction_setter.set(new_interaction_options);
    }
}
