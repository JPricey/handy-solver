use crate::components::*;
use crate::constants::*;
use crate::contexts::*;
use crate::types::*;
use glam::{DQuat, DVec3, EulerRot};
use handy_core::game::*;
use lazy_static::lazy_static;
use leptos::html::Div;
use leptos::*;
use regex::Regex;

lazy_static! {
    pub static ref NATIVE_CARD_SIZE: WindowSize = (223.0, 312.0);
    pub static ref RENDER_CARD_SIZE: WindowSize = scalar_mult(*NATIVE_CARD_SIZE, 1.2);
}

enum CardSide {
    Front,
    Back,
}

fn card_side_to_str(card_side: CardSide) -> &'static str {
    match card_side {
        CardSide::Front => "f",
        CardSide::Back => "b",
    }
}

fn get_card_url(card_id: CardId, card_side: CardSide) -> String {
    format!(
        "static/cards/{}{}.webp",
        card_id,
        card_side_to_str(card_side)
    )
}

fn is_quat_up(quat: &DQuat) -> bool {
    let upv = DVec3::Z;
    let res = quat.mul_vec3(upv);

    res.z >= 0.0
}

const INDEX_OFFSET_LEFT: &str = "0.6%";
const INDEX_OFFSET_TOP: &str = "0.6%";

pub fn card_id_to_id_str(card_id: CardId) -> String {
    format!("in-play-card-{}", card_id)
}

pub fn try_card_string_to_id(card_id_string: String) -> Option<CardId> {
    if card_id_string.len() == 0 {
        return None;
    }
    let re = Regex::new(r"^in-play-card-(\d+)$").unwrap();
    if let Some(id_capture) = re.captures(&card_id_string) {
        let id_str = id_capture.get(1).unwrap().as_str();
        if let Ok(id) = id_str.parse::<CardId>() {
            return Some(id);
        }
    }

    None
}

#[component]
pub fn InPlayGameCard(
    cx: Scope,
    render_card: RenderCard,
    is_animating: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let card_div_id = format!("in-play-card-{}", render_card.card_id);

    view! { cx,
        <div
            id=card_div_id
            style:position="absolute"
            style:top=move || wrap_px(placer_getter.get().scale(render_card.animated_point.get().y))
            style:left=move || wrap_px(placer_getter.get().scale(render_card.animated_point.get().x))
            style:z-index=move || render_card.z_index.get()
        >
            <GameCard
                card_id=render_card.card_id
                quat=render_card.animated_quat
                is_clickable=Signal::derive(cx, move || !is_animating.get() && render_card.is_clickable.get())
                scale=1.0
                index=Some(render_card.animated_position_in_pile)
                children=children
            />
        </div>
    }
}

#[component]
pub fn GameCard(
    cx: Scope,
    card_id: CardId,
    quat: Signal<DQuat>,
    scale: f64,
    is_clickable: Signal<bool>,
    index: Option<Signal<WindowUnit>>,
    #[prop(optional_no_strip)] children: Option<Children>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let el = create_node_ref::<Div>(cx);

    let src = move || {
        let quat = quat.get();

        let card_side = if is_quat_up(&quat) {
            CardSide::Front
        } else {
            CardSide::Back
        };

        get_card_url(card_id, card_side)
    };

    view! { cx,
        <div
            style:position="relative"
            style:display="grid"
            node_ref=el
            style:width={move || wrap_px(placer_getter.get().scale(RENDER_CARD_SIZE.0 * scale))}
            style:height={move || wrap_px(placer_getter.get().scale(RENDER_CARD_SIZE.1 * scale))}
            style:transform=move || {
                let quat = quat.get();
                let (x, y, z) = quat.to_euler(EulerRot::XYZ);
                let scale_x = if is_quat_up(&quat) {
                    1
                } else {
                    -1
                };

                format!("scaleX({scale_x}) rotateX({x}rad) rotateY({y}rad) rotateZ({z}rad)")
            }
        >
            <img
                style:position="absolute"
                style:width="100%"
                style:height="100%"
                src={src}
            />
            { children.map(|children| view! { cx,
                <div
                    style:position="absolute"
                    style:width="100%"
                    style:height="100%"
                >
                    {children(cx)}
                </div>
                })
            }
            <Show
                when=move || is_clickable.get()
                fallback=move |_| ()
            >
                <button
                    class="clickable-option-overlay"
                    style:border-radius={move || wrap_px(placer_getter.get().scale(11.0))}
                    style:border-width={move || wrap_px(placer_getter.get().scale(SELECTABLE_BUTTON_WIDTH_PX))}
                    style:position="absolute"
                    style:visibility={move || if is_clickable.get() {"visible"} else {"hidden"} }
                />
            </Show>
            <div
                style:position="absolute"
                style:left=INDEX_OFFSET_LEFT
                style:top=INDEX_OFFSET_TOP
                style:font-size={move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE * scale))}
            >
                {
                    if let Some(i) = index {
                        Some(view! { cx,
                            <CardIndexBadge
                                number=Signal::derive(cx, move || (i.get() + 1.0).round() as usize)
                                is_foreground=is_clickable
                            />
                        })
                    } else {
                        None
                    }
                }
            </div>
            <div
                style:position="absolute"
                style:right=INDEX_OFFSET_LEFT
                style:bottom=INDEX_OFFSET_TOP
                style:font-size={move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE * scale))}
                style:transform="rotate(180deg)"
            >
                {
                    if let Some(i) = index {
                        Some(view! { cx,
                            <CardIndexBadge
                                number=Signal::derive(cx, move || (i.get() + 1.0).round() as usize)
                                is_foreground=is_clickable
                            />
                        })
                    } else {
                        None
                    }
                }
            </div>
        </div>
    }
}

#[component]
pub fn StaticGameCard(
    cx: Scope,
    card_id: CardId,
    face_key: FaceKey,
    is_clickable: bool,
    scale: f64,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let quat = quat_for_face(face_key);
    view! { cx,
        <GameCard
            card_id=card_id
            scale=scale
            quat = Signal::derive(cx, move || quat)
            is_clickable=Signal::derive(cx, move || is_clickable)
            index=None
            children=children
        />
    }
}
