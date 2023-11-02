use crate::game::card_ptr::*;
use crate::game::game_state::*;
use crate::game::primitives::*;
use arrayvec::ArrayVec;

use enum_map::EnumMap;
use itertools::Itertools;
use std::cmp;
use strum::IntoEnumIterator;

const NO_ENERGY_USED: EnergyIds = vec![];

fn did_states_change<T: EngineGameState>(state_agg: &Vec<T>, state: &T) -> bool {
    !(state_agg.len() == 1 && state_agg[0].get_pile() == state.get_pile())
}

// Public
pub fn resolve_top_card<T: EngineGameState>(state: &T) -> Vec<T> {
    let mut card_outcomes = resolve_card_at_index(state, 0);
    if card_outcomes.len() == 0 {
        panic!("Saw no outcomes for top card: {:?}", state.get_pile())
    }

    for outcome in &mut card_outcomes {
        bottom_top_card(outcome);
    }

    card_outcomes
}

fn resolve_card_at_index<T: EngineGameState>(state: &T, active_idx: usize) -> Vec<T> {
    let pile = state.get_pile();
    let active_card = &pile[active_idx];
    let active_face = &active_card.get_active_face();
    let allegiance = active_card.get_active_face().allegiance;

    match allegiance {
        Allegiance::Baddie | Allegiance::Werewolf => {
            resolve_enemy_turn(state, allegiance, active_idx)
        }
        Allegiance::Hero => {
            let mut all_outcomes: Vec<T> = vec![];

            for (row_idx, row) in active_face.rows.iter().enumerate() {
                let base_row_state =
                    state
                        .clone()
                        .append_event(Event::PickRow(row_idx, active_idx, *active_card));
                let row_outcomes = resolve_player_row(&base_row_state, row, active_idx);
                all_outcomes.extend(row_outcomes);
            }

            all_outcomes.push(state.clone().append_event(Event::SkipTurn(*active_card)));
            all_outcomes
        }
    }
}

// Player only
fn resolve_player_row<T: EngineGameState>(state: &T, row: &Row, active_idx: usize) -> Vec<T> {
    let pile = state.get_pile();
    let mut active_states = match row.condition {
        Some(Condition::Energy(amount)) => {
            let mut energy_options: Vec<EnergyId> = vec![];
            for i in active_idx + 1..pile.len() {
                if pile[i]
                    .get_active_face()
                    .features
                    .intersects(Features::Energy)
                {
                    energy_options.push(i as EnergyId)
                }
            }

            let mut state_agg = vec![];
            for energy_combo in energy_options.iter().copied().combinations(amount as usize) {
                let mut energy_used_state = state.clone();
                let mut used_energy_event_data: PayEnergyArrType = ArrayVec::new();
                for used_energy_idx in &energy_combo {
                    let new_key = rotate_key(energy_used_state.get_pile()[*used_energy_idx].key);
                    energy_used_state.get_pile_mut()[*used_energy_idx].key = new_key;
                    let used_energy_card = state.get_pile()[*used_energy_idx];
                    used_energy_event_data.push((*used_energy_idx, used_energy_card));
                }
                energy_used_state.mut_append_event(Event::PayEnergy(used_energy_event_data));

                let new_states = resolve_player_row_post_conditions_no_mandatory(
                    &energy_used_state,
                    row,
                    active_idx,
                    &energy_combo,
                );

                state_agg.extend(new_states);
            }
            state_agg
        }
        None => {
            resolve_player_row_post_conditions_no_mandatory(state, row, active_idx, &NO_ENERGY_USED)
        }
        _ => {
            panic!("Unhandled condition {:?}", row.condition)
        }
    };

    if let Some(self_action) = row.mandatory {
        for active_state in &mut active_states {
            perform_mandatory_action(active_state, self_action, active_idx);
        }
    }

    return active_states;
}

fn resolve_player_row_post_conditions_no_mandatory<T: EngineGameState>(
    state: &T,
    row: &Row,
    active_idx: usize,
    energy_options: &EnergyIds,
) -> Vec<T> {
    let mut active_states = vec![state.clone()];

    for action in &row.actions {
        let mut next_active_states: Vec<T> = vec![];
        for current_state in active_states {
            let new_states =
                resolve_player_action(&current_state, action, active_idx, energy_options);
            next_active_states.extend(new_states);
        }
        active_states = next_active_states;
    }

    active_states
}

fn resolve_player_action<T: EngineGameState>(
    pre_event_state: &T,
    wrapped_action: &WrappedAction,
    active_idx: usize,
    energy_ids: &EnergyIds,
) -> Vec<T> {
    let mut results: Vec<T> = vec![];

    // let state = pre_event_state.clone().append_event(Event::StartAction(
    //     pre_event_state.get_pile()[active_idx],
    //     *wrapped_action,
    // ));

    let state = pre_event_state.clone();
    let pile = state.get_pile();
    results.push(pre_event_state.clone().append_event(Event::SkipAction(
        pre_event_state.get_pile()[active_idx],
        *wrapped_action,
    )));

    match wrapped_action.action {
        Action::Pull(_)
        | Action::Push(_)
        | Action::Death
        | Action::Void
        | Action::SpacedClaws(_) => {
            panic!("Skipping unimplemented player action {:?}", wrapped_action)
        }
        Action::CallAssist => {
            let assist_outcomes = _get_assist_action_outcomes(&state, active_idx, None);
            for (outcome, _) in assist_outcomes {
                results.push(outcome)
            }
        }
        Action::CallAssistTwice => {
            // HACKY: Calling 2 assists in a row comes with the restriction of not being able to
            // call the same card twice
            // Instead of passing state between actions, we instead use the action
            // "CallAssistTwice" to represent 2 assists, to make it easier to pass the state
            // between those actions
            let assist_outcomes = _get_assist_action_outcomes(&state, active_idx, None);
            for (outcome, used_assist_id) in assist_outcomes {
                for (double_assist_outcome, _) in
                    _get_assist_action_outcomes(&outcome, active_idx, Some(used_assist_id))
                {
                    results.push(double_assist_outcome);
                }
                results.push(outcome)
            }
        }
        Action::Inspire => {
            for target_idx in active_idx + 1..pile.len() {
                let target_card = pile[target_idx];
                if is_allegiance_match(
                    Allegiance::Hero,
                    target_card.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    let inspire_state = state
                        .clone()
                        .append_event(Event::Inspire(target_idx, pile[target_idx]));
                    results.extend(resolve_card_at_index(&inspire_state, target_idx));
                }
            }
        }
        Action::Revive => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }

            for target_idx in active_idx + 1..pile.len() {
                let target_card = pile[target_idx];
                if !is_allegiance_match(
                    Allegiance::Hero,
                    target_card.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    continue;
                }

                if target_card.get_active_face().health == Health::Empty {
                    let mut new_state = state.clone();
                    new_state.get_pile_mut()[target_idx].key = FaceKey::A;
                    new_state.mut_append_event(Event::Revive(
                        target_idx,
                        new_state.get_pile()[target_idx],
                    ));
                    results.push(new_state);
                }
            }
        }
        Action::Claws(range) => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }

            let range_cap = match range {
                Range::Inf => pile.len(),
                Range::Int(amount) => cmp::min(active_idx + amount + 1, pile.len()),
            };

            let states = attack_all_in_range(
                &state,
                Allegiance::Hero,
                active_idx + 1,
                range_cap,
                wrapped_action.target,
                HitType::Claw,
            );

            results.extend(states)
        }
        Action::Ablaze => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }

            for i in 0..energy_ids.len() {
                let energy_start_idx = (energy_ids[i]) as usize;
                for j in i + 1..energy_ids.len() {
                    let energy_end_idx = energy_ids[j] as usize;

                    let state_with_ablaze_event = state.clone().append_event(Event::Ablaze(
                        energy_start_idx,
                        pile[energy_start_idx],
                        energy_end_idx,
                        pile[energy_end_idx],
                    ));

                    let post_attack_states = attack_all_in_range(
                        &state_with_ablaze_event,
                        Allegiance::Hero,
                        energy_start_idx + 1,
                        energy_end_idx,
                        wrapped_action.target,
                        HitType::Ablaze,
                    );

                    results.extend(post_attack_states);
                }
            }
        }
        Action::Fireball => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }

            for energy_idx in energy_ids {
                let behind_idx = energy_idx + 1;

                let mut fireball_results = vec![state
                    .clone()
                    .append_event(Event::FireballTarget(*energy_idx, pile[*energy_idx]))];

                if behind_idx < pile.len() {
                    fireball_results =
                        attack_idx_in_all_states(fireball_results, behind_idx, HitType::Fireball);
                }
                if *energy_idx > active_idx + 1 {
                    let infront_attack_idx = energy_idx - 1;
                    fireball_results = attack_idx_in_all_states(
                        fireball_results,
                        infront_attack_idx,
                        HitType::Fireball,
                    );
                }

                results.extend(fireball_results);
            }
        }
        Action::Teleport => {
            if is_action_prevented(pile, Features::Web, active_idx) {
                return results;
            }

            for first_idx in active_idx + 1..pile.len() {
                let first_card = pile[first_idx];
                if first_card
                    .get_active_face()
                    .features
                    .intersects(Features::Weight)
                {
                    continue;
                }

                let first_allegiance = first_card.get_active_face().allegiance;
                if !(wrapped_action.target == Target::Any
                    || is_allegiance_match(
                        Allegiance::Hero,
                        first_allegiance,
                        wrapped_action.target,
                    ))
                {
                    continue;
                }

                for second_idx in first_idx + 1..pile.len() {
                    let second_card = pile[second_idx];
                    if second_card
                        .get_active_face()
                        .features
                        .intersects(Features::Weight)
                    {
                        continue;
                    }

                    let second_allegiance = second_card.get_active_face().allegiance;
                    if wrapped_action.target == Target::Any {
                        if (first_allegiance == Allegiance::Hero)
                            == (second_allegiance == Allegiance::Hero)
                        {
                            continue;
                        }
                    } else {
                        if !is_allegiance_match(
                            Allegiance::Hero,
                            second_allegiance,
                            wrapped_action.target,
                        ) {
                            continue;
                        }
                    }

                    let mut new_state = state.clone();
                    new_state.get_pile_mut().swap(first_idx, second_idx);
                    new_state.mut_append_event(Event::Teleport(
                        first_idx,
                        pile[first_idx],
                        second_idx,
                        pile[second_idx],
                    ));
                    results.push(new_state);
                }
            }
        }
        Action::Hit(range) => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }

            let range_cap = match range {
                Range::Inf => pile.len(),
                Range::Int(amount) => cmp::min(pile.len(), active_idx + amount + 1),
            };
            let pile = state.get_pile();

            let mut attack_candidates: EnumMap<Allegiance, bool> = EnumMap::default();

            for other in Allegiance::iter() {
                let is_match = is_allegiance_match(Allegiance::Hero, other, wrapped_action.target);
                attack_candidates[other] = is_match;
            }
            // Player can never block for team. We get that choice when we attack them directly.
            let is_player_candidate = attack_candidates[Allegiance::Hero];
            attack_candidates[Allegiance::Hero] = false;

            // Find all blockers
            for blocker_idx in (active_idx + 1..range_cap).rev() {
                let blocker_card_ptr = &pile[blocker_idx];
                let blocker_face = blocker_card_ptr.get_active_face();
                if !attack_candidates[blocker_face.allegiance] {
                    continue;
                }

                let block_results =
                    try_prevent_action_with_reaction(&state, blocker_idx, ReactionTrigger::Block);
                if block_results.len() > 0 {
                    results.extend(block_results);

                    if blocker_face.allegiance != Allegiance::Hero {
                        attack_candidates[blocker_face.allegiance] = false;
                    }
                }
            }

            // Find all hits
            attack_candidates[Allegiance::Hero] = is_player_candidate;
            for target_idx in active_idx + 1..range_cap {
                let target_card = pile[target_idx];
                if !attack_candidates[target_card.get_active_face().allegiance] {
                    continue;
                }

                let state_with_target = state.clone().append_event(Event::AttackCard(
                    target_idx,
                    target_card,
                    HitType::Hit,
                ));

                results.extend(attack_card_get_all_outcomes(
                    &state_with_target,
                    target_idx,
                    HitType::Hit,
                ));
            }
        }
        Action::Arrow => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }
            let start_idx = cmp::max(active_idx + 1, pile.len() - 3);

            for target_idx in (start_idx..pile.len()).rev() {
                let target_card_ptr = pile[target_idx];
                if !(is_allegiance_match(
                    Allegiance::Hero,
                    target_card_ptr.get_active_face().allegiance,
                    wrapped_action.target,
                )) {
                    continue;
                }

                results.extend(attack_card_get_all_outcomes(
                    &state.clone().append_event(Event::AttackCard(
                        target_idx,
                        target_card_ptr,
                        HitType::Arrow,
                    )),
                    target_idx,
                    HitType::Arrow,
                ));
            }
        }
        Action::DoubleArrow => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }
            let start_idx = cmp::max(active_idx + 1, pile.len() - 3);

            let mut arrow_targets = vec![];

            for target_idx in (start_idx..pile.len()).rev() {
                let target_card_ptr = pile[target_idx];
                if is_allegiance_match(
                    Allegiance::Hero,
                    target_card_ptr.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    arrow_targets.push(target_idx);
                }
            }

            for i in 0..arrow_targets.len() {
                let target_idx_1 = arrow_targets[i];
                let base_state = state.clone().append_event(Event::AttackCard(
                    target_idx_1,
                    state.get_pile()[target_idx_1],
                    HitType::Arrow,
                ));

                {
                    for first_arrow_state in
                        attack_card_get_all_outcomes(&base_state, target_idx_1, HitType::Arrow)
                    {
                        for j in 0..arrow_targets.len() {
                            if i == j {
                                continue;
                            }
                            let target_idx_2 = arrow_targets[j];
                            let base_state_2 =
                                first_arrow_state.clone().append_event(Event::AttackCard(
                                    target_idx_2,
                                    state.get_pile()[target_idx_2],
                                    HitType::Arrow,
                                ));

                            for second_arrow_state in attack_card_get_all_outcomes(
                                &base_state_2,
                                target_idx_2,
                                HitType::Arrow,
                            ) {
                                results.push(second_arrow_state);
                            }
                        }

                        // Only use 1 arrow
                        results.push(first_arrow_state.append_event(Event::SkipArrow));
                    }
                }
            }
        }
        Action::Quicken(max_amount) => {
            if is_action_prevented(pile, Features::Web, active_idx) {
                return results;
            }
            for target_idx in active_idx + 2..pile.len() {
                let target_card = pile[target_idx];
                let target_card_allegiance = target_card.get_active_face().allegiance;

                if !is_allegiance_match(
                    Allegiance::Hero,
                    target_card_allegiance,
                    wrapped_action.target,
                ) {
                    continue;
                }

                let max_move_amount = cmp::min(max_amount, target_idx - active_idx - 1);
                let move_results = move_card_by_up_to_amount(
                    &state,
                    target_idx,
                    max_move_amount as i32,
                    MoveType::Quicken,
                );
                results.extend(move_results);
            }
        }
        Action::Delay(max_amount) => {
            if is_action_prevented(pile, Features::Web, active_idx) {
                return results;
            }
            for target_idx in active_idx + 1..pile.len() - 1 {
                let target_card = pile[target_idx];
                let target_card_allegiance = target_card.get_active_face().allegiance;

                if !is_allegiance_match(
                    Allegiance::Hero,
                    target_card_allegiance,
                    wrapped_action.target,
                ) {
                    continue;
                }

                let max_move_amount = cmp::min(max_amount, pile.len() - target_idx - 1);
                let move_results = move_card_by_up_to_amount(
                    &state,
                    target_idx,
                    max_move_amount as i32,
                    MoveType::Delay,
                );
                results.extend(move_results);
            }
        }
        Action::Heal => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }

            for target_idx in active_idx + 1..pile.len() {
                let target_card = pile[target_idx];
                if !is_allegiance_match(
                    Allegiance::Hero,
                    target_card.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    continue;
                }

                if target_card.get_active_face().health != Health::Half {
                    continue;
                }

                let mut new_state = state.clone();
                new_state.get_pile_mut()[target_idx].key = FaceKey::A;
                let new_event = Event::Heal(target_idx, pile[target_idx]);
                results.push(new_state.append_event(new_event));
            }
        }
        Action::Manouver => {
            if is_action_prevented(pile, Features::Venom, active_idx) {
                return results;
            }
            for target_idx in active_idx + 1..pile.len() {
                let target_card_ptr = pile[target_idx];
                if !is_allegiance_match(
                    Allegiance::Hero,
                    target_card_ptr.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    continue;
                }

                let target_face = target_card_ptr.get_active_face();
                let target_health = target_face.health;
                let rotated_key = rotate_key(target_card_ptr.key);
                if target_card_ptr.get_card_def().faces[rotated_key].health <= target_health {
                    let mut new_state = state.clone();
                    new_state.get_pile_mut()[target_idx].key = rotated_key;
                    new_state.mut_append_event(Event::Manouver(target_idx, target_card_ptr));

                    results.push(new_state);
                }
            }
        }
    }

    return results;
}

fn _get_assist_action_outcomes<T: EngineGameState>(
    state: &T,
    active_idx: usize,
    maybe_used_assist_id: Option<CardId>,
) -> Vec<(T, CardId)> {
    let pile = state.get_pile();
    let mut results = vec![];
    for assist_idx in 0..pile.len() {
        if assist_idx == active_idx {
            continue;
        }

        let assist_card = pile[assist_idx];
        let assist_card_id = assist_card.get_card_id();
        if Some(assist_card.get_card_def().id) == maybe_used_assist_id {
            continue;
        }

        let assist_face = assist_card.get_active_face();
        if assist_face.allegiance != Allegiance::Hero {
            continue;
        }

        for (assist_row_idx, assist_option) in assist_face.assists.iter().enumerate() {
            let mut new_state = state
                .clone()
                .append_event(Event::UseActionAssistCard(assist_idx, assist_card))
                .append_event(Event::UseActionAssistRow(
                    assist_idx,
                    assist_card,
                    assist_row_idx,
                ));

            if let Some(self_action) = assist_option.mandatory {
                perform_mandatory_action(&mut new_state, self_action, assist_idx)
            }

            let outcomes = resolve_player_row_post_conditions_no_mandatory(
                &new_state,
                assist_option,
                active_idx,
                &NO_ENERGY_USED,
            );

            for outcome in outcomes {
                results.push((outcome, assist_card_id));
            }
        }
    }

    results
}

fn move_card_by_up_to_amount<T: EngineGameState>(
    state: &T,
    target_idx: usize,
    moves_remaining: i32,
    move_type: MoveType,
) -> Vec<T> {
    assert!(moves_remaining >= 1);
    if state.get_pile()[target_idx]
        .get_active_face()
        .features
        .intersects(Features::Weight)
    {
        return vec![];
    }

    let target_state = state.clone().append_event(Event::MoveTarget(
        target_idx,
        state.get_pile()[target_idx],
        move_type,
    ));

    let mut results_agg = Vec::new();
    _move_card_inner(
        &T::new(state.get_pile().clone()),
        target_idx,
        moves_remaining,
        0,
        0,
        move_type,
        &mut results_agg,
    );

    results_agg
        .into_iter()
        .map(|(distance, moved_state)| {
            let anchor_idx = match move_type {
                MoveType::Quicken => target_idx - distance,
                MoveType::Delay => target_idx + distance,
            };
            let anchor_card = state.get_pile()[anchor_idx];

            let result_prefix = target_state.clone().append_event(Event::MoveBy(
                anchor_idx,
                anchor_card,
                move_type,
                distance,
            ));

            T::combine(result_prefix, moved_state)
        })
        .collect()
}

fn _move_card_inner<T: EngineGameState>(
    state: &T,
    target_idx: usize,
    distance_remaining: i32,
    distance_so_far: usize,
    distance_since_last_event: usize,
    move_type: MoveType,
    results_agg: &mut Vec<(usize, T)>,
) {
    assert!(distance_remaining >= 1);
    let pile = state.get_pile();
    let direction = match move_type {
        MoveType::Delay => 1,
        MoveType::Quicken => -1,
    };
    let swap_with_idx = (target_idx as i32 + direction) as usize;
    let moved_card = pile[target_idx];
    let moved_over_card = pile[swap_with_idx];

    let mut new_state = state.clone();
    new_state.get_pile_mut().swap(target_idx, swap_with_idx);

    if moved_card.get_active_face().allegiance != Allegiance::Hero
        && moved_over_card
            .get_active_face()
            .features
            .intersects(Features::Trap)
    {
        let new_state_with_move = new_state
            .clone()
            .append_event(Event::MoveResult(move_type, distance_since_last_event + 1));
        let hit_options =
            attack_card_get_all_outcomes(&new_state_with_move, swap_with_idx, HitType::Trap);

        if hit_options.len() > 0 {
            for hit_option in hit_options {
                if distance_remaining > 1
                    && !hit_option.get_pile()[swap_with_idx]
                        .get_active_face()
                        .features
                        .intersects(Features::Weight)
                {
                    _move_card_inner(
                        &hit_option,
                        swap_with_idx,
                        distance_remaining - 1,
                        distance_so_far + 1,
                        0,
                        move_type,
                        results_agg,
                    );
                }

                results_agg.push((distance_so_far + 1, hit_option));
            }
            return;
        }
    }

    {
        let final_state = new_state
            .clone()
            .append_event(Event::MoveResult(move_type, distance_since_last_event + 1));
        results_agg.push((distance_so_far + 1, final_state));
    }

    if distance_remaining > 1 {
        _move_card_inner(
            &new_state,
            swap_with_idx,
            distance_remaining - 1,
            distance_so_far + 1,
            distance_since_last_event + 1,
            move_type,
            results_agg,
        );
    }
}

// Enemy Only
fn resolve_enemy_turn<T: EngineGameState>(
    pile: &T,
    allegiance: Allegiance,
    active_idx: usize,
) -> Vec<T> {
    let swarm_states = swarm_me_recursive(pile, allegiance, active_idx + 1);
    if swarm_states.len() > 0 {
        let mut results = vec![];
        for swarm_state in swarm_states {
            for child_state in resolve_enemy_turn_no_swarm(&swarm_state, allegiance, active_idx) {
                results.push(child_state);
            }
        }
        results
    } else {
        resolve_enemy_turn_no_swarm(pile, allegiance, active_idx)
    }
}

fn swarm_me_recursive<T: EngineGameState>(
    state: &T,
    allegiance: Allegiance,
    active_idx: usize,
) -> Vec<T> {
    let pile = state.get_pile();
    if let Some(active_card) = pile.get(active_idx) {
        let active_face = active_card.get_active_face();
        if active_card.get_active_face().allegiance == allegiance {
            if let Some(swarm_row) = &active_face.swarm {
                let mut child_states = swarm_me_recursive(state, allegiance, active_idx + 1);
                if child_states.len() == 0 {
                    child_states = vec![state.clone()];
                }

                let mut result_states = vec![];
                for mut base_state in child_states {
                    base_state.mut_append_event(Event::Swarm(
                        active_idx,
                        base_state.get_pile()[active_idx],
                    ));
                    let new_states =
                        resolve_enemy_row(&base_state, allegiance, &swarm_row, active_idx, true);
                    if new_states.len() == 0 {
                        result_states.push(base_state);
                    } else {
                        result_states.extend(new_states)
                    }
                }
                return result_states;
            }
        }
    }
    return vec![];
}

fn resolve_enemy_turn_no_swarm<T: EngineGameState>(
    state: &T,
    allegiance: Allegiance,
    active_idx: usize,
) -> Vec<T> {
    let pile = state.get_pile();
    let active_card = &pile[active_idx];
    let active_face = &active_card.get_active_face();

    for (row_idx, row) in active_face.rows.iter().enumerate() {
        let state_with_row_idx = state.clone().append_event(Event::PickRow(
            row_idx,
            active_idx,
            state.get_pile()[active_idx],
        ));
        let row_outcomes =
            resolve_enemy_row(&state_with_row_idx, allegiance, &row, active_idx, false);

        if row_outcomes.len() > 0 {
            return row_outcomes;
        }
    }

    // If no rows were taken we just skip instead
    return vec![state.clone().append_event(Event::SkipTurn(*active_card))];
}

fn resolve_enemy_row<T: EngineGameState>(
    state: &T,
    allegiance: Allegiance,
    row: &Row,
    active_idx: usize,
    force_mandatory: bool,
) -> Vec<T> {
    let pile = state.get_pile();
    if let Some(condition) = row.condition {
        match condition {
            Condition::Energy(_) => {
                panic!("Didn't expect to use energy for enemy row")
            }
            Condition::ExhaustedAllies(required_amount) => {
                let revive_targets = find_all_revive_targets(pile, allegiance, active_idx + 1);

                if revive_targets.len() < required_amount {
                    return vec![];
                }
            }
            Condition::Rage(required_amount) => {
                let mut total_rage = 0;
                for i in active_idx + 1..pile.len() {
                    let other_card = pile[i];
                    if other_card.get_active_face().allegiance == allegiance {
                        total_rage += other_card.get_active_face().rage;
                    }
                }
                if total_rage < required_amount {
                    return vec![];
                }
            }
        }
    }

    let mut active_states = vec![state.clone()];
    let mut did_any_actions = false;

    for action in &row.actions {
        let mut next_active_states: Vec<T> = vec![];
        for current_state in &active_states {
            let new_states = resolve_enemy_action(current_state, allegiance, action, active_idx);

            if new_states.len() > 0 {
                did_any_actions = true;
                next_active_states.extend(new_states);
            } else {
                next_active_states.push(
                    current_state
                        .clone()
                        .append_event(Event::SkipAction(state.get_pile()[active_idx], *action)),
                );
            }
        }
        active_states = next_active_states;
    }

    if !did_any_actions && !force_mandatory {
        return vec![];
    }

    for result_state in &mut active_states {
        if let Some(self_action) = row.mandatory {
            perform_mandatory_action(result_state, self_action, active_idx);
        }
    }

    active_states
}

fn resolve_enemy_action<T: EngineGameState>(
    state_no_event: &T,
    allegiance: Allegiance,
    wrapped_action: &WrappedAction,
    active_idx: usize,
) -> Vec<T> {
    // let state = state_no_event.clone().append_event(Event::StartAction(
    //     state_no_event.get_pile()[active_idx],
    //     *wrapped_action,
    // ));
    let state = state_no_event.clone();
    let pile = state.get_pile();
    let mut results: Vec<T> = vec![];

    match wrapped_action.action {
        Action::Arrow
        | Action::DoubleArrow
        | Action::Manouver
        | Action::Quicken(_)
        | Action::Delay(_)
        | Action::Fireball
        | Action::Ablaze
        | Action::Teleport
        | Action::CallAssist
        | Action::CallAssistTwice => {
            panic!(
                "Action not implemented for enemy: {:?}",
                wrapped_action.action
            );
        }
        Action::Hit(range) => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Venom, active_idx)
            {
                return results;
            }

            let range_cap = match range {
                Range::Inf => pile.len(),
                Range::Int(amount) => cmp::min(pile.len(), active_idx + amount + 1),
            };

            for target_idx in active_idx + 1..range_cap {
                let target_card = pile[target_idx];
                let target_card_allegiance = target_card.get_active_face().allegiance;
                if !is_allegiance_match(allegiance, target_card_allegiance, wrapped_action.target) {
                    continue;
                }
                if target_card.get_active_face().health == Health::Empty {
                    continue;
                }

                let state_with_target = state.clone().append_event(Event::AttackCard(
                    target_idx,
                    target_card,
                    HitType::Hit,
                ));
                let blockers_results = find_blockers_for_hit_outcomes(
                    &state_with_target,
                    active_idx,
                    range_cap,
                    target_idx,
                );

                let num_blockers_results = blockers_results.len();
                results.extend(blockers_results);

                // Hit the card if either we didn't block, or character has agency
                if num_blockers_results == 0 || target_card_allegiance == Allegiance::Hero {
                    results.extend(attack_card_get_all_outcomes(
                        &state_with_target,
                        target_idx,
                        HitType::Hit,
                    ));
                    break;
                }

                if num_blockers_results > 0 {
                    break;
                }
            }
        }
        Action::Claws(range) => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Venom, active_idx)
            {
                return results;
            }
            let range_cap = match range {
                Range::Inf => pile.len(),
                Range::Int(amount) => cmp::min(active_idx + amount + 1, pile.len()),
            };

            let states = attack_all_in_range(
                &state,
                allegiance,
                active_idx + 1,
                range_cap,
                wrapped_action.target,
                HitType::Claw,
            );

            results.extend(states)
        }
        Action::SpacedClaws(space_type) => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Venom, active_idx)
            {
                return results;
            }
            let start_idx = match space_type {
                ClawSpaceType::Odd => active_idx + 1,
                ClawSpaceType::Even => active_idx + 2,
            };

            let mut state_agg = vec![state.clone()];

            for target_idx in (start_idx..pile.len()).step_by(2).rev() {
                let target_card = pile[target_idx];
                if !is_allegiance_match(
                    allegiance,
                    target_card.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    continue;
                }

                state_agg = attack_idx_in_all_states(state_agg, target_idx, HitType::Claw);
            }

            if did_states_change(&state_agg, &state) {
                results.extend(state_agg)
            }
        }
        Action::Void => {
            for target_idx in active_idx + 1..pile.len() {
                let target_card = pile[target_idx];
                if is_allegiance_match(
                    allegiance,
                    target_card.get_active_face().allegiance,
                    wrapped_action.target,
                ) && target_card.get_active_face().health != Health::Empty
                {
                    for exhausted_key in exhaust_card(&pile[target_idx]) {
                        let mut new_state = state.clone();
                        new_state.get_pile_mut()[target_idx].key = exhausted_key;
                        results.push(new_state.append_event(Event::Void(
                            target_idx,
                            target_card,
                            exhausted_key,
                        )));
                    }
                    break;
                }
            }
        }
        Action::Death => {
            let mut new_state = state.clone();
            for card in new_state.get_pile_mut().iter_mut() {
                if is_allegiance_match(
                    allegiance,
                    card.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    exhaust_card_no_options(card);
                }
            }
            results.push(new_state.append_event(Event::Death));
        }
        Action::Pull(range) => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Web, active_idx)
            {
                return results;
            }
            let max_range = match range {
                Range::Inf => pile.len(),
                Range::Int(r) => cmp::min(active_idx + r + 1, pile.len()),
            };

            for target_idx in (active_idx + 2..max_range).rev() {
                let target_card = pile[target_idx];
                let target_card_allegiance = target_card.get_active_face().allegiance;

                if !is_allegiance_match(allegiance, target_card_allegiance, wrapped_action.target) {
                    continue;
                }

                if target_card_allegiance != allegiance
                    && (
                        // Enemies must have health in order to pull
                        target_card.get_active_face().health == Health::Empty
                        // Enemies can't be heavy in order to pull
                             || target_card
                                 .get_active_face()
                                 .features
                                 .intersects(Features::Weight)
                    )
                {
                    continue;
                }

                let dodge_outcomes =
                    try_prevent_action_with_reaction(&state, target_idx, ReactionTrigger::Dodge);
                results.extend(dodge_outcomes);

                {
                    let target = state.get_pile()[target_idx];

                    let mut pull_results = vec![];
                    move_card_to_end(
                        &mut state.clone().append_event(Event::Pull(target_idx, target)),
                        active_idx,
                        target_idx,
                        &mut pull_results,
                        EndPileMoveType::Pull,
                    );

                    results.extend(pull_results);

                    break;
                }
            }
        }
        Action::Push(range) => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Web, active_idx)
            {
                return results;
            }
            let max_range = match range {
                Range::Inf => pile.len() - 1,
                Range::Int(r) => cmp::min(active_idx + r + 1, pile.len() - 1),
            };

            for target_idx in active_idx + 1..max_range {
                let target_card = pile[target_idx];
                let target_card_allegiance = target_card.get_active_face().allegiance;

                if !is_allegiance_match(allegiance, target_card_allegiance, wrapped_action.target) {
                    continue;
                }

                if target_card_allegiance != allegiance
                    && (
                        // Enemies can't be heavy in order to push
                        target_card
                            .get_active_face()
                            .features
                            .intersects(Features::Weight)
                    )
                {
                    continue;
                }

                let dodge_outcomes =
                    try_prevent_action_with_reaction(&state, target_idx, ReactionTrigger::Dodge);
                results.extend(dodge_outcomes);

                {
                    let target = state.get_pile()[target_idx];

                    let mut pull_results = vec![];
                    move_card_to_end(
                        &mut state.clone().append_event(Event::Push(target_idx, target)),
                        active_idx,
                        target_idx,
                        &mut pull_results,
                        EndPileMoveType::Push,
                    );

                    results.extend(pull_results);
                    break;
                }
            }
        }
        Action::Heal => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Venom, active_idx)
            {
                return results;
            }
            let maybe_target = find_heal_target(
                pile,
                Health::Half,
                allegiance,
                wrapped_action.target,
                active_idx + 1,
            );
            if let Some(target) = maybe_target {
                let mut new_state = state.clone();
                let new_event = Event::Heal(target, pile[target]);
                new_state.get_pile_mut()[target].key = FaceKey::A;
                results.push(new_state.append_event(new_event));
            }
        }
        Action::Revive => {
            if allegiance == Allegiance::Werewolf
                && is_action_prevented(pile, Features::Venom, active_idx)
            {
                return results;
            }
            let maybe_target = find_heal_target(
                pile,
                Health::Empty,
                allegiance,
                wrapped_action.target,
                active_idx + 1,
            );
            if let Some(target) = maybe_target {
                let mut new_state = state.clone();
                let new_event = Event::Revive(target, pile[target]);
                new_state.get_pile_mut()[target].key = FaceKey::A;
                results.push(new_state.append_event(new_event));
            }
        }
        Action::Inspire => {
            for target_idx in active_idx + 1..pile.len() {
                let target_card = pile[target_idx];
                if is_allegiance_match(
                    allegiance,
                    target_card.get_active_face().allegiance,
                    wrapped_action.target,
                ) {
                    let inspire_event = Event::Inspire(target_idx, pile[target_idx]);
                    let state_with_inspire_event = state.clone().append_event(inspire_event);
                    results.extend(resolve_card_at_index(&state_with_inspire_event, target_idx));
                    break;
                }
            }
        }
    }

    return results;
}

// Applicators

fn perform_mandatory_action<T: EngineGameState>(
    state: &mut T,
    self_action: SelfAction,
    active_idx: usize,
) {
    perform_card_self_action(self_action, &mut state.get_pile_mut()[active_idx]);
    state.mut_append_event(Event::Mandatory(state.get_pile()[active_idx], self_action));
}

fn try_prevent_action_with_reaction<T: EngineGameState>(
    state: &T,
    target_idx: usize,
    trigger: ReactionTrigger,
) -> Vec<T> {
    let pile = state.get_pile();
    let target_card = pile[target_idx];
    let target_face = target_card.get_active_face();
    if let Some(reaction) = target_face.reaction {
        match reaction {
            Reaction::Standard(standard_reaction) => {
                if standard_reaction.trigger == trigger {
                    return vec![get_standard_reaction_results(
                        state,
                        target_idx,
                        standard_reaction,
                    )];
                }
            }
            Reaction::Assist(request_assist) => {
                for assist_idx in 0..pile.len() {
                    let assist_card = pile[assist_idx];
                    let target_allegiance = target_face.allegiance;
                    let assist_face = assist_card.get_active_face();
                    if assist_face.allegiance != target_allegiance {
                        continue;
                    }

                    if let Some(reaction_assist_option) = assist_face.reaction_assist {
                        if reaction_assist_option.trigger == trigger {
                            return get_reaction_assist_results(
                                state,
                                target_idx,
                                assist_idx,
                                reaction_assist_option,
                                request_assist.outcome,
                            );
                        }
                    }
                }
            }
            Reaction::WhenHit(_) => (), // Don't
        }
    }
    vec![]
}

fn get_reaction_assist_results<T: EngineGameState>(
    initial_state: &T,
    assist_user_idx: usize,
    assist_provider_idx: usize,
    assist: ProvideAssistReaction,
    assist_outcome: Option<SelfAction>,
) -> Vec<T> {
    // Pay the cost of the react assist
    let mut react_cost_state = initial_state.clone();
    perform_card_self_action(
        assist.assist_cost,
        &mut react_cost_state.get_pile_mut()[assist_provider_idx],
    );
    react_cost_state.mut_append_event(Event::ReactAssistUsed(
        assist_provider_idx,
        initial_state.get_pile()[assist_provider_idx],
        assist.trigger,
        assist.assist_cost,
    ));
    react_cost_state = get_standard_reaction_results(
        &react_cost_state,
        assist_user_idx,
        StandardReaction {
            trigger: assist.trigger,
            outcome: assist_outcome,
        },
    );

    // Try doing another assist action
    let mut final_results: Vec<T> = _get_assist_action_outcomes(
        &react_cost_state,
        assist_user_idx,
        Some(react_cost_state.get_pile()[assist_provider_idx].get_card_id()),
    )
    .into_iter()
    .map(|(s, _)| s)
    .collect();

    // Skip the other assist action
    final_results.push(react_cost_state.append_event(Event::SkipReactActionAssist));

    final_results
}

fn attack_idx_in_all_states<T: EngineGameState>(
    states: Vec<T>,
    target_idx: usize,
    hit_type: HitType,
) -> Vec<T> {
    let mut results = vec![];
    for state in states {
        let hit_outcomes = attack_card_get_all_outcomes(&state, target_idx, hit_type);

        if hit_outcomes.len() > 0 {
            for outcome in hit_outcomes {
                results.push(outcome);
            }
        } else {
            results.push(state);
        }
    }

    results
}

fn attack_all_in_range<T: EngineGameState>(
    state: &T,
    attacker_allegiance: Allegiance,
    start_idx_inclusive: usize,
    end_idx_exclusive: usize,
    target: Target,
    hit_type: HitType,
) -> Vec<T> {
    let pile = state.get_pile();
    let mut state_agg = vec![state.clone()];

    for target_idx in (start_idx_inclusive..end_idx_exclusive).rev() {
        let target_card = pile[target_idx];
        if !is_allegiance_match(
            attacker_allegiance,
            target_card.get_active_face().allegiance,
            target,
        ) {
            continue;
        }

        state_agg = attack_idx_in_all_states(state_agg, target_idx, hit_type);
    }

    if did_states_change(&state_agg, &state) {
        state_agg
    } else {
        vec![]
    }
}

fn attack_card_get_all_outcomes<T: EngineGameState>(
    state: &T,
    target_idx: usize,
    hit_type: HitType,
) -> Vec<T> {
    let mut results: Vec<T> = vec![];

    let pile = state.get_pile();
    let target_card = pile[target_idx];
    let target_face = target_card.get_active_face();
    let target_allegiance = target_face.allegiance;
    let is_reaction_forced = target_allegiance != Allegiance::Hero;

    if let Some(reaction) = target_face.reaction {
        match reaction {
            Reaction::Standard(standard_reaction) => {
                results.push(get_standard_reaction_results(
                    state,
                    target_idx,
                    standard_reaction,
                ));
            }
            Reaction::Assist(assist_reaction) => {
                for assist_idx in 0..pile.len() {
                    let assist_card = pile[assist_idx];
                    let assist_face = assist_card.get_active_face();
                    if assist_face.allegiance != target_allegiance {
                        continue;
                    }

                    if let Some(reaction_assist_option) = assist_face.reaction_assist {
                        results.extend(get_reaction_assist_results(
                            state,
                            target_idx,
                            assist_idx,
                            reaction_assist_option,
                            assist_reaction.outcome,
                        ));
                    }
                }
            }
            Reaction::WhenHit(row) => {
                let reaction_results = resolve_enemy_row(
                    &state
                        .clone()
                        .append_event(Event::OnHurt(target_idx, target_card)),
                    target_face.allegiance,
                    &row,
                    target_idx,
                    true,
                );

                for reaction_result in reaction_results {
                    let post_hurt_results =
                        hurt_card_get_all_outcomes(&reaction_result, target_idx, hit_type);
                    for hurt_result in post_hurt_results {
                        results.push(hurt_result);
                    }
                }
            }
        }

        if results.len() > 0 && is_reaction_forced {
            return results;
        }
    }

    results.extend(hurt_card_get_all_outcomes(state, target_idx, hit_type));

    results
}

fn get_standard_reaction_results<T: EngineGameState>(
    state: &T,
    target_idx: usize,
    standard_reaction: StandardReaction,
) -> T {
    let mut new_state = state.clone();
    if let Some(self_action) = standard_reaction.outcome {
        perform_card_self_action(self_action, &mut new_state.get_pile_mut()[target_idx]);
    }
    let event = match standard_reaction.trigger {
        ReactionTrigger::Block => Event::Block(
            target_idx,
            state.get_pile()[target_idx],
            standard_reaction.outcome,
        ),
        ReactionTrigger::Dodge => Event::Dodge(
            target_idx,
            state.get_pile()[target_idx],
            standard_reaction.outcome,
        ),
    };
    new_state.mut_append_event(event);

    new_state
}

fn find_blockers_for_hit_outcomes<T: EngineGameState>(
    state: &T,
    active_idx: usize,
    range_cap: usize,
    attack_target_idx: usize,
) -> Vec<T> {
    let mut results = vec![];
    let pile = state.get_pile();
    let victim_allegiance = pile[attack_target_idx].get_active_face().allegiance;

    for blocker_idx in (active_idx + 1..range_cap).rev() {
        if blocker_idx == attack_target_idx {
            continue;
        }

        let blocker_card_ptr = &pile[blocker_idx];
        if blocker_card_ptr.get_active_face().allegiance == victim_allegiance {
            let block_results =
                try_prevent_action_with_reaction(state, blocker_idx, ReactionTrigger::Block);
            if block_results.len() > 0 {
                results.extend(block_results);

                if victim_allegiance != Allegiance::Hero {
                    return results;
                }
            }
        }
    }

    results
}

fn hurt_card_get_all_outcomes<T: EngineGameState>(
    state: &T,
    target_idx: usize,
    hit_type: HitType,
) -> Vec<T> {
    let mut results = vec![];
    let pile = state.get_pile();
    let target_card = pile[target_idx];
    for hurt_key in find_hurt_faces(&target_card) {
        let mut new_state = state.clone();
        new_state.get_pile_mut()[target_idx].key = hurt_key;
        let event = Event::Damage(target_idx, pile[target_idx], hit_type, hurt_key);
        results.push(new_state.append_event(event))
    }
    results
}

fn move_card_to_end<T: EngineGameState>(
    state: &mut T,
    active_idx: usize,
    mut target_idx: usize,
    mut results_agg: &mut Vec<T>,
    move_type: EndPileMoveType,
) {
    let direction = match move_type {
        EndPileMoveType::Push => 1,
        EndPileMoveType::Pull => -1,
    };

    let mut did_move = false;
    loop {
        let swap_with_idx = (target_idx as i32 + direction) as usize;
        if swap_with_idx <= active_idx || swap_with_idx >= state.get_pile().len() {
            if did_move {
                state.mut_append_event(Event::EndPileMoveResult(move_type))
            }
            results_agg.push(state.clone());
            return;
        }
        did_move = true;

        let moved_card = state.get_pile()[target_idx];
        let moved_over_card = state.get_pile()[swap_with_idx];
        state.get_pile_mut().swap(target_idx, swap_with_idx);

        target_idx = swap_with_idx;

        if moved_card.get_active_face().allegiance != Allegiance::Hero
            && moved_over_card
                .get_active_face()
                .features
                .intersects(Features::Trap)
        {
            let new_state_with_move = state
                .clone()
                .append_event(Event::EndPileMoveResult(move_type));
            let hit_options =
                attack_card_get_all_outcomes(&new_state_with_move, swap_with_idx, HitType::Trap);

            if hit_options.len() > 0 {
                for mut hit_option in hit_options {
                    move_card_to_end(
                        &mut hit_option,
                        active_idx,
                        target_idx,
                        &mut results_agg,
                        move_type,
                    )
                }
                return;
            }
        }
    }
}

fn bottom_top_card<T: EngineGameState>(state: &mut T) {
    state.get_pile_mut().rotate_left(1);
    state.mut_append_event(Event::BottomCard);
}

// Utils
pub fn is_game_winner(pile: &Pile) -> Option<Allegiance> {
    let mut player_wins = true;
    let mut enemy_wins = true;

    for card in pile.iter() {
        let active_face = card.get_active_face();
        if active_face.health != Health::Empty {
            match active_face.allegiance {
                Allegiance::Hero => {
                    enemy_wins = false;
                    if !player_wins {
                        return None;
                    }
                }
                Allegiance::Baddie => {
                    player_wins = false;
                    if !enemy_wins {
                        return None;
                    }
                }
                Allegiance::Werewolf => (),
            }
        }
    }

    if player_wins {
        Some(Allegiance::Hero)
    } else if enemy_wins {
        Some(Allegiance::Baddie)
    } else {
        None
    }
}

pub fn perform_card_self_action(self_action: SelfAction, active_card: &mut CardPtr) {
    match self_action {
        SelfAction::Rotate => {
            active_card.key = rotate_key(active_card.key);
        }
        SelfAction::Flip => {
            active_card.key = flip_key(active_card.key);
        }
    }
}

fn is_allegiance_match(me: Allegiance, other: Allegiance, spec: Target) -> bool {
    match spec {
        Target::Any => true,
        Target::Ally => me == other,
        Target::Enemy => me != other,
    }
}

fn find_heal_target(
    pile: &Pile,
    target_health: Health,
    allegiance: Allegiance,
    target_type: Target,
    starting_idx: usize,
) -> Option<usize> {
    let max_range = pile.len();

    for i in usize::from(starting_idx)..max_range {
        let active_card_ptr = &pile[i];
        if is_allegiance_match(
            allegiance,
            active_card_ptr.get_active_face().allegiance,
            target_type,
        ) {
            if active_card_ptr.get_active_face().health == target_health {
                return Some(i);
            }
        }
    }

    None
}

fn find_all_revive_targets(pile: &Pile, allegiance: Allegiance, starting_idx: usize) -> Vec<usize> {
    let mut result = Vec::new();

    for i in usize::from(starting_idx)..pile.len() {
        let active_card_ptr = &pile[i];
        if active_card_ptr.get_active_face().allegiance == allegiance {
            if active_card_ptr.get_active_face().health == Health::Empty {
                result.push(i);
            }
        }
    }

    result
}

// Optimization: enum array?
// Optimization: convert to lookup table?
fn find_hurt_faces(card: &CardPtr) -> Vec<FaceKey> {
    let card_def = card.get_card_def();
    let current_health = card_def.faces[card.key].health;
    if current_health == Health::Empty {
        return vec![];
    }

    let mut results = vec![];
    let target_health = one_damage(current_health);
    for key in FaceKey::iter() {
        if key == card.key {
            continue;
        }

        if card_def.faces[key].health == target_health {
            results.push(key);
        }
    }

    results
}

fn one_damage(health: Health) -> Health {
    match health {
        Health::Full => Health::Half,
        Health::Half => Health::Empty,
        Health::Empty => {
            panic!("Tried to hurt card with no health")
        }
    }
}

fn rotate_key(key: FaceKey) -> FaceKey {
    match key {
        FaceKey::A => FaceKey::B,
        FaceKey::B => FaceKey::A,
        FaceKey::C => FaceKey::D,
        FaceKey::D => FaceKey::C,
    }
}

fn flip_key(key: FaceKey) -> FaceKey {
    match key {
        FaceKey::A => FaceKey::C,
        FaceKey::C => FaceKey::A,
        FaceKey::B => FaceKey::D,
        FaceKey::D => FaceKey::B,
    }
}

fn exhaust_card(card: &CardPtr) -> Vec<FaceKey> {
    if card.get_active_face().health == Health::Empty {
        panic!("Card is already exhausted")
    }

    let mut results = vec![];
    for key in FaceKey::iter() {
        if card.get_card_def().faces[key].health == Health::Empty {
            results.push(key)
        }
    }
    if results.len() == 0 {
        panic!("Could not find exhausted face of card");
    }

    return results;
}

fn exhaust_card_no_options(card: &mut CardPtr) {
    if card.get_active_face().health == Health::Empty {
        return;
    }

    for key in FaceKey::iter() {
        if card.get_card_def().faces[key].health == Health::Empty {
            card.key = key;
            return;
        }
    }
    panic!("Could not find exhausted face of card");
}

fn is_action_prevented(pile: &Pile, feature: Features, active_idx: usize) -> bool {
    if let Some(infront) = pile.get(active_idx + 1) {
        infront.get_active_face().features.intersects(feature)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{string_to_card_ptr, string_to_pile};
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use std::fmt::Debug;

    type T = GameStateWithEventLog;

    #[allow(dead_code)]
    fn pprint<U>(thing: &Vec<U>)
    where
        U: Debug,
    {
        for s in thing {
            println!("{:?}", s)
        }
    }

    fn states_to_pile_set(game_states: &Vec<T>) -> HashSet<Pile> {
        let mut result: HashSet<Pile> = HashSet::new();

        for state in game_states {
            result.insert(state.get_pile().clone());
        }

        result
    }

    fn assert_actual_vs_expected_piles(actual_results: &Vec<T>, expected_strings: Vec<&str>) {
        let actual_piles: HashSet<Pile> = states_to_pile_set(&actual_results);
        let expected_piles: HashSet<Pile> =
            HashSet::from_iter(expected_strings.iter().map(|x| string_to_pile(x)));

        assert_eq!(actual_piles, expected_piles)
    }

    #[test]
    fn test_bug1() {
        // The bug was that 9D was listed as always blocking
        let pile = string_to_pile("2A 9D");

        let new_states = resolve_player_action(
            &GameStateWithEventLog::new(pile),
            &WrappedAction {
                action: Action::Hit(Range::Int(1)),
                target: Target::Any,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "2 9D", // Skip
                "2 9C", // Hit
            ],
        );
    }

    #[test]
    fn test_bug2() {
        // 2A is not rotating after perfoming its row0 attack
        let pile = string_to_pile("2A 9D");
        let new_states =
            resolve_player_row(&T::new(pile.clone()), &pile[0].get_active_face().rows[0], 0);
        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "2B 9D", // Skip
                "2B 9C", // Hit
            ],
        )
    }

    #[test]
    fn test_bug3() {
        // Enemies pulling doesn't seem to work
        let pile = string_to_pile("6A 1 2 3 4 5");
        let test_for_range = |range: Range, expected_str: &str| {
            let new_states = resolve_enemy_action(
                &T::new(pile.clone()),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(range),
                    target: Target::Enemy,
                },
                0,
            );
            assert_actual_vs_expected_piles(&new_states, vec![expected_str]);
        };

        test_for_range(Range::Inf, "6A 5 1 2 3 4");
        test_for_range(Range::Int(5), "6A 5 1 2 3 4");
        test_for_range(Range::Int(4), "6A 4 1 2 3 5");
        test_for_range(Range::Int(3), "6A 3 1 2 4 5");
        test_for_range(Range::Int(2), "6A 2 1 3 4 5");
        test_for_range(Range::Int(10), "6A 5 1 2 3 4");

        {
            // Pull 1
            let result = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Int(1)),
                    target: Target::Enemy,
                },
                0,
            );
            assert_eq!(result.len(), 0);
        }
    }

    #[test]
    fn test_bug4() {
        // Not showing pull action
        let pile = string_to_pile("6A 1 2 3 4A");
        let row = &pile[0].get_active_face().rows[0];
        let new_states =
            resolve_enemy_row(&T::new(pile.clone()), Allegiance::Baddie, row, 0, false);

        assert_eq!(new_states.len(), 2);

        for state in new_states {
            assert!(state
                .events
                .contains(&Event::Pull(4, string_to_card_ptr("4A"))));
        }
    }
    #[test]
    fn test_bug5() {
        // Not showing pull action
        let pile = string_to_pile("p2A, o9C, p4B, p3B, p1A, o8B, o7C, p5D, o6B");
        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Delay(2),
                target: Target::Any,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "2A 9C 4B 3B 1A 8B 7C 5D 6B", // Skip
                "2A 9C 3B 4B 1A 8B 7C 5D 6B", // Delay(2, p4B, 1)
                "2A 9C 3B 1A 4B 8B 7C 5D 6B", // Delay(2, p4B, 2)
                "2A 9C 4B 1A 3B 8B 7C 5D 6B", // Delay(3, p3B, 1)
                "2A 9C 4B 1A 8B 3B 7C 5D 6B", // Delay(3, p3B, 2)
                "2A 9C 4B 3B 8B 1A 7C 5D 6B", // Delay(4, p1A, 1)
                "2A 9C 4B 3B 8B 7C 1A 5D 6B", // Delay(4, p1A, 2)
                "2A 9C 4B 3B 1A 8B 5D 7C 6B", // Delay(6, o7C, 1)
                "2A 9C 4B 3B 1A 8B 5D 6B 7C", // Delay(6, o7C, 2)
                "2A 9C 4B 3B 1A 8B 7C 6B 5D", // Delay(7, p5D, 1)
            ],
        );
    }

    #[test]
    fn test_bug6() {
        // Don't remember what was wrong here
        let pile = string_to_pile("9A 6A 3A 7A 8A 1A 4A 5A 2A");
        let new_states = resolve_top_card(&T::new(pile));

        assert_actual_vs_expected_piles(&new_states, vec!["6A 7A 8A 1A 4A 5A 2A 3A 9B"]);
    }

    #[test]
    fn test_bug7() {
        // When an enemy can't make an action, their other actions are ignored
        let pile = string_to_pile("9A 7A 8A 6B 4A 5A 2A 1D 3B");
        let new_states = resolve_top_card(&T::new(pile.clone()));

        assert_actual_vs_expected_piles(&new_states, vec!["7A 8A 6B 4A 5A 2A 1D 3B 9A"]);
        assert_eq!(
            new_states[0].events,
            vec![Event::SkipTurn(pile[0]), Event::BottomCard]
        )
    }

    #[test]
    fn test_bug8() {
        // Didn't allow moving 0 health unit through traps, since the attack would fizzle
        let pile = string_to_pile("10C 18C 14C");
        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Delay(1),
                target: Target::Any,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "10C 18C 14C", // Skip
                "10C 14C 18C", // Move
            ],
        );
    }

    #[test]
    fn test_bug9() {
        // Block and attack both send 4 to D? Block should send it to B. UI bug only
        let pile = string_to_pile("5A 4A 8A");
        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Hit(Range::Int(1)),
                target: Target::Any,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "5A 4A 8A", // Skip
                "5A 4B 8A", // Block
                "5A 4C 8A", // Hit
            ],
        );
    }

    #[test]
    fn test_bug10() {
        {
            // 6A pulls first, and player has a choice to dodge or not dodge
            // Bug was that when the player dodged, 6As attack would fizzle, and
            // the engine would count that as a failed row, thus not allowing it
            // Fixed by accounting for skipped actions per-state, instead of checking if any
            // actions fizzled
            let state = T::new(string_to_pile("6A 11D 14A"));
            let new_states = resolve_enemy_row(
                &state,
                Allegiance::Baddie,
                &state.pile[0].get_active_face().rows[0],
                0,
                false,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "6B, 11D, 14B", // Dodge the pull
                    "6B, 14B, 11D", // Dodge the attack
                    "6B, 14C, 11D", // Get hit
                ],
            );
        }
    }

    #[test]
    fn test_bug11() {
        {
            // Werewolf was able to pull & push heavy monsters
            let state = T::new(string_to_pile("32C 9A 30A 8A 7A"));
            let new_states = resolve_enemy_row(
                &state,
                Allegiance::Werewolf,
                &state.pile[0].get_active_face().rows[0],
                0,
                false,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "32D 8C 9D 30A 7A", // Dodge the pull
                ],
            );
        }
    }

    #[test]
    fn test_partial_row_fizzle() {
        // Just another case of bug10
        let state = T::new(string_to_pile("33C 1D"));
        let new_states = resolve_enemy_row(
            &state,
            Allegiance::Baddie,
            &state.pile[0].get_active_face().rows[2],
            0,
            false,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "33D 1C", // 1 Gets hit 1 time
            ],
        );
    }

    #[test]
    fn test_game_over() {
        {
            let pile = string_to_pile("6D 3C 2C 5D 8C 1C 4D 7C 9C");
            assert_eq!(is_game_winner(&pile), Some(Allegiance::Baddie));
        }

        {
            let pile = string_to_pile("6C 3C 2C 5D 8C 1C 4D 7C 9C");
            assert_eq!(is_game_winner(&pile), Some(Allegiance::Hero));
        }
    }

    #[test]
    fn test_player_hit_basic() {
        let pile = string_to_pile("4C 1A 8A 9A");
        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Hit(Range::Inf),
                target: Target::Any,
            },
            0,
            &vec![],
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "4C 1A 8A 9A", // Skip
                "4C 1D 8A 9A", // Hit 1
                "4C 1A 8D 9A", // Hit 8
                "4C 1A 8A 9D", // Hit 9
            ],
        );
    }

    #[test]
    fn test_player_hit_enemy_blocker() {
        {
            let pile = string_to_pile("4C 1A 8A 6B 9A");
            let new_states = resolve_player_action(
                &T::new(pile),
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Any,
                },
                0,
                &vec![],
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "4C 1A 8A 6B 9A", // Skip
                    "4C 1D 8A 6B 9A", // Hit 1
                    "4C 1A 8A 6A 9A", // 6 Blocks
                ],
            );
        }
    }

    #[test]
    fn test_player_hit_enemy_many_blockers() {
        {
            let pile = string_to_pile("4C 6B 9B 8A");
            let new_states = resolve_player_action(
                &T::new(pile),
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Any,
                },
                0,
                &vec![],
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "4C 6B 9B 8A", // Skip
                    "4C 6B 9A 8A", // 9 Blocks
                                   // Can only hit furthest back blocker
                ],
            );
        }
    }

    #[test]
    fn test_player_hit_player_blocker() {
        {
            let pile = string_to_pile("4C 1A 2B 8A 6B");
            let new_states = resolve_player_action(
                &T::new(pile),
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Any,
                },
                0,
                &vec![],
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "4C 1A 2B 8A 6B", // Skip
                    "4C 1D 2B 8A 6B", // Hit 1
                    "4C 1A 2A 8A 6B", // 2 Blocks
                    "4C 1A 2D 8A 6B", // Hit 2
                    "4C 1A 2B 8A 6A", // 6 Blocks
                ],
            );
        }
    }

    #[test]
    fn test_werewolf_hits() {
        {
            let pile = string_to_pile("8A 29C 28A");
            let new_states = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            // Werewolf blocks the hit
            assert_actual_vs_expected_piles(&new_states, vec!["8A 29D 28A"])
        }

        {
            let pile = string_to_pile("8A 28A 29C");
            let new_states = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            // Werewolf doesn't block for the hero card
            assert_actual_vs_expected_piles(&new_states, vec!["8A 28D 29C", "8A 28C 29C"])
        }

        {
            // Furthest back werewolf blocks for the other
            let pile = string_to_pile("8A 28C 29C 30A");
            let new_states = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            // Furthest back werewolf should block
            assert_actual_vs_expected_piles(&new_states, vec!["8A 28C 29D 30A"])
        }

        {
            // Furthest back werewolf blocks for the other, for hero attacks too
            let pile = string_to_pile("1A 28C 29C 30A");
            let new_states = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Hero,
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            // Furthest back werewolf should block
            assert_actual_vs_expected_piles(&new_states, vec!["1A 28C 29D 30A"])
        }
    }

    #[test]
    fn test_werewolf_spider() {
        let starting_pile = string_to_pile("28C 26A");
        let new_states = resolve_enemy_turn(&T::new(starting_pile), Allegiance::Werewolf, 0);
        // Werewolf can't perform the attack
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([string_to_pile("28C 26A")]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_swarm_inspire() {
        let starting_pile = string_to_pile("30A 24A 27B 29A");
        let new_states = resolve_player_action(
            &GameStateWithEventLog::new(starting_pile),
            &WrappedAction {
                action: Action::Inspire,
                target: Target::Enemy,
            },
            0,
            &NO_ENERGY_USED,
        );

        // When player inspires 24, 27 should swarm
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("30A 24A 27B 29A"), // Skip, or inspire 27B
            string_to_pile("30A 24A 27B 29B"), // Werewolf doens't block so 24A doesn't
            // activate
            string_to_pile("30A 24B 27B 29B"), // Werewolf blocks so 24 activates and
                                               // rotates
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_attacks() {
        {
            // Basic healthy -> hurt
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("1A")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["1D"]);
        }

        {
            // hurt -> exhausted
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("1D")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["1C"]);
        }

        {
            // exhausted -> no options
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("1C")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec![]);
        }

        {
            // can shield or not
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("5A")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["5B", "5C"]);
        }

        {
            // If reaction is forced, we can only block
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("7B")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["7A"]);
        }

        {
            // Can be hurt 2 ways
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("33B")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["33C", "33D"]);
        }

        {
            // Can be hurt 2 ways. Even with forced reactions
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("45A")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["45C", "45D"]);
        }

        {
            // The exhausted card has a reaction trigger, apply it.
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("33C")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(&outcomes, vec!["33D"]);
        }
    }

    #[test]
    fn test_beastmaster_gets_attacked() {
        {
            // beastmaster can pull in an assist to block
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("37A 40A")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(
                &outcomes,
                vec![
                    "37C 40A", // Get Hit
                    "37B 40B", // React assist to block
                ],
            );
        }

        {
            // pull in assist dodge too
            let outcomes =
                attack_card_get_all_outcomes(&T::new(string_to_pile("37A 41A")), 0, HitType::Hit);
            assert_actual_vs_expected_piles(
                &outcomes,
                vec![
                    "37C 41A", // Get Hit
                    "37B 41B", // React assist to dodge
                ],
            );
        }
    }

    #[test]
    fn test_beastmaster_block_for_ally() {
        {
            let pile = string_to_pile("6A 38B 37A 40A");
            // Can assist block on behalf of ally
            let outcomes = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );
            assert_actual_vs_expected_piles(
                &outcomes,
                vec![
                    "6A 38C 37A 40A", // Get Hit
                    "6A 38B 37B 40B", // blocked by 37, assisted by 41
                ],
            );
        }

        {
            // Dodge doesn't work
            let outcomes = resolve_enemy_action(
                &T::new(string_to_pile("6A 38B 37A 41A")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Hit(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );
            assert_actual_vs_expected_piles(
                &outcomes,
                vec![
                    "6A 38C 37A 41A", // Get Hit
                ],
            );
        }
    }

    #[test]
    fn test_enemy_push() {
        let tester = |start: &str, range: Range, target: Target, expected: &str| {
            let pile = string_to_pile(start);
            let expected_pile = string_to_pile(expected);

            let result_states = resolve_enemy_action(
                &T::new(pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Push(range),
                    target,
                },
                0,
            );

            if start == expected {
                // Expect no change
                assert_eq!(result_states.len(), 0);
            } else {
                assert_eq!(result_states.len(), 1);
                let end_pile = &result_states[0].pile;
                assert_eq!(end_pile, &expected_pile);
            }
        };

        tester("7 8 9 1 2 3", Range::Inf, Target::Enemy, "7 8 9 2 3 1");
        tester("7 8 9 1 2 3", Range::Int(10), Target::Enemy, "7 8 9 2 3 1");
        tester("7 8 9 1 2 3", Range::Int(3), Target::Enemy, "7 8 9 2 3 1");
        tester("7 8 9 1 2 3", Range::Int(2), Target::Enemy, "7 8 9 1 2 3");

        tester("7 8 9 1 2 3", Range::Inf, Target::Ally, "7 9 1 2 3 8");
        tester("7 8 9 1 2 3", Range::Inf, Target::Any, "7 9 1 2 3 8");
    }

    #[test]
    fn test_enemy_heal() {
        let pile = string_to_pile("9D 6D 3D 7A 8D");

        let new_states_1 = resolve_enemy_action(
            &T::new(pile.clone()),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Heal,
                target: Target::Ally,
            },
            0,
        );

        {
            let futures = states_to_pile_set(&new_states_1);
            let expected_futures = HashSet::from([string_to_pile("9D 6A 3D 7A 8D")]);

            assert_eq!(futures, expected_futures);
        }

        let new_states_2 = resolve_enemy_action(
            &T::new(new_states_1[0].pile.clone()),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Heal,
                target: Target::Ally,
            },
            0,
        );

        {
            let futures = states_to_pile_set(&new_states_2);
            let expected_futures = HashSet::from([string_to_pile("9D 6A 3D 7A 8A")]);

            assert_eq!(futures, expected_futures);
        }
    }

    #[test]
    fn test_player_heal() {
        {
            let new_states = resolve_player_action(
                &T::new(string_to_pile("8D 6D 3D 1D 2C")),
                &WrappedAction {
                    action: Action::Heal,
                    target: Target::Ally,
                },
                0,
                &NO_ENERGY_USED,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec!["8D 6D 3D 1D 2C", "8D 6D 3A 1D 2C", "8D 6D 3D 1A 2C"],
            );
        }

        {
            let new_states = resolve_player_action(
                &T::new(string_to_pile("8D 6D 3D 1D 2C")),
                &WrappedAction {
                    action: Action::Heal,
                    target: Target::Any,
                },
                0,
                &NO_ENERGY_USED,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("8D 6D 3D 1D 2C"), // Skip
                string_to_pile("8D 6D 3A 1D 2C"), // Heal 3
                string_to_pile("8D 6D 3D 1A 2C"), // Heal 1
                string_to_pile("8D 6A 3D 1D 2C"), // Heal 6
                                                  // 2 is dead, can't be healed
                                                  // 8 is active, can't be healed
            ]);
            assert_eq!(futures, expected_futures);
        }
    }

    #[test]
    fn test_move_card_by_up_to_amount() {
        let starting_pile = string_to_pile("1 7 2 6 5 4 8 9 3");
        {
            let new_states =
                move_card_by_up_to_amount(&T::new(starting_pile), 3, 2, MoveType::Quicken);
            assert_actual_vs_expected_piles(
                &new_states,
                vec!["1 7 6 2 5 4 8 9 3", "1 6 7 2 5 4 8 9 3"],
            );
        }
    }

    #[test]
    fn test_quicken_delay() {
        let starting_pile = string_to_pile("1 7 2 6 5 4 8 9 3");

        {
            let new_states = resolve_player_action(
                &T::new(starting_pile.clone()),
                &WrappedAction {
                    action: Action::Quicken(2),
                    target: Target::Any,
                },
                2,
                &NO_ENERGY_USED,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("1 7 2 6 8 5 4 9 3"),
                string_to_pile("1 7 2 6 4 5 8 9 3"),
                string_to_pile("1 7 2 4 6 5 8 9 3"),
                string_to_pile("1 7 2 6 5 9 4 8 3"),
                string_to_pile("1 7 2 6 5 4 3 8 9"),
                string_to_pile("1 7 2 6 5 8 4 9 3"),
                string_to_pile("1 7 2 5 6 4 8 9 3"),
                string_to_pile("1 7 2 6 5 4 9 8 3"),
                string_to_pile("1 7 2 6 5 4 8 9 3"),
                string_to_pile("1 7 2 6 5 4 8 3 9"),
            ]);
            assert_eq!(futures, expected_futures);
        }

        {
            let new_states = resolve_player_action(
                &T::new(starting_pile.clone()),
                &WrappedAction {
                    action: Action::Quicken(2),
                    target: Target::Ally,
                },
                2,
                &NO_ENERGY_USED,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("1 7 2 6 4 5 8 9 3"),
                string_to_pile("1 7 2 4 6 5 8 9 3"),
                string_to_pile("1 7 2 6 5 4 3 8 9"),
                string_to_pile("1 7 2 5 6 4 8 9 3"),
                string_to_pile("1 7 2 6 5 4 8 9 3"),
                string_to_pile("1 7 2 6 5 4 8 3 9"),
            ]);

            assert_eq!(futures, expected_futures);
        }

        {
            let new_states = resolve_player_action(
                &T::new(starting_pile.clone()),
                &WrappedAction {
                    action: Action::Quicken(2),
                    target: Target::Enemy,
                },
                2,
                &NO_ENERGY_USED,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("1 7 2 6 8 5 4 9 3"),
                string_to_pile("1 7 2 6 5 9 4 8 3"),
                string_to_pile("1 7 2 6 5 8 4 9 3"),
                string_to_pile("1 7 2 6 5 4 9 8 3"),
                string_to_pile("1 7 2 6 5 4 8 9 3"),
            ]);

            assert_eq!(futures, expected_futures);
        }

        {
            let new_states = resolve_player_action(
                &T::new(starting_pile.clone()),
                &WrappedAction {
                    action: Action::Quicken(2),
                    target: Target::Any,
                },
                2,
                &NO_ENERGY_USED,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("1 7 2 6 5 8 4 9 3"),
                string_to_pile("1 7 2 6 5 4 9 8 3"),
                string_to_pile("1 7 2 6 5 9 4 8 3"),
                string_to_pile("1 7 2 6 5 4 3 8 9"),
                string_to_pile("1 7 2 6 8 5 4 9 3"),
                string_to_pile("1 7 2 5 6 4 8 9 3"),
                string_to_pile("1 7 2 6 5 4 8 3 9"),
                string_to_pile("1 7 2 4 6 5 8 9 3"),
                string_to_pile("1 7 2 6 4 5 8 9 3"),
                string_to_pile("1 7 2 6 5 4 8 9 3"),
            ]);
            assert_eq!(futures, expected_futures);
        }
    }

    #[test]
    fn test_quicken_trap() {
        // 10C and 14C have traps. Only enemy is at the end
        let starting_pile = string_to_pile("11A 10C 14C 6B");

        let new_states = resolve_player_action(
            &T::new(starting_pile),
            &WrappedAction {
                action: Action::Quicken(3),
                target: Target::Enemy,
            },
            0,
            &NO_ENERGY_USED,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("11A 10C 14C 6B"), // Skip
            string_to_pile("11A 10C 6A 14C"), // Move over first trap
            string_to_pile("11A 6D 10C 14C"), // Move over second trap
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_quicken_trap_with_allies() {
        // No friendly fire
        let starting_pile = string_to_pile("11A 10C 14C");

        let new_states = resolve_player_action(
            &T::new(starting_pile),
            &WrappedAction {
                action: Action::Quicken(3),
                target: Target::Ally,
            },
            0,
            &NO_ENERGY_USED,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("11A 10C 14C"), // Skip
            string_to_pile("11A 14C 10C"), // Move
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_delay_trap() {
        let starting_pile = string_to_pile("11A 6B 10C 14C");

        let new_states = resolve_player_action(
            &T::new(starting_pile),
            &WrappedAction {
                action: Action::Delay(3),
                target: Target::Enemy,
            },
            0,
            &NO_ENERGY_USED,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("11A 6B 10C 14C "), // Skip
            string_to_pile("11A 10C 6A 14C"),  // Move over first trap
            string_to_pile("11A 10C 14C 6D"),  // Move over second trap
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_quicken_trap_into_weight() {
        // 6D is heavy, so we won't move it over the second trap once it takes damage
        let starting_pile = string_to_pile("11A 10C 14C 6A");

        let new_states = resolve_player_action(
            &T::new(starting_pile),
            &WrappedAction {
                action: Action::Quicken(2),
                target: Target::Enemy,
            },
            0,
            &NO_ENERGY_USED,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("11A 10C 14C 6A"), // Skip
            string_to_pile("11A 10C 6D 14C"), // Move over first trap
                                              // We won't move over the second trap,
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_standard_ally_pull() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("26A 12A 13B 27B")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "26A 27B 12A 13B", // 27 pulled over no traps
                ],
            );
        }
    }

    #[test]
    fn test_ally_pull_over_trap() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("26A 12A 13A 27B")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "26A 27A 12A 13A", // 27 pulled and takes damage over 13
                ],
            );
        }
    }

    #[test]
    fn test_ally_pull_over_multi_traps() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("26A 10A 12B 13A 27B")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "26A 27C 10A 12B 13A", // 27 pulled and takes 3 damage
                ],
            );
        }
    }

    #[test]
    fn test_ally_pull_exhausted_over_trap() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("26A 12A 13A 27C")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "26A 27C 12A 13A", // 27 pulled but cant take damage
                ],
            );
        }
    }

    #[test]
    fn test_pull_with_options_over_trap() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("36C 10A 35B")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "36C 35C 10A", // Hurt 35 to C
                    "36C 35D 10A", // Hurt 35 to D
                ],
            );
        }
    }

    #[test]
    fn test_pull_hero_over_trap() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("8D 10A 12B")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            assert_actual_vs_expected_piles(&new_states, vec!["8D 12B 10A"]);
        }
    }

    #[test]
    fn test_pull_dodge() {
        let starting_pile = string_to_pile("6 8 12");

        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Pull(Range::Inf),
                target: Target::Enemy,
            },
            0,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("6 12A 8"), // Skip
            string_to_pile("6 8 12B"), // Dodge
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_pull_exhausted_hero() {
        {
            // Regular case: hero is pulled when healthy
            let starting_pile = string_to_pile("6 7 1 2");
            let new_states = resolve_enemy_action(
                &T::new(starting_pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("6 2 7 1"), // Dodge
            ]);
            assert_eq!(futures, expected_futures);
        }

        {
            // Shouldn't target 2 when it's exhausted
            let starting_pile = string_to_pile("6 7 1 2C");
            let new_states = resolve_enemy_action(
                &T::new(starting_pile),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            let futures = states_to_pile_set(&new_states);
            let expected_futures = HashSet::from([
                string_to_pile("6 1 7 2C"), // Dodge
            ]);
            assert_eq!(futures, expected_futures);
        }
    }

    #[test]
    fn test_pull_beastmaster() {
        {
            // Regular case: hero is pulled when healthy
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("6A 41A 37A")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Enemy,
                },
                0,
            );

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "6A 37A 41A", // Get pulled
                    "6A 41B 37B", // Dodge Assist
                ],
            );
        }
    }

    #[test]
    fn test_inspired_pull() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("1A 26A 12A 13B 27B")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Pull(Range::Inf),
                    target: Target::Ally,
                },
                1,
            );

            assert_actual_vs_expected_piles(&new_states, vec!["1A 26A 27B 12A 13B"]);
        }
    }

    #[test]
    fn test_standard_ally_push() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("26B 27B 11A 14A")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Push(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(&new_states, vec!["26B 11A 14A 27B"]);
        }
    }

    #[test]
    fn test_ally_push_over_trap() {
        {
            let new_states = resolve_enemy_action(
                &T::new(string_to_pile("26B 27B 10A 13A")),
                Allegiance::Baddie,
                &WrappedAction {
                    action: Action::Push(Range::Inf),
                    target: Target::Ally,
                },
                0,
            );

            assert_actual_vs_expected_piles(&new_states, vec!["26B 10A 13A 27D"]);
        }
    }

    #[test]
    fn test_push_dodge() {
        let starting_pile = string_to_pile("6 12 8");

        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Push(Range::Inf),
                target: Target::Enemy,
            },
            0,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("6 12B 8"), // Skip
            string_to_pile("6 8 12A"), // Dodge
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_delay_trap_into_weight() {
        let starting_pile = string_to_pile("11A 6A 10C 14C");

        let new_states = resolve_player_action(
            &T::new(starting_pile),
            &WrappedAction {
                action: Action::Delay(2),
                target: Target::Enemy,
            },
            0,
            &NO_ENERGY_USED,
        );

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("11A 6A 10C 14C "), // Skip
            string_to_pile("11A 10C 6D 14C"),  // Move over first trap
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_swarm() {
        let starting_pile = string_to_pile("27B 26B 25A 1A");

        let new_states = swarm_me_recursive(&T::new(starting_pile), Allegiance::Baddie, 1);

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("27B 26B 25B 1D "), // 25B rotates, 26B hits 1A
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_swarm_no_impact() {
        // 24C swarm is to heal, but it has no targets
        // 27A should attack 1A anyway
        let starting_pile = string_to_pile("26A 24C 27A 1A");
        let new_states = swarm_me_recursive(&T::new(starting_pile), Allegiance::Baddie, 1);

        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([string_to_pile("26A 24C 27A 1D")]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_death() {
        let starting_pile = string_to_pile("1 7 2 6 4 5");
        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Death,
                target: Target::Enemy,
            },
            0,
        );
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([string_to_pile("1C 7 2C 6 4D 5D")]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_void_1() {
        let starting_pile = string_to_pile("36 1 2 3");
        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Void,
                target: Target::Enemy,
            },
            0,
        );
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([string_to_pile("36 1C 2 3")]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_void_2() {
        // We should skip over 1C because it's already dead
        let starting_pile = string_to_pile("36 1C 2 3");
        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Void,
                target: Target::Enemy,
            },
            0,
        );
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([string_to_pile("36 1C 2C 3")]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_claws_inf() {
        let starting_pile = string_to_pile("33D 35A 1B 3A 2A");
        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Claws(Range::Inf),
                target: Target::Enemy,
            },
            0,
        );
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([
            string_to_pile("33D 35A 1A 3D 2D"), // Block
            string_to_pile("33D 35A 1D 3D 2D"), // Hit
        ]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_claws_range() {
        let starting_pile = string_to_pile("33D 3A 2A 1A");
        let new_states = resolve_enemy_action(
            &T::new(starting_pile),
            Allegiance::Baddie,
            &WrappedAction {
                action: Action::Claws(Range::Int(2)),
                target: Target::Enemy,
            },
            0,
        );
        let futures = states_to_pile_set(&new_states);
        let expected_futures = HashSet::from([string_to_pile("33D 3D 2D 1A")]);
        assert_eq!(futures, expected_futures);
    }

    #[test]
    fn test_attack_all_in_range() {
        {
            // Basic ex:
            let pile = string_to_pile("4 1 2 3 5");
            let new_states = attack_all_in_range(
                &T::new(pile),
                Allegiance::Baddie,
                1,
                4,
                Target::Any,
                HitType::Hit,
            );
            assert_actual_vs_expected_piles(&new_states, vec!["4A 1D 2D 3D 5A"]);
        }

        {
            // Even when someone doesn't get hit in middle
            let pile = string_to_pile("4 1 2C 3 5");
            let new_states = attack_all_in_range(
                &T::new(pile),
                Allegiance::Baddie,
                1,
                4,
                Target::Any,
                HitType::Hit,
            );
            assert_actual_vs_expected_piles(&new_states, vec!["4A 1D 2C 3D 5A"]);
        }

        {
            // When no one gets hit, there should be no results
            let pile = string_to_pile("4 1C 2C");
            let new_states = attack_all_in_range(
                &T::new(pile),
                Allegiance::Baddie,
                1,
                3,
                Target::Any,
                HitType::Hit,
            );
            assert_actual_vs_expected_piles(&new_states, vec![]);
        }
    }

    #[test]
    fn test_ablaze() {
        let pile = string_to_pile("20A 23A 6A 7A 19A 22A");
        let new_states =
            resolve_player_row(&T::new(pile.clone()), &pile[0].get_active_face().rows[0], 0);

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "20A 23B 6D 7D 19B 22A", // ablaze 23, 19 and hit
                "20A 23B 6A 7A 19B 22A", // ablaze 23, 19 and skip
                //
                "20A 23B 6D 7D 19B 22B", // ablaze 23, 22 and hit
                "20A 23B 6A 7A 19A 22B", // ablaze 23, 22 and skip
                //
                "20A 23A 6A 7A 19B 22B", // ablaze 19, 22
            ],
        );
    }

    #[test]
    fn test_fireball() {
        {
            let pile = string_to_pile("21A 19D 6A 23A 9A");
            let new_states =
                resolve_player_row(&T::new(pile.clone()), &pile[0].get_active_face().rows[0], 0);

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "21B 19C 6A 23B 9A", // Pay but don't attack
                    "21B 19C 6D 23B 9A", // Attack with 19
                    "21B 19C 6D 23B 9D", // Attack with 23
                ],
            );
        }

        {
            // Check bounds
            let pile = string_to_pile("21A 19D 6A 9A 23A");
            let new_states =
                resolve_player_row(&T::new(pile.clone()), &pile[0].get_active_face().rows[0], 0);

            assert_actual_vs_expected_piles(
                &new_states,
                vec![
                    "21B 19C 6A 9A 23B ", // Pay but don't attack
                    "21B 19C 6D 9A 23B ", // Attack with 19
                    "21B 19C 6A 9D 23B ", // Attack with 23
                ],
            );
        }
    }

    #[test]
    fn test_teleport_ally() {
        let pile = string_to_pile("21 23 6 19 20 9");

        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Teleport,
                target: Target::Ally,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "21 23 6 20 19 9",
                "21 23 6 19 20 9",
                "21 19 6 23 20 9",
                "21 20 6 19 23 9",
            ],
        );
    }

    #[test]
    fn test_teleport_enemy() {
        let pile = string_to_pile("21 23 6 19 20 9");

        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Teleport,
                target: Target::Enemy,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(&new_states, vec!["21 23 6 19 20 9", "21 23 9 19 20 6"]);
    }

    #[test]
    fn test_teleport_any() {
        let pile = string_to_pile("21 23 6 19 20 9");

        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Teleport,
                target: Target::Any,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "21 23 6 19 20 9",
                "21 6 23 19 20 9",
                "21 9 6 19 20 23",
                "21 23 19 6 20 9",
                "21 23 20 19 6 9",
                "21 23 6 9 20 19",
                "21 23 6 19 9 20",
            ],
        );
    }

    #[test]
    fn test_verdant_hit() {
        let new_states =
            attack_card_get_all_outcomes(&T::new(string_to_pile("43D 1 4 2 5 3")), 0, HitType::Hit);

        assert_actual_vs_expected_piles(&new_states, vec!["43B 1D 4 2D 5 3D"]);
    }

    #[test]
    fn test_manouver() {
        let pile = string_to_pile("11B 13A 10D 12D");

        let new_states = resolve_player_action(
            &T::new(pile),
            &WrappedAction {
                action: Action::Manouver,
                target: Target::Ally,
            },
            0,
            &NO_ENERGY_USED,
        );

        assert_actual_vs_expected_piles(
            &new_states,
            vec![
                "11B 13A 10D 12D", // Skip
                "11B 13B 10D 12D", // Rotate 13
                "11B 13A 10C 12D", // Rotate 10, which damages it
                                   // 12 cannot rotate, which would result in a heal
            ],
        );
    }
}
