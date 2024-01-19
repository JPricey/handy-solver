use handy_core::game::{Class, *};
use leptos::*;

fn card_ptr_to_hex(card_ptr: CardPtr) -> String {
    let class = card_ptr.get_card_def().class;

    match class {
        Class::Warrior => "#008a97".to_owned(),
        Class::Huntress => "#007d30".to_owned(),
        Class::Pyro => "#ee7f00".to_owned(),
        Class::Cursed => {
            // dark area: b9af77
            // light border: ede384
            match card_ptr.get_active_face().allegiance {
                Allegiance::Hero => "#b9af77".to_owned(),
                _ => "#1a150f".to_owned(),
            }
        }
        Class::Beastmaster => "#463723".to_owned(),
        Class::Assassin => "#ca4f96".to_owned(),
        Class::Ogre => "#7b4627".to_owned(),
        Class::Vampire => "#478577".to_owned(),
        Class::Spider => "#726490".to_owned(),
        Class::Demon => "#921833".to_owned(),
        Class::Flora => "#203176".to_owned(),
        Class::Wall => "#463723".to_owned(),
        // _ => "#000000".to_owned(),
    }
}

#[component]
pub fn CardIdPill(cx: Scope, card_ptr: CardPtr) -> impl IntoView {
    let card_text = format!("{:?}{:?}", card_ptr.get_card_id(), card_ptr.get_card_face());

    view! { cx,
        <strong
            style:color={card_ptr_to_hex(card_ptr)}
        >
            {card_text}
        </strong>
    }
}
