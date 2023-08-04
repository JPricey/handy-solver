use crate::components::*;
use crate::contexts::*;
use crate::key_manager::*;
use crate::types::*;
use glam::{DQuat, DVec3, EulerRot};
use handy_core::game::*;
use lazy_static::lazy_static;
use leptos::html::Div;
use leptos::*;

lazy_static! {
    pub static ref NATIVE_CARD_SIZE: WindowSize = (223.0, 312.0);
    pub static ref RENDER_CARD_SIZE: WindowSize = scalar_mult(*NATIVE_CARD_SIZE, 1.1);
}

enum CardSide {
    Front,
    Back,
}

fn flip_card_side(card_side: CardSide) -> CardSide {
    match card_side {
        CardSide::Front => CardSide::Back,
        CardSide::Back => CardSide::Front,
    }
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

#[component]
pub fn InPlayGameCard(
    cx: Scope,
    render_card: RenderCard,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    view! { cx,
        <div
            style:position = "absolute"
            style:top={move || wrap_px(placer_getter.get().scale(render_card.animated_point.get().y))}
            style:left={move || wrap_px(placer_getter.get().scale(render_card.animated_point.get().x))}
        >
            <GameCard
                card_id=render_card.card_id
                quat=render_card.animated_quat
                is_clickable=render_card.is_clickable.into()
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
    let key_manager_getter = get_key_manager_getter(cx);
    let el = create_node_ref::<Div>(cx);
    let is_hovered = use_single_element_hover(cx, el);

    let is_highlighted = create_memo(cx, move |_| {
        let key_manager = key_manager_getter.get();
        is_hovered.get()
            && (key_manager.is_pressed(Key::Control)
                || key_manager.is_pressed(Key::Shift)
                || key_manager.is_pressed(Key::CapsLock))
    });

    let is_highlighted_modified = create_memo(cx, move |_| {
        let key_manager = key_manager_getter.get();
        is_hovered.get() && key_manager.is_pressed(Key::Shift)
    });

    let z_index = move || {
        if is_highlighted.get() {
            "1"
        } else {
            "0"
        }
    };

    let src = move || {
        let quat = quat.get();

        let mut card_side = if is_quat_up(&quat) {
            CardSide::Front
        } else {
            CardSide::Back
        };

        if is_highlighted.get() && key_manager_getter.get().is_pressed(Key::Shift) {
            card_side = flip_card_side(card_side);
        }
        get_card_url(card_id, card_side)
    };

    view! { cx,
        <div
            style:position="relative"
            style:display="grid"
            node_ref=el
            style:z-index={z_index}
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

                return format!("scaleX({scale_x}) rotateX({x}rad) rotateY({y}rad) rotateZ({z}rad)")
            }
        >
            <img
                style:position="absolute"
                style:width="100%"
                style:height="100%"
                src={src}
            />
            <div
                style:position="absolute"
                style:left=INDEX_OFFSET_LEFT
                style:top=INDEX_OFFSET_TOP
                style:font-size={move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE * scale))}
            >
                {
                    if let Some(i) = index {
                        Some(move || format!("({})", (i.get() + 1.0).round()))
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
                        Some(move || format!("({})", (i.get() + 1.0).round()))
                    } else {
                        None
                    }
                }
            </div>
            { children.map(|children| view! { cx,
                <div
                    style:position="absolute"
                    style:width="100%"
                    style:height="100%"
                    style:visibility={move || if is_highlighted_modified.get() {"hidden"} else {"visible"} }
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
                    style:border-radius={move || wrap_px(placer_getter.get().scale(12.0))}
                    style:position="absolute"
                    style:border="none"
                    style:background-color="white"
                    style:cursor="pointer"
                    style:opacity="0.4"
                    style:width="100%"
                    style:height="100%"
                    style:visibility={move || if is_clickable.get() {"visible"} else {"hidden"} }
                />
            </Show>
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
