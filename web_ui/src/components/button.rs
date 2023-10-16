use crate::components::utils::*;
use crate::contexts::*;
use crate::types::*;
use leptos::*;

#[component]
pub fn Button(
    cx: Scope,
    children: Children,
    width: WindowUnit,
    height: WindowUnit,
    background: Signal<String>,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] font_size: Option<WindowUnit>,
    #[prop(optional)] border_colour: Option<String>,
    #[prop(optional)] border: Option<String>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let font_size = font_size.unwrap_or(DEFAULT_FONT_SIZE);

    let disabled_signal = if let Some(disabled_arg) = disabled {
        disabled_arg
    } else {
        Signal::derive(cx, || false)
    };

    view! { cx,
        <button
            class="standard-button"
            style:position="relative"
            style:border=border.unwrap_or("none".to_owned())
            style:border-color=border_colour
            style:border-radius={move || wrap_px(placer_getter.get().scale(BUTTON_BORDER_RADIUS_PX))}
            style:width={move || wrap_px(placer_getter.get().scale(width))}
            style:height={move || wrap_px(placer_getter.get().scale(height))}
            style:font-size={move || wrap_px(placer_getter.get().scale(font_size))}
            style:background-color={move || background.get()}
            style:text-align="center"
            style:cursor={move || if !disabled_signal.get() {Some("pointer")} else {None}}
            disabled={move || disabled_signal.get()}
        >
            {children(cx)}
        </button>
    }
}
