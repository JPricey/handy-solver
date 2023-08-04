use crate::components::*;
use handy_core::game::*;
use leptos::*;

#[component]
pub fn PileSpan(cx: Scope, pile: Pile) -> impl IntoView {
    view! { cx,
        <span>
            {
                pile
                    .iter()
                    .map(|card_ptr| view! {cx, <CardIdPill card_ptr=*card_ptr />{" "}})
                    .collect_view(cx)
            }
        </span>

    }
}
