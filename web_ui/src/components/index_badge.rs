use crate::components::*;
use crate::contexts::*;
use crate::types::*;
use leptos::*;

const DIAMETER_PX: WindowUnit = 14.0;
const BORDER_WIDTH_PX: WindowUnit = 1.2;
const TRUE_RAD_PX: WindowUnit = DIAMETER_PX / 2.0 - BORDER_WIDTH_PX;
const BADGE_FONT_SIZE_PX: WindowUnit = 10.0;

#[component]
pub fn CardIndexBadge(
    number: Signal<usize>,
    #[prop(optional)] scale: Option<WindowUnit>,
    #[prop(optional)] is_foreground: Option<Signal<bool>>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>().unwrap();
    let scale = scale.unwrap_or(1.0);

    let opacity_fn = move || {
        if is_foreground.map_or(true, |s| s.get()) {
            None
        } else {
            Some("50%")
        }
    };

    view! { 
        <svg
            style:display="block"
            style:width={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale))}
            style:height={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale))}
            style:opacity=opacity_fn
        >
            <circle
                cx={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale / 2.0))}
                cy={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale / 2.0))}
                r={move || wrap_px(placer_getter.get().scale(TRUE_RAD_PX * scale))}
                stroke-width={move || wrap_px(placer_getter.get().scale(BORDER_WIDTH_PX * scale))}
                stroke="white"
                fill="#333"
            />
            <text
                x="50%"
                y="50%"
                dominant-baseline="central"
                text-anchor="middle"
                font-size={move || wrap_px(placer_getter.get().scale(BADGE_FONT_SIZE_PX))}
                fill="white"
                font-weight="bold"
            >
                {move || number.get()}
            </text>
        </svg>
    }
}

#[component]
pub fn RowIndexBadge(
    number: Signal<usize>,
    #[prop(optional)] scale: Option<WindowUnit>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>().unwrap();
    let scale = scale.unwrap_or(1.0);

    view! { 
        <svg
            style:display="block"
            style:width={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale))}
            style:height={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale))}
        >
            <circle
                cx={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale / 2.0))}
                cy={move || wrap_px(placer_getter.get().scale(DIAMETER_PX * scale / 2.0))}
                r={move || wrap_px(placer_getter.get().scale(TRUE_RAD_PX * scale))}
                stroke-width={move || wrap_px(placer_getter.get().scale(BORDER_WIDTH_PX * scale))}
                stroke="black"
                fill="white"
            />
            <text
                x="50%"
                y="50%"
                dominant-baseline="central"
                text-anchor="middle"
                font-size={move || wrap_px(placer_getter.get().scale(BADGE_FONT_SIZE_PX))}
                fill="black"
                font-weight="bold"
            >
                {move || number.get()}
            </text>
        </svg>
    }
}
