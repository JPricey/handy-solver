use crate::components::*;
use crate::contexts::*;
use handy_core::game::*;
use leptos::*;

#[component]
pub fn PileSpan( pile: Pile) -> impl IntoView {
    let placer_getter = use_context::<Memo<GameComponentPlacer>>().unwrap();
    view! { 
        <span>
            {
                pile
                    .iter()
                    .enumerate()
                    .map(|(i, card_ptr)| {
                         let width = match i >= pile.len() - 1 {
                             true => 0.0,
                             false => 1.2,
                         };
                        return view! {  <span>
                            <CardIdPill card_ptr=*card_ptr />
                            <span
                                style:display="inline-block"
                                style:width={move || wrap_px(placer_getter.get().scale(width))}
                            />
                        </span> }
                    })
                    .collect_view()
            }
        </span>

    }
}
