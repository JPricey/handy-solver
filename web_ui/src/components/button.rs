use crate::components::utils::*;
use crate::contexts::*;
use crate::types::*;
use colors_transform::*;
use leptos::logging::log;
use leptos::*;

#[component]
pub fn Button(
    children: Children,
    width: WindowUnit,
    height: WindowUnit,
    background: Signal<String>,
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] font_size: Option<WindowUnit>,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>().unwrap();
    let font_size = font_size.unwrap_or(DEFAULT_FONT_SIZE);

    let disabled_signal = if let Some(disabled_arg) = disabled {
        disabled_arg
    } else {
        Signal::derive(|| false)
    };

    let border_color = move || {
        let background_color = background.get();
        match Rgb::from_hex_str(&background_color) {
            Ok(hex_color) => {
                let lighter = hex_color.lighten(-20.0);
                Some(lighter.to_css_string())
            }
            Err(err) => {
                log!("Failed to parse color {}: {:?}", background_color, err);
                None
            }
        }
    };

    view! {
        <button
            class="standard-button"
            style:position="relative"
            style:border="solid"
            style:border-radius={move || wrap_px(placer_getter.get().scale(BUTTON_BORDER_RADIUS_PX))}
            style:border-width={move || wrap_px(placer_getter.get().scale(BUTTON_BORDER_WIDTH_PX))}
            style:border-color=border_color
            style:width={move || wrap_px(placer_getter.get().scale(width))}
            style:height={move || wrap_px(placer_getter.get().scale(height))}
            style:font-size={move || wrap_px(placer_getter.get().scale(font_size))}
            style:background-color={move || background.get()}
            style:text-align="center"
            style:cursor={move || if !disabled_signal.get() {Some("pointer")} else {None}}
            disabled={move || disabled_signal.get()}
        >
            {children()}
        </button>
    }
}
