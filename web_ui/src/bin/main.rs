use leptos::*;
use leptos_animation::*;
use web_ui::contexts::*;
use web_ui::key_manager::*;
use web_ui::menu_screen::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    AnimationContext::provide(cx);
    provide_single_hover_context(cx);
    provide_options(cx);
    register_key_manager(cx);

    // There seems to be a bug in leptos_animation where when all animated signals are destroyed
    // future signals will stall as well.
    // We create this useless animation so that there's an animated signal around at all times
    let animation_hack: Signal<f64> = create_animated_signal(cx, move || (0.0).into(), tween_default);
    create_effect(cx, move |_| animation_hack.track());

    view! { cx,
        <PlacerContainer>
            <MenuScreen />
        </PlacerContainer>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx,  <App /> })
}
