use gpui::{
    Context, IntoElement, Render, SharedString, Window,
    div, prelude::*, rgb,
};

pub struct HelloWorld {
    pub text: SharedString,
}

impl Render for HelloWorld {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(gpui::px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::red())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::green())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::blue())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
            )
    }
}