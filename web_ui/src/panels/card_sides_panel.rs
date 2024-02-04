use crate::components::*;
use handy_core::game::*;
use leptos::*;

#[component]
pub fn CardSidesPanel(
    cx: Scope,
    card_id: CardId,
    face_key: FaceKey,
) -> impl IntoView
{
    view! { cx,
        <div
            style:width="100%"
            style:height="100%"
            style:display="flex"
            style:flex-direction="column"
            style:align-items="center"
            style:justify-content="space-evenly"
        >
            <StaticGameCard
                card_id=card_id
                face_key=face_key
                is_clickable=false
                scale=1.0
            />
            <StaticGameCard
                card_id=card_id
                face_key=flip_key(face_key)
                is_clickable=false
                scale=1.0
            />
        </div>
    }
}
