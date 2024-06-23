use handy_core::game::*;

pub fn main_colour_for_class(class: Class) -> &'static str {
    match class {
        Class::Dummy => "#000000",
        Class::Warrior => "#008A97",
        Class::Huntress => "#00813A",
        Class::Pyro => "#F08319",
        Class::Cursed => "#EDE387",
        Class::Beastmaster => "#4E3F30",
        Class::Assassin => "#CA4E96",
        Class::Monk => "#EF9C66",
        Class::Ogre => "#7B4627",
        Class::Vampire => "#4D897C",
        Class::Spider => "#746991",
        Class::Demon => "#942A3D",
        Class::Flora => "#203176",
        Class::Wall => "#7B4627",
        Class::Piper => "#4673A6",
        Class::Troupe => "#FA2A13",
        Class::Ooze => "#4D9116",
    }
}

pub const ICON_BLACK_HEX_COLOUR: &str = "#2b2a29";
pub const ICON_WHITE_HEX_COLOUR: &str = "#FEFEFE";

pub fn card_ptr_to_hex(card_ptr: CardPtr) -> &'static str {
    let class = card_ptr.get_card_def().class;

    match class {
        Class::Cursed => match card_ptr.get_active_face().allegiance {
            Allegiance::Hero => "#b9af77",
            _ => "#1a150f",
        },
        Class::Piper => match card_ptr.get_active_face().allegiance {
            Allegiance::Hero => main_colour_for_class(class),
            _ => "#636262",
        },
        _ => main_colour_for_class(class),
    }
}
