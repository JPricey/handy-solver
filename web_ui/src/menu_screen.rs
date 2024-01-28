use crate::class_helpers::*;
use crate::components::*;
use crate::contexts::*;
use crate::game_player::*;
use crate::init_pile_provider::InitPileProvider;
use crate::init_pile_provider::*;
use crate::types::*;
use handy_core::game::primitives::*;
use handy_core::game::Class;
use handy_core::solver::{BADDIES, HEROS};
use handy_core::utils::pile_utils::*;
use leptos::*;
use web_sys::Navigator;
use std::collections::HashSet;
use url::Url;

const CHAR_SELECT_BUTTON_WIDTH_PX: WindowUnit = 200.0;
const CHAR_SELECT_BUTTON_HEIGHT_PX: WindowUnit = 40.0;

const LOGO_WIDTH_PCT: WindowUnit = 60.0;
const OPTIONS_WIDTH_PCT: WindowUnit = 100.0 - LOGO_WIDTH_PCT;

const BRAWL_BUTTON_WIDTH_PX: WindowUnit = 320.0;
const BRAWL_BUTTON_HEIGHT_PX: WindowUnit = 40.0;
const BRAWL_COLOUR: &str = "#e2a5b1";

const VS_FONT_SIZE: WindowUnit = 24.0;
const SELECT_FONT_SIZE: WindowUnit = 24.0;

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
                    .map(|class| {
                        let icon_path = get_class_full_health_icon_path(class);

                        view! { cx,
                        <div
                            style:margin-top={move || wrap_px(placer_getter.get().scale(4.0))}
                        >
                            <Button
                                background=Signal::derive(cx, move || {
                                    if selection.get() == class {
                                        BUTTON_SELECTED_COLOUR.to_owned()
                                    } else {
                                        BUTTON_NON_SELECTED_COLOUR.to_owned()
                                    }
                                })
                                width=CHAR_SELECT_BUTTON_WIDTH_PX
                                height=CHAR_SELECT_BUTTON_HEIGHT_PX
                                on:click = move |_| {
                                    selection.set(class);
                                }
                            >
                                <div
                                    style:display="flex"
                                    style:flex-direction="row"
                                    style:height="100%"
                                >
                                    <div
                                        style:flex-grow=0
                                        style:height="100%"
                                        style:display="flex"
                                        style:justify-content="center"
                                        style:align-items="center"
                                    >
                                        <img
                                            style:height="90%"
                                            style:margin-left={move || wrap_px(placer_getter.get().scale(4.0))}
                                            src={icon_path}
                                        />
                                    </div>
                                    <div
                                        style:flex-grow=1

                                        style:display="flex"
                                        style:flex-direction="column"
                                        style:justify-content="center"
                                        style:align-items="center"
                                    >
                                        <div
                                        >
                                            {class_to_character_name(class)}
                                        </div>
                                        <div>
                                            The {class_to_full_class_name(class)}
                                        </div>
                                    </div>
                                </div>
                            </Button>
                        </div>
                    }})
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
                style:font-size={move || wrap_px(placer_getter.get().scale(SELECT_FONT_SIZE))}
                style:padding={move || wrap_px(placer_getter.get().scale(8.0))}
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
                // background=button_background
                background=Signal::derive(cx, || BRAWL_COLOUR.to_string())
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

fn get_query_param_pile() -> Option<Pile> {
    let Ok(full_url) = document().url() else {
        return None;
    };

    let Ok(parsed_url) = Url::parse(&full_url) else {
        return None;
    };

    for (k, v) in parsed_url.query_pairs() {
        log!("{:?}", (&k, &v));

        if k == "pile" {
            let parsed_pile = string_to_pile_result(&v);

            return if let Ok(pile) = parsed_pile {
                Some(pile)
            } else {
                None
            };
        }
    }

    return None;
}

#[component]
pub fn MenuScreen(cx: Scope) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let is_playing = create_rw_signal(cx, false);

    let hero_signal = create_rw_signal(cx, Class::Warrior);
    let enemy_signal = create_rw_signal(cx, Class::Ogre);

    let pile_provider_signal: RwSignal<Box<dyn InitPileProvider>> = create_rw_signal(
        cx,
        Box::new(MatchupPileProvider {
            matchup: (hero_signal.get_untracked(), enemy_signal.get_untracked()),
        }),
    );

    if let Some(query_param_pile) = get_query_param_pile() {
        pile_provider_signal.set(Box::new(ExactPileProvider {
            pile: query_param_pile,
        }));
        is_playing.set(true);
    }

    view! { cx,
        <Show
            when=move || !is_playing.get()
            fallback=move |cx| view! {cx, <GamePlayer init_pile_provider={pile_provider_signal.get()} is_playing=is_playing /> }
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
                        <div
                            style:font-size={move || wrap_px(placer_getter.get().scale(VS_FONT_SIZE))}
                        >
                            Select a Matchup
                        </div>

                        <MatchupSelector hero=hero_signal baddie=enemy_signal/>

                        <div
                            style:height={move || wrap_px(placer_getter.get().scale(32.0))}
                        />

                        <Button
                            background=Signal::derive(cx, || BRAWL_COLOUR.to_string())
                            width=BRAWL_BUTTON_WIDTH_PX
                            height=BRAWL_BUTTON_HEIGHT_PX
                            on:click=move |_| {
                                pile_provider_signal.set(Box::new(MatchupPileProvider {
                                    matchup: (hero_signal.get(), enemy_signal.get())
                                }));

                                is_playing.set(true)
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
                        <div
                            style:font-size={move || wrap_px(placer_getter.get().scale(VS_FONT_SIZE))}
                        >
                            Or Use a Custom Start
                        </div>
                        <PileSelector on_select=move |pile| {
                            pile_provider_signal.set(Box::new(ExactPileProvider {
                                pile,
                            }));
                            is_playing.set(true)
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
                    style:margin={move || wrap_px(placer_getter.get().scale(2.0))}
                >
                    <div>
                        <a href="https://boardgamegeek.com/boardgame/362692/handy-brawl">Handy Brawl</a>
                        game designed by
                        <a href="https://boardgamegeek.com/boardgamedesigner/145462/igor-zuber" >Igor Zuber</a>
                    </div>
                    <div>
                        Art by
                        <a href="https://boardgamegeek.com/boardgameartist/116088/aleksander-jagodzinski">Aleksander Jagodziński</a>
                        and
                        <a href="https://boardgamegeek.com/boardgameartist/145463/weronika-kaluza">Weronika Kałuża</a>
                    </div>
                    <div>
                        <a href="https://github.com/JPricey/handy-solver"> Implemented by Joe Price</a>
                    </div>
                </div>
            </div>
        </Show>
    }
}
