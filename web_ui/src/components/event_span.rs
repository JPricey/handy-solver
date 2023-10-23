use crate::components::*;
use crate::types::*;
use closure::closure;
use handy_core::game::*;
use handy_core::utils::*;
use leptos::*;

fn format_cost(cost: &Option<SelfAction>) -> String {
    match cost {
        None => "".to_owned(),
        Some(action) => format!(" ({})", action),
    }
}

#[component]
pub fn FrameSpan(cx: Scope, frame: GameFrame) -> impl IntoView {
    let current_pile = frame.current_pile.clone();

    view! { cx,
        {
            closure!(clone current_pile, || {
                let maybe_last_event = frame.event_history.last().clone();
                if let Some(last_event) = maybe_last_event {
                    view! {cx, <span><EventSpan event=last_event.clone() /></span> }
                } else {
                    view! {cx,
                        <span
                            style:display="flex"
                        >
                            <span
                                style:flex="1"
                            >
                                <CardIdPill card_ptr=current_pile[0].clone() />
                                Activates
                            </span>

                            <PileSpan pile=current_pile.clone() />
                        </span>

                    }
                }
            })
        }
    }
}

#[component]
pub fn EventSpan(cx: Scope, event: Event) -> impl IntoView {
    match event {
        Event::PickRow(row_num, _, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtr(card_ptr),
                            SpanItem::Text("Use Row".to_owned()),
                            SpanItem::RowIndex(row_num),
                        ]
                    />
                </span>
            }
        }
        Event::SkipTurn(card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtr(card_ptr),
                            SpanItem::Text("Skip Turn".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::SkipArrow => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Skip Arrow".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::BottomCard => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("End Turn".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::SkipAction(card_ptr, wrapped_action) => {
            let action_text = action_simple_name(&wrapped_action);
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtr(card_ptr),
                            SpanItem::Text(format!("Skip {action_text}")),
                        ]
                    />
                </span>
            }
        }
        Event::AttackCard(card_idx, card_ptr, hit_type) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Attack".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text(format!("({})",  hit_type)),
                        ]
                    />
                </span>
            }
        }
        Event::Damage(card_idx, card_ptr, hit_type, result_face) => {
            let new_card_ptr = CardPtr::new_from_id(card_ptr.get_card_id(), result_face);
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text(format!("Damaged by {}",  hit_type)),
                            SpanItem::CardPtr(new_card_ptr),
                        ]
                    />
                </span>
            }
        }
        Event::Death => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Death".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::Void(card_idx, card_ptr, face_key) => {
            let new_card_ptr = CardPtr::new_from_id(card_ptr.get_card_id(), face_key);
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(new_card_ptr, card_idx),
                            SpanItem::Text("Void".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::Inspire(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Inspire".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::MoveTarget(card_idx, card_ptr, move_type) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text(format!("{} ", move_type)),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::MoveBy(card_idx, card_ptr, move_type, amount) => {
            let verb = match move_type {
                MoveType::Quicken => "Over",
                MoveType::Delay => "Under",
            };
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text(format!("{} by {} {} ", move_type, amount, verb)),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::MoveResult(move_type, amount) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text(format!("Execute {} {}", move_type, amount)),
                        ]
                    />
                </span>
            }
        }
        Event::Pull(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Pull".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::Push(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Push".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::Teleport(card_idx1, card_ptr1, card_idx2, card_ptr2) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Teleport".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr1, card_idx1),
                            SpanItem::Text("-".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr2, card_idx2),
                        ]
                    />
                </span>
            }
        }
        Event::Mandatory(card_ptr, self_action) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtr(card_ptr),
                            SpanItem::Text(format!("Forced {}", self_action)),
                        ]
                    />
                </span>
            }
        }
        Event::FireballTarget(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Target Fireball".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::Ablaze(card_idx1, card_ptr1, card_idx2, card_ptr2) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Ablaze".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr1, card_idx1),
                            SpanItem::Text("-".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr2, card_idx2),
                        ]
                    />
                </span>
            }
        }
        Event::Heal(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Heal".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::Revive(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Revive".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::Manouver(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Manouver".to_owned()),
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                        ]
                    />
                </span>
            }
        }
        Event::Block(card_idx, card_ptr, cost) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text(format!("Blocks{}", format_cost(&cost))),
                        ]
                    />
                </span>
            }
        }
        Event::Dodge(card_idx, card_ptr, cost) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text(format!("Dodges{}", format_cost(&cost))),
                        ]
                    />
                </span>
            }
        }
        Event::OnHurt(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text("Hurt".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::PayEnergy(cards) => {
            let mut elems = vec![SpanItem::Text("Pay".to_owned())];
            for (card_idx, card_ptr) in cards {
                elems.push(SpanItem::CardPtrAndIndex(card_ptr,card_idx));
            }

            view! {cx,
                <span>
                    <TokenSpan elements=elems />
                </span>
            }
        }
        Event::Swarm(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text("Swarm".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::UseActionAssistCard(assist_idx, assist_card_ptr) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(assist_card_ptr, assist_idx),
                            SpanItem::Text("Assists".to_owned()),
                        ]
                    />
                </span>
            }
        }
        Event::UseActionAssistRow(assist_idx, assist_card_ptr, assist_row_idx) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(assist_card_ptr, assist_idx),
                            SpanItem::Text("Use Assist Row".to_owned()),
                            SpanItem::RowIndex(assist_row_idx),
                        ]
                    />
                </span>
            }
        }

        Event::ReactAssistUsed(card_idx, card_ptr, trigger, cost) => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::CardPtrAndIndex(card_ptr, card_idx),
                            SpanItem::Text(format!("Assists Reaction {} ({})", trigger, cost)),
                        ]
                    />
                </span>
            }
        }

        Event::SkipReactActionAssist => {
            view! { cx,
                <span>
                    <TokenSpan
                        elements=vec![
                            SpanItem::Text("Skip React Action".to_owned()),
                        ]
                    />
                </span>
            }
        }
    }
}
