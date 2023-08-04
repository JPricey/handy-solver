use crate::components::*;
use crate::types::*;
use handy_core::game::*;
use handy_core::utils::*;
use leptos::*;
use closure::closure;

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
                    <CardIdPill card_ptr=card_ptr />
                    Uses Row #{row_num + 1}
                </span>
            }
        }
        Event::SkipTurn(card_ptr) => {
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr /> Skip Turn
                </span>
            }
        }
        Event::SkipArrow => {
            view! { cx,
                <span>
                    Skip Arrow
                </span>
            }
        }
        Event::BottomCard => {
            view! { cx,
                <span>
                    Done Turn
                </span>
            }
        }
        Event::SkipAction(card_ptr, wrapped_action) => {
            let action_text = action_simple_name(&wrapped_action);
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> Skips {action_text}
                </span>
            }
        }
        Event::AttackCard(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Attack <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Damage(card_idx, card_ptr, hit_type, result_face) => {
            let new_card_ptr = CardPtr::new_from_id(card_ptr.get_card_id(), result_face);
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> {format!("({}) Hit by {:?}", card_idx+1, hit_type)} to <CardIdPill card_ptr=new_card_ptr/>
                </span>
            }
        }
        Event::Death => {
            view! { cx,
                <span>
                    Death
                </span>
            }
        }
        Event::Void(card_idx, card_ptr, face_key) => {
            let new_card_ptr = CardPtr::new_from_id(card_ptr.get_card_id(), face_key);
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> {format!("({}) Voided to", card_idx + 1)} <CardIdPill card_ptr=new_card_ptr/>
                </span>
            }
        }
        Event::Inspire(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Inspire <CardIdPill card_ptr=card_ptr/>{format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::MoveTarget(card_idx, card_ptr, move_type) => {
            view! { cx,
                <span>
                    {format!("{:?} ", move_type)}<CardIdPill card_ptr=card_ptr/>{format!("({})", card_idx + 1)}
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
                    {format!("{:?} by {} {} ", move_type, amount, verb)} <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::MoveResult(move_type, amount) => {
            view! { cx,
                <span>
                    {format!("Execute {:?} {}", move_type, amount)}
                </span>
            }
        }
        Event::Pull(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Pull <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Push(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Push <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Teleport(card_idx1, card_ptr1, card_idx2, card_ptr2) => {
            view! { cx,
                <span>
                    Teleport <CardIdPill card_ptr=card_ptr1/>{format!("({})", card_idx1 + 1)} - <CardIdPill card_ptr=card_ptr2/>{format!("({})", card_idx2 + 1)}
                </span>
            }
        }
        Event::Mandatory(card_ptr, self_action) => {
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> Forced {format!("{:?}", self_action)}
                </span>
            }
        }
        Event::FireballTarget(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Target Fireball <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Ablaze(card_idx1, card_ptr1, card_idx2, card_ptr2) => {
            view! { cx,
                <span>
                    Ablaze <CardIdPill card_ptr=card_ptr1/>{format!("({})", card_idx1 + 1)} - <CardIdPill card_ptr=card_ptr2/>{format!("({})", card_idx2 + 1)}
                </span>
            }
        }
        Event::Heal(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Heal <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Revive(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Revive <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Manouver(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    Manouver <CardIdPill card_ptr=card_ptr/> {format!("({})", card_idx + 1)}
                </span>
            }
        }
        Event::Block(card_idx, card_ptr, cost) => {
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> {format!("({}) Blocks: {:?}", card_idx + 1, cost)}
                </span>
            }
        }
        Event::Dodge(card_idx, card_ptr, cost) => {
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> {format!("({}) Dodges: {:?}", card_idx, cost)}
                </span>
            }
        }
        Event::OnHurt(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> {format!("({}) Hurt", card_idx + 1)}
                </span>
            }
        }
        Event::PayEnergy(cards) => {
            let cards = cards
                .iter()
                .map(|(card_idx, card_ptr)| view! {cx, <CardIdPill card_ptr=*card_ptr /> {format!("({})", card_idx+1)}})
                .collect_view(cx);
            view! {cx,
                <span>
                    Pay {cards}
                </span>
            }
        }
        Event::Swarm(card_idx, card_ptr) => {
            view! { cx,
                <span>
                    <CardIdPill card_ptr=card_ptr/> {format!("({}) Swarms", card_idx + 1)}
                </span>
            }
        }
        Event::UseActionAssist(assist_idx, assist_card_ptr, assist_row_idx) => {
            let text = format!(
                "{:?}@{}: Assist Row {}",
                assist_card_ptr,
                assist_idx + 1,
                assist_row_idx + 1
            );
            view! { cx,
                <span>
                    {text}
                </span>
            }
        }

        Event::ReactAssistUsed(card_idx, card_ptr, trigger, cost) => {
            let text = format!("{:?}({}): {:?} React Assited({:?})", card_ptr, card_idx + 1, trigger, cost);
            view! { cx,
                <span>
                    {text}
                </span>
            }
        }

        Event::SkipReactActionAssist => {
            view! { cx,
                <span>
                    Skip React Action
                </span>
            }
        }
    }
}
