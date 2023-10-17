use handy_core::game::*;

pub fn get_class_full_health_icon_path(class: Class) -> String {
    format!(
        "static/character_icons/{}-full.webp",
        format!("{:?}", class).to_lowercase()
    )
}
