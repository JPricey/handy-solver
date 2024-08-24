use crate::components::*;
use crate::solver::*;
use crate::types::*;
use crate::versioning::add_version_to_path;
use closure::closure;
use futures::SinkExt;
use futures::StreamExt;
use gloo::worker::Spawnable;
use handy_core::game::end_game::{is_game_winner, GameEndCheckType};
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use leptos::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use url::Url;

fn get_full_path(suffix: &str) -> String {
    let base_uri = document().document_uri().unwrap();
    let base_url = Url::parse(&base_uri).unwrap().join(suffix);
    let result = base_url.unwrap().to_string();
    return add_version_to_path(&result);
}

fn model_url(matchup: Matchup) -> String {
    let model_file = format!("static/models/{:?}.{:?}.yaml", matchup.0, matchup.1);

    return get_full_path(&model_file);
}

async fn fetch_model_from_full_url(url: &str) -> Result<Model, ()> {
    let model_string = reqwasm::http::Request::get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let model: Model = serde_yaml::from_str(&model_string).unwrap();
    Ok(model)
}

#[component]
pub fn OraclePanel(
    cx: Scope,
    width: WindowUnit,
    height: WindowUnit,
    current_frame: Signal<GameFrame>,
    game_end_type: Memo<GameEndCheckType>,
    is_enabled: RwSignal<bool>,
) -> impl IntoView {
    let worker_path = get_full_path("worker.js");
    let (bridge_sink, mut bridge_stream) = SolverWorker::spawner().spawn(&worker_path).split();
    let bridge_sink = Rc::new(RefCell::new(bridge_sink));

    let (raw_ai_path, set_raw_ai_path) =
        create_signal::<(GameEndCheckType, Vec<Pile>)>(cx, (game_end_type.get_untracked(), vec![]));
    let (is_worker_started, set_worker_started) = create_signal(cx, false);
    let (worker_state, set_worker_state) = create_signal::<SolverState>(cx, SolverState::Init);

    let game_winner = create_memo(cx, move |_| {
        let current_frame = current_frame.get();
        if current_frame.event_history.len() == 0 {
            is_game_winner(&current_frame.current_pile, game_end_type.get())
        } else {
            WinType::Unresolved
        }
    });

    let trigger_clear_root_piles = closure!(clone bridge_sink, || {
        let bridge_sink = bridge_sink.clone();
        spawn_local(async move {
            if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                bridge_sink
                    .send(ControlSignal::ClearRootPiles)
                    .await
                    .unwrap();
            }
        });
    });

    // Clear piles when disabled
    create_effect(
        cx,
        closure!(clone trigger_clear_root_piles, |_| {
            if is_enabled.get() {
                return;
            }

            trigger_clear_root_piles();
        }),
    );

    // Set piles on start, or when frame changes
    create_effect(
        cx,
        closure!(clone bridge_sink, clone trigger_clear_root_piles, |_| {
            if !is_worker_started.get() || !is_enabled.get() {
                return;
            }
            let current_frame = current_frame.get();
            let game_end_check_type = game_end_type.get();

            // If the game is already over, don't compute anything
            if current_frame.event_history.len() == 0 && is_game_winner(&current_frame.root_pile, game_end_check_type).is_over() {
                trigger_clear_root_piles();
                return;
            }

            let next_root_piles = find_final_piles_matching_prefix(
                &current_frame.root_pile,
                &current_frame.event_history,
            );

            // If a next pile is winning, don't bother the solver
            for candidate_root_pile in &next_root_piles {
                if is_game_winner(candidate_root_pile, game_end_check_type) == WinType::Win {
                    set_raw_ai_path.set((game_end_type.get(), vec![candidate_root_pile.clone()]));
                    trigger_clear_root_piles();
                    return;
                }
            }

            let strings: Vec<_> = next_root_piles
                .iter()
                .filter_map(|pile| {
                    if is_game_winner(pile, game_end_check_type) == WinType::Lose {
                        None
                    } else {
                        Some(format!("{pile:?}"))
                    }
                })
                .collect();

            log!("sending states: {:?}", &strings);

            set_worker_state.set(SolverState::Working);

            let bridge_sink = bridge_sink.clone();
            spawn_local(async move {
                if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                    bridge_sink
                        .send(ControlSignal::SetRootPiles(strings))
                        .await
                        .unwrap();
                }
            });
        }),
    );

    // Update game end type
    create_effect(
        cx,
        closure!(clone bridge_sink, |_| {
            if !is_worker_started.get() {
                return;
            }

            let bridge_sink = bridge_sink.clone();
            let game_end_type_mode = game_end_type.get();

            spawn_local(async move {
                if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                    bridge_sink
                        .send(ControlSignal::SetGameEndMode(game_end_type_mode))
                        .await
                        .unwrap();
                }
            });
        }),
    );

    // Set model on start
    create_effect(
        cx,
        closure!(clone bridge_sink, |_| {
            if !is_worker_started.get() {
                return;
            }

            let bridge_sink = bridge_sink.clone();
            spawn_local(async move {
                let pile = current_frame.get_untracked().root_pile;
                let matchups = get_all_matchups_from_pile(&pile);

                let mut models: Vec<Model> = Vec::new();
                for matchup in matchups {
                    let model_url = model_url(matchup);
                    let model = fetch_model_from_full_url(&model_url).await.unwrap();
                    models.push(model);
                }

                let final_model = if models.len() == 1 {
                    models[0].clone()
                } else {
                    merge_models_for_pile(&pile, &models)
                };

                if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                    bridge_sink
                        .send(ControlSignal::SetGameEndMode(game_end_type.get_untracked()))
                        .await
                        .unwrap();

                    bridge_sink
                        .send(ControlSignal::SetModel(final_model))
                        .await
                        .unwrap();
                }
            });
        }),
    );

    spawn_local(async move {
        while let Some(output_signal) = bridge_stream.next().await {
            match output_signal {
                OutputSignal::Start => {
                    set_worker_started.set(true);
                }
                OutputSignal::SolutionCrumb(game_end_check_type, pile_strings) => {
                    log!(
                        "got new output: {:?}, {:?}",
                        game_end_check_type,
                        pile_strings
                    );
                    let piles: Vec<_> = pile_strings.iter().map(|s| string_to_pile(s)).collect();
                    set_raw_ai_path.set((game_end_check_type, piles));
                    set_worker_state.set(SolverState::Working);
                }
                OutputSignal::Working => {
                    set_worker_state.set(SolverState::Working);
                }
                OutputSignal::Sleeping | OutputSignal::Done => match worker_state.get_untracked() {
                    SolverState::Pending => set_worker_state.set(SolverState::Idle),
                    SolverState::Idle => (),
                    _ => set_worker_state.set(SolverState::Pending),
                },
                OutputSignal::Init => {
                    set_worker_state.set(SolverState::Init);
                }
            };
        }
    });

    let best_path = create_memo::<Option<(GameEndCheckType, Vec<Pile>)>>(cx, move |last| {
        let current_frame = current_frame.get();
        let next_piles: HashSet<Pile> = find_final_piles_matching_prefix(
            &current_frame.root_pile,
            &current_frame.event_history,
        )
        .into_iter()
        .collect();
        let last: Option<(GameEndCheckType, Vec<Pile>)> = last.map_or(None, |l| l.clone());

        let mut last_best_path: Option<Vec<Pile>> = None;
        if let Some((last_type, last_piles)) = last {
            if last_type == game_end_type.get() {
                for (i, pile) in last_piles.iter().enumerate().rev() {
                    if next_piles.contains(pile) {
                        let smaller_crumb: Vec<Pile> =
                            last_piles[i..last_piles.len()].iter().cloned().collect();
                        last_best_path = Some(smaller_crumb);
                        break;
                    }
                }
            }
        }

        let mut raw_best_path: Option<Vec<Pile>> = None;
        let (raw_ai_type, raw_ai_path) = raw_ai_path.get();
        if raw_ai_path.len() > 0 && raw_ai_type == game_end_type.get() {
            for (i, pile) in raw_ai_path.iter().enumerate().rev() {
                if next_piles.contains(pile) {
                    let smaller_crumb: Vec<Pile> =
                        raw_ai_path[i..raw_ai_path.len()].iter().cloned().collect();
                    raw_best_path = Some(smaller_crumb);
                    break;
                }
            }
        }

        let new_best_path = match (last_best_path, raw_best_path) {
            (None, None) => None,
            (Some(last), None) => Some(last),
            (None, Some(raw)) => Some(raw),
            (Some(last), Some(raw)) => {
                if last.len() <= raw.len() {
                    Some(last)
                } else {
                    Some(raw)
                }
            }
        };

        if let Some(new_best_path) = new_best_path {
            Some((game_end_type.get(), new_best_path))
        } else {
            None
        }
    });

    let next_event = create_memo::<Option<Event>>(cx, move |_| {
        let best_path = best_path.get();
        let Some(best_path) = best_path else {
            return None;
        };

        let current_frame = current_frame.get();
        let states = resolve_top_card_starting_with_prefix_dedupe_excess(
            &GameStateWithPileTrackedEventLog::new(current_frame.root_pile),
            &current_frame.event_history,
        );

        find_next_event_matching_prefix_and_with_final_state(
            &states,
            &current_frame.event_history,
            &best_path.1[0],
        )
    });

    let path_text = move || {
        let resolution = game_winner.get();
        if resolution == WinType::Win {
            return ":)".to_owned();
        } else if resolution == WinType::Lose {
            return ":(".to_owned();
        }

        if let Some(path) = best_path.get() {
            let win_in_text = format!("Win in {}.", path.1.len());

            match worker_state.get() {
                SolverState::Working | SolverState::Pending => {
                    format!("{win_in_text} Looking for better...")
                }
                _ => win_in_text,
            }
        } else {
            match worker_state.get() {
                SolverState::Init => {
                    format!("Waking up...")
                }
                SolverState::Working | SolverState::Pending => {
                    format!("Searching...")
                }
                SolverState::Idle => {
                    format!("No win found")
                }
            }
        }
    };

    on_cleanup(
        cx,
        closure!(clone bridge_sink, || {
                    let bridge_sink = bridge_sink.clone();
                    spawn_local(async move {
                        // log!("Sending End Local");
                        if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                            bridge_sink
                                .send(ControlSignal::End)
                                .await
                                .unwrap();
                            // log!("Sent End Local");
                        }
                    });
                }
        ),
    );

    view! { cx,
        <Button
            width=width
            height=height
            background=Signal::derive(cx, || MENU_BUTTON_COLOUR.to_owned())
            on:click= move |_| is_enabled.set(!is_enabled.get())
        >
            <Show
                when=move || is_enabled.get()
                fallback=move |_| "Show Engine (E)"
            >
                <div>
                    {path_text}
                </div>
                <div>
                    <Show
                        when=move || next_event.get().is_some() && !game_winner.get().is_over()
                        fallback=|_| ()
                    >
                        {move || view!{cx,
                            <EventSpan event=next_event.get().unwrap() />
                        }}
                    </Show>
                </div>
            </Show>
        </Button>
    }
}
