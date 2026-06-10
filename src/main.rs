mod hello_world;
use gpui::AppContext;

use gpui::{
    App, Bounds, WindowBounds, WindowOptions,
    px, size,
};
use gpui_platform::application;

use crate::hello_world::HelloWorld;

fn run_example() {
    application().run(|cx: &mut App| {
        let bounds =
            Bounds::centered(None, size(px(500.), px(500.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
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