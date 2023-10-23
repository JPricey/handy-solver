use crate::{components::*, types::WindowUnit};
use handy_core::game::*;
use leptos::*;

const BADGE_SCALE: WindowUnit = 1.0;
const BADGE_VERTICAL_ALIGN_PCT: WindowUnit = -12.0;

#[derive(Clone, Debug)]
pub enum SpanItem {
    Text(String),
    CardPtr(CardPtr),
    CardIndex(usize),
    CardPtrAndIndex(CardPtr, usize),
    RowIndex(usize),
}

#[component]
pub fn TokenSpan(cx: Scope, elements: Vec<SpanItem>) -> impl IntoView {
    let mut spaced_elements: Vec<SpanItem> = vec![elements[0].clone()];

    for item in elements[1..].iter().cloned() {
        spaced_elements.push(SpanItem::Text(" ".to_owned()));
        spaced_elements.push(item);
    }

    view! { cx,
        <span>
            {
                spaced_elements
                    .into_iter()
                    .map(|element|
                         match element {
                            SpanItem::CardPtr(card_ptr) => view! {cx,
                                <span><CardIdPill card_ptr=card_ptr /></span>
                            },
                            SpanItem::CardIndex(card_index) => view! {cx,
                                <span
                                    style:display="inline-block"
                                    style:vertical-align=wrap_pct(BADGE_VERTICAL_ALIGN_PCT)
                                >
                                    <CardIndexBadge number=Signal::derive(cx, move || card_index + 1 ) scale=BADGE_SCALE/>
                                </span>
                            },
                            SpanItem::CardPtrAndIndex(card_ptr, card_index) => view! {cx,
                                <span>
                                    <CardIdPill card_ptr=card_ptr />
                                    <span
                                        style:display="inline-block"
                                        style:vertical-align=wrap_pct(BADGE_VERTICAL_ALIGN_PCT)
                                    >
                                        <CardIndexBadge number=Signal::derive(cx, move || card_index + 1 ) scale=BADGE_SCALE/>
                                    </span>
                                </span>
                            },
                            SpanItem::RowIndex(row_index) => view! {cx,
                                <span
                                    style:display="inline-block"
                                    style:vertical-align=wrap_pct(BADGE_VERTICAL_ALIGN_PCT)
                                >
                                    <RowIndexBadge number=Signal::derive(cx, move || row_index + 1 ) scale=BADGE_SCALE/>
                                </span>
                            },
                            SpanItem::Text(text) => view! {cx,
                                <span>{text}</span>
                            },
                         }
                    )
                    .collect_view(cx)
            }
        </span>

    }
}
