use crate::components::game_card::*;
use crate::contexts::*;
use crate::game_player_types::*;
use crate::types::*;
use core::time::Duration;
use glam::DQuat;
use handy_core::game::end_game::{is_game_winner, GameEndCheckType};
use handy_core::game::*;
use handy_core::utils::*;
use leptos::*;
use leptos_animation::AnimatedSignal;
use leptos_animation::{create_animated_signal, tween_default, AnimationTarget};

use leptos_dom::helpers::TimeoutHandle;
use std::cmp::max;
use std::f64::consts::PI;

const BASE_CARD_MOVE_SPEED: WindowUnit = 1000.0;
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
        match option.get_primary_event() {
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
    0.165 + 0.11 * (row_index as f64)
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
        match &available_move.get_primary_event() {
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
            Event::PayRowConditionCosts(cost_type, cards) => {
                new_interaction_options
                    .selection_options
                    .push(CompleteSelectionOption::new(
                        cards.iter().map(|(_, ptr)| ptr.get_card_id()).collect(),
                        available_move.clone(),
                        format!("Pay {}", string_condition_cost_type(cost_type)),
                    ));
                new_interaction_options.hints.insert(format!(
                    "Pick {} Sources",
                    string_condition_cost_type(cost_type)
                ));
            }
            Event::UseCardModifiers(cards, amount, wrapped_action) => {
                let amount_string = match amount {
                    0.. => format!("+{amount}"),
                    _ => format!("{amount}"),
                };
                let action_text = action_simple_name(&wrapped_action);
                new_interaction_options
                    .selection_options
                    .push(CompleteSelectionOption::new(
                        cards.iter().map(|(_, ptr)| ptr.get_card_id()).collect(),
                        available_move.clone(),
                        format!("Modify {action_text} {amount_string}"),
                    ));
                new_interaction_options
                    .hints
                    .insert("Pick Modifiers".to_owned());
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
            Event::SkipAction(_, wrapped_action, _) => {
                assert!(new_interaction_options.skip_button.len() == 0);
                new_interaction_options.skip_button.push(SkipButton {
                    move_option: available_move.clone(),
                    text: format!("Skip {}", action_simple_name(&wrapped_action)),
                });
                new_interaction_options
                    .hints
                    .insert("Skip This Action".to_owned());
            }
            Event::StartAction(_, wrapped_action) => {
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: format!("Start {}", action_simple_name(&wrapped_action)),
                    });
                new_interaction_options
                    .hints
                    .insert("Pick Action".to_owned());
            }
            Event::SkipHit(hit_type) => {
                assert!(new_interaction_options.skip_button.len() == 0);
                new_interaction_options.skip_button.push(SkipButton {
                    move_option: available_move.clone(),
                    text: format!("Skip {:?}", hit_type),
                });
                new_interaction_options
                    .hints
                    .insert(format!("Skip {:?}", hit_type));
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
            Event::Hypnosis(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Hypnosis".to_owned());
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
                new_interaction_options.hints.insert("Pull".to_owned());
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
                new_interaction_options.hints.insert("Push".to_owned());
            }
            Event::EndPileMoveResult(move_type) => {
                let move_type_str = format!("{:?}", move_type);
                new_interaction_options
                    .interaction_buttons
                    .push(InteractionButton {
                        move_option: available_move.clone(),
                        text: move_type_str.clone(),
                    });
                new_interaction_options.hints.insert(move_type_str.clone());
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
            Event::Rat(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options.hints.insert("Heal Rat".to_owned());
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
            Event::Maneuver(_, card_ptr) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert("Target Maneuver".to_owned());
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
            Event::AttackCard(_, card_ptr, hit_type) => {
                add_clickable_card_option(
                    card_ptr.get_card_id(),
                    ClickableCardReason::Move(available_move.clone()),
                );
                new_interaction_options
                    .hints
                    .insert(format!("Target {:?}", hit_type));
            }
            Event::Block(_, card_ptr, _self_action, face_key) => {
                let mut new_card_ptr = card_ptr.clone();
                new_card_ptr.key = *face_key;
                // If this card is getting damaged, show the block as both a damage and card option
                if Some(card_ptr.get_card_id()) == only_damage_option {
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
            Event::WhiffHit(_, card_ptr, _) => {
                new_interaction_options
                    .damage_card_options
                    .push(DamageCardOption {
                        move_option: available_move.clone(),
                        card_ptr: card_ptr.clone(),
                    });
                new_interaction_options.hints.insert("Whiff Hit".to_owned());
            }
            Event::Dodge(_, card_ptr, _self_action, face_key) => {
                let mut new_card_ptr = card_ptr.clone();
                new_card_ptr.key = *face_key;

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

    new_interaction_options
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
        // remaining cards as selectable options
        if remaining_options.len() as i64
            == complete_selection_option.selected_cards.len() as i64
                - interaction_options.selected_cards.len() as i64
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
    small_amount + delta * (1.0 - (index as WindowUnit / max as WindowUnit))
}

pub fn get_card_position(
    card_index: usize,
    pile_len: usize,
    zone_width: WindowUnit,
    is_selected: bool,
) -> Point2D {
    let mut y = place_within_inclusive(
        card_index,
        pile_len - 1,
        *BOTTOM_CARD_Y_OFFSET_PX,
        *TOP_CARD_Y_OFFSET_PX,
    );
    if is_selected {
        y += SELECTED_Y_DELTA_PX;
    }
    let max_x = zone_width - *CARD_ZONE_BUFFER_WIDTH - RENDER_CARD_SIZE.0;

    let x = place_within_inclusive(card_index, pile_len - 1, *CARD_ZONE_BUFFER_WIDTH, max_x);

    Point2D::new(x, y)
}

pub fn render_pile_update(
    render_card_map: &mut RenderCardMap,
    interaction_options: &InteractionOptions,
    pile: &Pile,
    card_zone_width_px: WindowUnit,
) -> Duration {
    let card_move_speed =
        (card_zone_width_px / (GOLDEN_MIN_WIDTH - *HISTORY_ZONE_WIDTH_PX)) * BASE_CARD_MOVE_SPEED;
    let pile_len = pile.len();

    let mut max_applied_duration = Duration::new(0, 0);
    for (i, card) in pile.iter().enumerate() {
        let render_card = render_card_map.get_mut(&card.get_card_id()).unwrap();

        let is_selected = interaction_options
            .selected_cards
            .contains(&card.get_card_id());

        render_card.active_face.set(card.get_card_face());

        let current_pos = render_card.animated_point.get_untracked();
        let desired_pos = get_card_position(i, pile_len, card_zone_width_px, is_selected);

        if current_pos != desired_pos {
            let position_s = (current_pos - desired_pos).length() / card_move_speed;
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

    max_applied_duration
}

pub fn find_next_moves(
    pile: &Pile,
    prefix: &Vec<Event>,
    game_end_check_type: GameEndCheckType,
) -> (Vec<MoveOption>, bool) {
    let states = resolve_top_card_starting_with_prefix_dedupe_excess(
        &GameStateWithPileTrackedEventLog::new(pile.clone()),
        prefix,
    );
    let mut is_definite_win = true;

    let mut results: Vec<MoveOption> = vec![];
    for state in states {
        if let Some((new_pile, next_events)) =
            get_next_available_events_past_prefix_allowing_skips(prefix, &state)
        {
            let move_option = MoveOption::new(next_events, new_pile);
            if !results.contains(&move_option) {
                results.push(move_option);
            }
            if is_definite_win && is_game_winner(&state.pile, game_end_check_type) != WinType::Win {
                is_definite_win = false;
            }
        }
    }

    is_definite_win = is_definite_win && results.len() > 0;

    assert!(results.len() > 0, "Could not find any next move options");

    (results, is_definite_win)
}

pub fn get_frame_from_root_pile(pile: Pile, game_end_check_type: GameEndCheckType) -> GameFrame {
    let resolution = is_game_winner(&pile, game_end_check_type);
    let (available_moves, is_definite_win) = if resolution == WinType::Unresolved {
        find_next_moves(&pile, &Vec::new(), game_end_check_type)
    } else {
        (Vec::new(), false)
    };

    GameFrame {
        root_pile: pile.clone(),
        current_pile: pile.clone(),
        event_history: Vec::new(),
        events_since_last_fame_this_activation: Vec::new(),
        available_moves,
        resolution,
        is_definite_win,
    }
}

pub fn get_frame_from_option(
    last_frame: &GameFrame,
    option: &MoveOption,
    game_end_check_type: GameEndCheckType,
) -> GameFrame {
    let mut new_event_history = last_frame.event_history.clone();
    new_event_history.extend(option.events.clone());

    let (available_moves, is_definite_win) = find_next_moves(
        &last_frame.root_pile,
        &new_event_history,
        game_end_check_type,
    );

    GameFrame {
        root_pile: last_frame.root_pile.clone(),
        event_history: new_event_history,
        events_since_last_fame_this_activation: option.events.clone(),
        current_pile: option.next_pile.clone(),
        available_moves,
        resolution: WinType::Unresolved,
        is_definite_win,
    }
}

pub fn dquat_tween(from: &DQuat, to: &DQuat, progress: f64) -> DQuat {
    let res = from.slerp(to.clone(), progress);
    res
}

fn get_init_render_pile(init_pile: &Pile) -> RenderCardMap {
    init_pile
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let active_face = create_rw_signal(FaceKey::A);

            let point = create_rw_signal((Point2D::default(), Duration::new(0, 0)));
            let animated_point: AnimatedSignal<Point2D, Point2D> =
                create_animated_signal(move || point.get().into(), tween_default);

            let quat = create_rw_signal((quat_for_face(FaceKey::A), Duration::new(0, 0)));
            let animated_quat = create_animated_signal(move || quat.get().into(), dquat_tween);

            let position_in_pile = create_rw_signal((i as WindowUnit, Duration::new(0, 0)));
            let animated_position_in_pile = create_animated_signal(
                move || {
                    <(f64, Duration) as Into<AnimationTarget<f64>>>::into(position_in_pile.get())
                },
                tween_default,
            );

            let is_important = create_rw_signal(false);
            let z_index = create_memo(move |_| {
                let important_offset = if is_important.get() { 100.0 } else { 0.0 };

                ((important_offset - animated_position_in_pile.get() + 100.0) * 100.0) as i32
            });

            let render_card = RenderCard {
                card_id: card.get_card_id(),
                active_face,

                point,
                animated_point: Signal::derive(move || animated_point.get()),

                quat,
                animated_quat: Signal::derive(move || animated_quat.get()),

                position_in_pile,
                animated_position_in_pile: Signal::derive(move || animated_position_in_pile.get()),
                is_important,
                z_index: z_index.into(),

                is_clickable: create_rw_signal(false),
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
    pub card_zone_width_px: Signal<WindowUnit>,
    pub game_end_check_type: Memo<GameEndCheckType>,
}

impl GamePlayerState {
    pub fn new(
        init_pile: Pile,
        card_zone_width_px: Signal<WindowUnit>,
        game_end_check_type: Memo<GameEndCheckType>,
    ) -> Self {
        let initial_frame =
            get_frame_from_root_pile(init_pile.clone(), game_end_check_type.get_untracked());
        let init_state = GameHistory {
            all_frames: vec![initial_frame.clone()],
        };
        let game_history = create_rw_signal(init_state);
        let game_history_getter = game_history.into();

        let (render_card_map_getter, _) = create_signal(get_init_render_pile(&init_pile));

        let (interaction_getter, interaction_setter) = create_signal::<InteractionOptions>(
            InteractionOptions::new(initial_frame.current_pile.clone()),
        );

        let maybe_animation_queue = create_rw_signal(None);

        Self {
            game_history,
            game_history_getter,
            render_card_map_getter: render_card_map_getter.into(),
            interaction_getter,
            interaction_setter,
            maybe_animation_queue,
            options: use_options(),
            card_zone_width_px,
            game_end_check_type,
        }
    }

    fn set_interaction_options(self, new_interaction_options: InteractionOptions) {
        let render_card_map = self.render_card_map_getter.get_untracked();
        for render_card in render_card_map.values() {
            render_card.is_important.set(
                new_interaction_options
                    .important_cards
                    .contains(&render_card.card_id),
            );
        }
        self.interaction_setter.set(new_interaction_options);
    }

    pub fn set_init_pile(self, pile: Pile) {
        let initial_frame =
            get_frame_from_root_pile(pile.clone(), self.game_end_check_type.get_untracked());
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
            self.card_zone_width_px.get_untracked(),
        );
        self.set_interaction_options(new_interaction_options);
        result
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

        if move_option.get_primary_event() == &Event::BottomCard {
            game_history.all_frames.push(get_frame_from_root_pile(
                move_option.next_pile.clone(),
                self.game_end_check_type.get_untracked(),
            ));
        } else {
            let next_frame = get_frame_from_option(
                game_history.all_frames.last().unwrap(),
                &move_option,
                self.game_end_check_type.get_untracked(),
            );
            game_history.all_frames.push(next_frame.clone());
        }

        self.set_history(game_history.clone());
        self.do_render_pile_update()
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
        if !(game_frame.available_moves.len() == 1 || game_frame.is_definite_win) {
            return;
        }

        let forced_move = game_frame.available_moves[0].clone();

        let animation_result =
            set_timeout_with_handle(move || self.apply_option(&forced_move), animation_duration);
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

        if let Some(single_move_option) = new_interaction_options.should_force_selection_options() {
            self.apply_option(&single_move_option);
        } else {
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
                self.card_zone_width_px.get_untracked(),
            );
            self.set_interaction_options(new_interaction_options);
        }
    }
}
