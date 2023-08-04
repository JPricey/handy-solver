use crate::components::*;
use crate::contexts::*;
use crate::game_player::*;
use crate::types::*;
use handy_core::game::primitives::*;
use handy_core::game::Class;
use handy_core::solver::{BADDIES, HEROS};
use handy_core::utils::pile_utils::*;
use leptos::*;
use std::collections::HashSet;

const CHAR_SELECT_BUTTON_WIDTH_PX: WindowUnit = 200.0;
const CHAR_SELECT_BUTTON_HEIGHT_PX: WindowUnit = 40.0;

const LOGO_WIDTH_PCT: WindowUnit = 60.0;
const OPTIONS_WIDTH_PCT: WindowUnit = 100.0 - LOGO_WIDTH_PCT;

const BRAWL_BUTTON_WIDTH_PX: WindowUnit = 300.0;
const BRAWL_BUTTON_HEIGHT_PX: WindowUnit = 40.0;
const BRAWL_COLOUR: &str = "rgb(221, 119, 139)";
const DISABLED_BRAWL_COLOUR: &str = "rgb(226, 165, 177)";

const VS_FONT_SIZE: WindowUnit = 24.0;

#[component]
fn ClassSelector(cx: Scope, options: Vec<Class>, selection: RwSignal<Class>) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();

    view! { cx,
        <div
            style:display="flex"
            style:flex-direction="column"
        >
            {
                options.into_iter()
                    .map(|n| view! { cx,
                        <div
                            style:margin={move || wrap_px(placer_getter.get().scale(4.0))}
                        >
                            <Button
                                background=Signal::derive(cx, move || {
                                    if selection.get() == n {
                                        BUTTON_SELECTED_COLOUR.to_owned()
                                    } else {
                                        BUTTON_NON_SELECTED_COLOUR.to_owned()
                                    }
                                })
                                width=CHAR_SELECT_BUTTON_WIDTH_PX
                                height=CHAR_SELECT_BUTTON_HEIGHT_PX
                                on:click = move |_| {
                                    selection.set(n);
                                }
                            >
                                {format!("{n:?}")}
                            </Button>
                        </div>
                    })
                    .collect_view(cx)
            }
        </div>
    }
}

#[component]
fn MatchupSelector(cx: Scope, hero: RwSignal<Class>, baddie: RwSignal<Class>) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();

    view! { cx,
        <div
            style:display="flex"
            style:flex-direction="row"
        >
            <ClassSelector options={HEROS.clone().into()} selection=hero />
            <div
                style:display="flex"
                style:flex-direction="column"
                style:justify-content="center"
                style:font-size={move || wrap_px(placer_getter.get().scale(VS_FONT_SIZE))}
            >
                VS
            </div>
            <ClassSelector options={BADDIES.clone().into()} selection=baddie />
        </div>
    }
}

const PILE_SELECTOR_WIDTH_PX: WindowUnit = 400.0;

#[component]
fn PileSelector<F>(cx: Scope, on_select: F) -> impl IntoView
where
    F: Fn(Pile) + 'static,
{
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let (raw, set_raw) = create_signal(cx, "".to_string());
    let valid_pile = Signal::derive(cx, move || {
        let parsed_pile = string_to_pile_result(&raw.get());
        if let Ok(pile) = parsed_pile {
            Some(pile)
        } else {
            None
        }
    });

    let button_background = Signal::derive(cx, move || {
        if valid_pile.get().is_none() {
            DISABLED_BRAWL_COLOUR.to_string()
        } else {
            BRAWL_COLOUR.to_string()
        }
    });

    view! { cx,
        <div
            style:display="flex"
            style:flex-direction="column"
            style:justify-content="center"
            style:align-items="center"
        >
            <input
                style:width={move || wrap_px(placer_getter.get().scale(PILE_SELECTOR_WIDTH_PX))}
                style:font-size={move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE))}
                placeholder="Enter pile string"
                on:input=move |ev| {
                    let raw_string = event_target_value(&ev);
                    let valid_filter_chars: HashSet<char> = vec![
                        '0',
                        '1',
                        '2',
                        '3',
                        '4',
                        '5',
                        '6',
                        '7',
                        '8',
                        '9',
                        'a',
                        'A',
                        'b',
                        'B',
                        'c',
                        'C',
                        'd',
                        'D',
                        ',',
                        ' '
                    ].into_iter().collect();

                    let filter_string:String = raw_string.chars().filter(|c| {
                        valid_filter_chars.contains(c)
                    }).collect();
                    set_raw.set(filter_string.clone());
                }
                prop:value=move || {raw.get()}
            />

            <div
                style:height={move || wrap_px(placer_getter.get().scale(4.0))}
            />

            <Button
                background=button_background
                width=BRAWL_BUTTON_WIDTH_PX
                height=BRAWL_BUTTON_HEIGHT_PX
                on:click={move |_| {
                    if let Some(pile) = valid_pile.get() {
                        on_select(pile)
                    }
                }}
                disabled=Signal::derive(cx, move || valid_pile.get().is_none())
            >
                { move || {
                    if let Some(pile) = valid_pile.get() {
                        view!(cx, <span>Start: <PileSpan pile=pile/></span> )
                    } else {
                        if raw.get().len() == 0 {
                            view!(cx, <span>Enter pile for custom start</span>)
                        } else {
                            view!(cx, <span>Could not parse</span>)
                        }
                    }
                }}
            </Button>
        </div>
    }
}

#[component]
pub fn MenuScreen(cx: Scope) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();

    let init_pile = get_start_from_classes(Class::Pyro, Class::Ogre, &mut rand::thread_rng());
    let pile_signal = create_rw_signal(cx, init_pile);

    let is_playing = use_is_playing(cx);

    let hero_signal = create_rw_signal(cx, Class::Warrior);
    let enemy_signal = create_rw_signal(cx, Class::Ogre);

    view! { cx,
        <Show
            when=move || is_playing.with(|s| !s.is_playing)
            fallback=move |cx| view! {cx, <GamePlayer init_pile={pile_signal.get()} /> }
        >
            <div
                style:width="100%"
                style:height="100%"
                style:background="white"
                style:display="flex"
                style:flex-direction="row"
            >
                <div
                    style:width=wrap_pct(OPTIONS_WIDTH_PCT)
                    style:height=wrap_pct(100.0)
                    style:display="flex"
                    style:flex-direction="column"
                    style:justify-content="space-around"
                >
                    <div
                        style:display="flex"
                        style:flex-direction="column"
                        style:justify-content="center"
                        style:align-items="center"
                    >
                        // Matchup Div
                        <MatchupSelector hero=hero_signal baddie=enemy_signal/>

                        <div
                            style:height={move || wrap_px(placer_getter.get().scale(32.0))}
                        />

                        <Button
                            background=Signal::derive(cx, || BRAWL_COLOUR.to_string())
                            width=BRAWL_BUTTON_WIDTH_PX
                            height=BRAWL_BUTTON_HEIGHT_PX
                            on:click=move |_| {
                                let pile = get_start_from_classes(hero_signal.get(), enemy_signal.get(), &mut rand::thread_rng());
                                pile_signal.set(pile);
                                is_playing.update(|s| s.is_playing = true)
                            }
                        >
                            BRAWL
                        </Button>
                    </div>


                    <div
                        style:display="flex"
                        style:flex-direction="column"
                        style:justify-content="center"
                        style:align-items="center"
                    >
                        // Pile Selector Div
                        <PileSelector on_select=move |pile| {
                            pile_signal.set(pile);
                            is_playing.update(|s| s.is_playing = true)
                        }/>
                    </div>
                </div>

                <div
                    style:width=wrap_pct(LOGO_WIDTH_PCT)
                    style:height=wrap_pct(100.0)
                >
                    <img
                        src="static/images/logo-full.webp"
                        style:width=wrap_pct(100.0)
                    />
                </div>
                <div
                    style:position="absolute"
                    style:right="0%"
                    style:bottom="0%"
                >
                    <a
                        href="https://boardgamegeek.com/boardgame/362692/handy-brawl"
                        style:margin={move || wrap_px(placer_getter.get().scale(2.0))}
                    >
                    Handy Brawl Designed by Igor Zuber</a>
                </div>
            </div>
        </Show>
    }
}
