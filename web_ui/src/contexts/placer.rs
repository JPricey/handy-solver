use crate::types::*;
use leptos::*;
use crate::components::*;

fn min_f64(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameComponentPlacer {
    scale: f64,
}

impl GameComponentPlacer {
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    pub fn new_from_root_window_size(window_size: WindowSize) -> Self {
        let width_scale = window_size.0 / GOLDEN_WIDTH;
        let height_scale = window_size.1 / GOLDEN_HEIGHT;
        let scale = min_f64(width_scale, height_scale);

        Self::new(scale)
    }

    pub fn scale(&self, size: WindowUnit) -> WindowUnit {
        size * self.scale
    }
}

pub fn get_places(
    placer_getter: Memo<GameComponentPlacer>,
    point: WindowSize,
    width: WindowUnit,
    height: WindowUnit,
) -> Places {
    Places {
        width: Box::new(move || wrap_px(placer_getter.get().scale(width))),
        height: Box::new(move || wrap_px(placer_getter.get().scale(height))),
        x: Box::new(move || wrap_px(placer_getter.get().scale(point.0))),
        y: Box::new(move || wrap_px(placer_getter.get().scale(point.1))),
    }
}

pub fn get_origin_from_window_size(window_size: WindowSize) -> WindowSize {
    let placer = GameComponentPlacer::new_from_root_window_size(window_size);

    let projected_size = scalar_mult(GOLDEN_SIZE, placer.scale);
    let origin = scalar_mult(point_sub(window_size, projected_size), 0.5);
    origin
}

pub struct Places {
    pub width: Box<dyn Fn() -> String>,
    pub height: Box<dyn Fn() -> String>,
    pub x: Box<dyn Fn() -> String>,
    pub y: Box<dyn Fn() -> String>,
}

fn get_current_window_size() -> Option<WindowSize> {
    let cur_window = window();
    let Ok(width_js_value) = cur_window.inner_width() else {
        return None;
    };
    let Some(width) = width_js_value.as_f64() else {
        return None;
    };
    let Ok(height_js_value) = cur_window.inner_height() else {
        return None;
    };
    let Some(height) = height_js_value.as_f64() else {
        return None;
    };

    Some((width, height))
}

#[component]
pub fn PlacerContainer(cx: Scope, children: Children) -> impl IntoView {
    let (window_size_getter, window_size_setter) =
        create_signal(cx, get_current_window_size().unwrap());

    window_event_listener(ev::resize, move |_ev| {
        if let Some(current_window) = get_current_window_size() {
            // let now = performance_now();
            // log!("{now:?} Updating window size {current_window:?}");
            window_size_setter.set(current_window);
        }
    });

    let placer_getter = create_memo(cx, move |_| {
        GameComponentPlacer::new_from_root_window_size(window_size_getter.get())
    });
    provide_context(cx, placer_getter);

    let places = get_places(placer_getter, (0.0, 0.0), GOLDEN_WIDTH, GOLDEN_HEIGHT);
    view! { cx,
        <div
            style:background="#bae8f5"
            style:width="100%"
            style:height="100%"
        >
            <div
                style:position="absolute"
                style:width={places.width}
                style:height={places.height}
                style:left={move || format!("{}px", get_origin_from_window_size(window_size_getter.get()).0)}
                style:top={move || format!("{}px", get_origin_from_window_size(window_size_getter.get()).1)}
                style:background="rgb(248, 238, 226)"
                style:font-size={move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE))}
            >
                {children(cx)}
                // <GamePlayer init_pile={init_pile} />
            </div>
        </div>
    }
}
