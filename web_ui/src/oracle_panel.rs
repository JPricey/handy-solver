use crate::components::*;
use crate::solver::*;
use crate::types::*;
use closure::closure;
use futures::SinkExt;
use futures::StreamExt;
use gloo::worker::Spawnable;
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
    return result;
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
) -> impl IntoView {
    let (is_enabled, set_enabled) = create_signal(cx, false);

    let worker_path = get_full_path("worker.js");
    let (bridge_sink, mut bridge_stream) = SolverWorker::spawner().spawn(&worker_path).split();
    let bridge_sink = Rc::new(RefCell::new(bridge_sink));

    let (raw_ai_path, set_raw_ai_path) = create_signal::<Vec<Pile>>(cx, vec![]);
    let (is_worker_started, set_worker_started) = create_signal(cx, false);
    let (worker_state, set_worker_state) = create_signal::<SolverState>(cx, SolverState::Init);

    // Clear piles when disabled
    create_effect(
        cx,
        closure!(clone bridge_sink, |_| {
                if is_enabled.get() {
                    return;
                }

                let bridge_sink = bridge_sink.clone();
                spawn_local(async move {
                    if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                        bridge_sink
                            .send(ControlSignal::ClearRootPiles)
                            .await
                            .unwrap();
                    }
                });
            }
        ),
    );

    // Set piles on start, or when frame changes
    create_effect(
        cx,
        closure!(clone bridge_sink, |_| {
            if !is_worker_started.get() || !is_enabled.get() {
                return;
            }
            let current_frame = current_frame.get();
            let next_root_states = find_final_piles_matching_prefix(
                &current_frame.root_pile,
                &current_frame.event_history,
            );
            let strings: Vec<_> = next_root_states
                .iter()
                .map(|pile| format!("{pile:?}"))
                .collect();
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
                        .send(ControlSignal::SetModel(final_model))
                        .await
                        .unwrap();
                }
            });
        }),
    );

    spawn_local(async move {
        while let Some(output_signal) = bridge_stream.next().await {
            // log!("msg: {:?}", output_signal);
            match output_signal {
                OutputSignal::Start => {
                    set_worker_started.set(true);
                }
                OutputSignal::SolutionCrumb(pile_strings) => {
                    let piles: Vec<_> = pile_strings.iter().map(|s| string_to_pile(s)).collect();
                    set_raw_ai_path.set(piles);
                    set_worker_state.set(SolverState::Working);
                }
                OutputSignal::Working => {
                    set_worker_state.set(SolverState::Working);
                }
                OutputSignal::Sleeping | OutputSignal::Done => match worker_state.get() {
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

    let best_path = create_memo::<Option<Vec<Pile>>>(cx, move |last| {
        let current_frame = current_frame.get();
        let next_piles: HashSet<Pile> = find_final_piles_matching_prefix(
            &current_frame.root_pile,
            &current_frame.event_history,
        )
        .into_iter()
        .collect();
        let last: Option<Vec<Pile>> = last.map_or(None, |l| l.clone());

        let mut last_best_path: Option<Vec<Pile>> = None;
        if let Some(last_piles) = last {
            for (i, pile) in last_piles.iter().enumerate().rev() {
                if next_piles.contains(pile) {
                    let smaller_crumb: Vec<Pile> =
                        last_piles[i..last_piles.len()].iter().cloned().collect();
                    last_best_path = Some(smaller_crumb);
                    break;
                }
            }
        }

        let mut raw_best_path: Option<Vec<Pile>> = None;
        let raw_ai_path = raw_ai_path.get();
        if raw_ai_path.len() > 0 {
            for (i, pile) in raw_ai_path.iter().enumerate().rev() {
                if next_piles.contains(pile) {
                    let smaller_crumb: Vec<Pile> =
                        raw_ai_path[i..raw_ai_path.len()].iter().cloned().collect();
                    raw_best_path = Some(smaller_crumb);
                    break;
                }
            }
        }

        match (last_best_path, raw_best_path) {
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
        }
    });

    let next_event = create_memo::<Option<Event>>(cx, move |_| {
        let best_path = best_path.get();
        let Some(best_path) = best_path else {
            return None;
        };

        let current_frame = current_frame.get();
        let states = resolve_top_card(&GameStateWithEventLog::new(current_frame.root_pile));

        find_next_event_matching_prefix_and_with_final_state(
            &states,
            &current_frame.event_history,
            &best_path[0],
        )
    });

    let path_text = move || match worker_state.get() {
        SolverState::Init => {
            format!("Waking up...")
        }
        SolverState::Working | SolverState::Pending => {
            if let Some(path) = best_path.get() {
                format!("Win in {}. Looking for better...", path.len())
            } else {
                format!("Searching...")
            }
        }
        SolverState::Idle => {
            if let Some(path) = best_path.get() {
                format!("Win in {}.", path.len())
            } else {
                format!("No win found")
            }
        }
    };

    on_cleanup(
        cx,
        closure!(clone bridge_sink, || {
                    let bridge_sink = bridge_sink.clone();
                    spawn_local(async move {
                        if let Ok(mut bridge_sink) = bridge_sink.try_borrow_mut() {
                            bridge_sink
                                .send(ControlSignal::End)
                                .await
                                .unwrap();
                        }
                    });
                }
        ),
    );

    view! { cx,
        <Button
            width=width
            height=height
            background=Signal::derive(cx, || "d7d7a2".to_owned())
            on:click= move |_| set_enabled.set(!is_enabled.get())
            border="solid".to_owned()
            border_colour="#816b5b".to_owned()
        >
            <Show
                when=move || is_enabled.get()
                fallback=move |_| "Show Engine"
            >
                <div>
                    {path_text}
                </div>
                <div>
                    <Show
                        when=move || next_event.get().is_some()
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
