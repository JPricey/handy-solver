use leptos::*;
use leptos_animation::*;
use web_ui::contexts::*;
use web_ui::key_manager::*;
use web_ui::menu_screen::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    AnimationContext::provide(cx);
    provide_single_hover_context(cx);
    provide_is_playing(cx);
    register_key_manager(cx);

    let stuff = create_rw_signal::<f64>(cx, 0.0);
    let animated: Signal<f64> =
        create_animated_signal(cx, move || stuff.get().into(), tween_default);
    create_effect(cx, move |_| animated.track());

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
