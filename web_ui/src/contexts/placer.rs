use crate::components::*;
use crate::types::*;
use leptos::*;
use regex::Regex;

fn min_f64(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameComponentPlacer {
    pub scale: WindowUnit,
    pub golden_width: WindowUnit,
    pub is_mobile: bool,
    pub is_rotated: bool,
}

impl GameComponentPlacer {
    pub fn new(
        scale: WindowUnit,
        golden_width: WindowUnit,
        is_mobile: bool,
        is_rotated: bool,
    ) -> Self {
        Self {
            scale,
            golden_width,
            is_mobile,
            is_rotated,
        }
    }

    pub fn new_from_root_window_size(window_size: WindowSize, is_mobile: bool) -> Self {
        let (mut width, mut height) = window_size;

        let should_rotate = is_mobile && height > width * 1.5;

        if should_rotate {
            (width, height) = (height, width);
        }

        let width_scale = width / GOLDEN_MIN_WIDTH;
        let height_scale = height / GOLDEN_HEIGHT;
        let scale = min_f64(width_scale, height_scale);

        let computed_golden_width = width / scale;
        let golden_width = min_f64(computed_golden_width, GOLDEN_MAX_WIDTH);

        Self::new(scale, golden_width, is_mobile, should_rotate)
    }

    pub fn scale(&self, size: WindowUnit) -> WindowUnit {
        size * self.scale
    }
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

pub fn compute_is_mobile() -> bool {
    let user_agent_string = format!(
        "{}",
        window().navigator().user_agent().unwrap_or("".to_owned())
    );
    let re = Regex::new(r"/Mobile|iP(hone|od|ad)|Android|BlackBerry|IEMobile/").unwrap();

    re.is_match(&user_agent_string)
}

#[component]
pub fn PlacerContainer( children: Children) -> impl IntoView {
    let (window_size_getter, window_size_setter) =
        create_signal( get_current_window_size().unwrap());

    window_event_listener(ev::resize, move |_ev| {
        if let Some(current_window) = get_current_window_size() {
            window_size_setter.set(current_window);
        }
    });

    let is_mobile = compute_is_mobile();
    let placer_getter = create_memo( move |_| {
        GameComponentPlacer::new_from_root_window_size(window_size_getter.get(), is_mobile)
    });
    provide_context( placer_getter);

    let origin = create_memo( move |_| {
        let window_size = window_size_getter.get();
        let placer = placer_getter.get();
        let projected_size = scalar_mult((placer.golden_width, GOLDEN_HEIGHT), placer.scale);
        scalar_mult(point_sub(window_size, projected_size), 0.5)
    });

    let width = move || {
        let placer = placer_getter.get();
        wrap_px(placer.scale(placer.golden_width))
    };

    let height = move || {
        let placer = placer_getter.get();
        wrap_px(placer.scale(GOLDEN_HEIGHT))
    };

    view! { 
        <div
            style:background="#bae8f5"
            style:width="100%"
            style:height="100%"
        >
            <div
                style:position="absolute"
                style:width=width
                style:height=height
                style:left=move || format!("{}px", origin.get().0)
                style:top=move || format!("{}px", origin.get().1)
                style:background="rgb(248, 238, 226)"
                style:transform=move || if placer_getter.get().is_rotated { "rotate(90deg)" } else { "" }
                style:font-size=move || wrap_px(placer_getter.get().scale(DEFAULT_FONT_SIZE))
            >
                {children()}
            </div>
        </div>
    }
}
