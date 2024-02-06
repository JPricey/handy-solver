use crate::colours::*;
use crate::components::*;
use crate::contexts::*;
use crate::types::*;
use handy_core::game::Class;
use handy_core::game::Target;
use handy_core::game::WrappedAction;
use leptos::*;

const ICONS_DIR: &str = "static/action_icons/";

fn get_full_icon_path(icon_spec: &IconSpec) -> String {
    format!("{ICONS_DIR}{}", icon_spec.filename)
}

pub enum ActionIconBackgroundColor {
    Any,
    Enemy,
    Ally(Class),
}

fn get_background_colour(target: Target, class: Class) -> &'static str {
    match target {
        Target::Any => ICON_WHITE_HEX_COLOUR,
        Target::Enemy => ICON_BLACK_HEX_COLOUR,
        Target::Ally => main_colour_for_class(class),
    }
}

fn get_invert(target: Target, class: Class) -> bool {
    match target {
        Target::Any => false,
        Target::Enemy => true,
        Target::Ally => match class {
            Class::Cursed => false,
            _ => true,
        },
    }
}

// TODO: take a target, range
#[component]
pub fn IconBadge(
    cx: Scope,
    action: WrappedAction,
    actor: Class,
    scale: WindowUnit,
) -> impl IntoView {
    let background_colour = get_background_colour(action.target, actor);
    let is_inverted = get_invert(action.target, actor);
    let (icon_type, _range) = action_to_icon_type_and_range(action.action);
    view! { cx,
        <InnerIconBadge
            scale=scale
            background_colour=background_colour
            icon_type=icon_type
            is_inverted=is_inverted
        />
    }
}

// TODO: take a range
// TODO: reverse colours when target is any
#[component]
pub fn InnerIconBadge(
    cx: Scope,
    scale: WindowUnit,
    icon_type: IconType,
    background_colour: &'static str,
    is_inverted: bool,
) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();
    let icon_spec = icon_type.value();
    let inverted_result = if is_inverted {
        Some("invert(100%)")
    } else {
        None
    };

    view! { cx,
        <div
            style:position="relative"
            style:width={move || wrap_px(placer_getter.get().scale(icon_spec.native_size.0 * scale))}
            style:height={move || wrap_px(placer_getter.get().scale(icon_spec.native_size.1 * scale))}
        >
            <svg
                style:position="absolute"
                style:left="0%"
                style:top="0%"
                xmlns="http://www.w3.org/2000/svg"
            >
                <circle
                    cx=move || placer_getter.get().scale(icon_spec.background_point.0 * scale)
                    cy=move || placer_getter.get().scale(icon_spec.background_point.1 * scale)
                    r=move || placer_getter.get().scale(icon_spec.background_rad * scale)
                    fill=background_colour
                />
            </svg>
            <img
                style:position="absolute"
                style:left="0%"
                style:top="0%"
                style:width={move || wrap_px(placer_getter.get().scale(icon_spec.native_size.0 * scale))}
                style:height={move || wrap_px(placer_getter.get().scale(icon_spec.native_size.1 * scale))}
                src={get_full_icon_path(&icon_spec)}
                style:filter=inverted_result
            />
        </div>
    }
}
