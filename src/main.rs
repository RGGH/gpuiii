mod hello_world;

use gpui::{App, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;
use gpui::AppContext;
use hello_world::HelloWorld;

fn run_example() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| HelloWorld {
                    text: "".into(),
                    focus_handle: cx.focus_handle(),
                    selected_range: 0..0,
                    marked_range: None,
                    last_layout: None,
                    last_bounds: None,
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}

fn main() {
    run_example();
}