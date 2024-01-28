use crate::components::*;
use crate::contexts::*;
use crate::types::*;
use leptos::*;

const ICONS_DIR: &str = "static/action_icons/";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionIcon {
    Dodge,
    Block,
}

pub fn get_filename_for_icon(icon: ActionIcon) -> String {
    match icon {
        ActionIcon::Dodge => "dodge".to_owned(),
        ActionIcon::Block => "block".to_owned(),
    }
}

pub fn get_icon_path(icon: ActionIcon) -> String {
    format!("{ICONS_DIR}{}.svg", get_filename_for_icon(icon))
}

#[component]
pub fn IconBadge(cx: Scope, icon: ActionIcon, width: WindowUnit) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>(cx).unwrap();

    view! { cx,
        <img
            style:display="block"
            style:width={move || wrap_px(placer_getter.get().scale(width))}
            src={get_icon_path(icon)}
        />
    }
}
