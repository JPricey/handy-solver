use crate::colours::*;
use handy_core::game::*;
use leptos::*;

#[component]
pub fn CardIdPill(card_ptr: CardPtr) -> impl IntoView {
    let card_text = format!("{:?}{:?}", card_ptr.get_card_id(), card_ptr.get_card_face());

    view! { 
        <strong
            style:color={card_ptr_to_hex(card_ptr)}
        >
            {card_text}
        </strong>
    }
}
