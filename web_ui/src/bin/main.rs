use leptos::*;
use leptos_animation::*;
use web_ui::contexts::*;
use web_ui::key_manager::*;
use web_ui::menu_screen::*;

#[component]
fn App() -> impl IntoView {
    AnimationContext::provide();
    provide_single_hover_context();
    provide_options();
    register_key_manager();

    // There seems to be a bug in leptos_animation where when all animated signals are destroyed
    // future signals will stall as well.
    // We create this useless animation so that there's an animated signal around at all times
    let animation_hack: AnimatedSignal<f64, f64> =
        create_animated_signal(move || (0.0).into(), tween_default);
    create_effect(move |_| animation_hack.track());

    view! {
        <PlacerContainer>
            <MenuScreen />
        </PlacerContainer>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! {   <App /> })
}
