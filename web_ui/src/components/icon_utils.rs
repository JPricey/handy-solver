use crate::versioning::add_version_to_path;
use handy_core::game::*;

pub fn get_class_full_health_icon_path(class: Class) -> String {
    return add_version_to_path(&format!(
        "static/character_icons/{}-full.webp",
        format!("{:?}", class).to_lowercase()
    ));
}
