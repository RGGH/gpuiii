use std::ops::Range;

use gpui::{
    App, Bounds, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, IntoElement,
    LayoutId, MouseButton, MouseDownEvent, PaintQuad, ParentElement, Pixels,
    Render, ShapedLine, SharedString, Style, Styled, TextRun,
    UTF16Selection, UnderlineStyle, Window, black, div, fill, hsla, point,
    prelude::*, px, relative, rgb, rgba, size, white,
};

// ── The model ────────────────────────────────────────────────────────────────

pub struct HelloWorld {
    pub text: SharedString,
    pub focus_handle: FocusHandle,
    pub selected_range: Range<usize>,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<ShapedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
}

impl Focusable for HelloWorld {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// UTF-8 ↔ UTF-16 helpers (GPUI's IME layer speaks UTF-16)
impl HelloWorld {
    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for ch in self.text.chars() {
            if utf16 >= offset { break; }
            utf16 += ch.len_utf16();
            utf8  += ch.len_utf8();
        }
        utf8
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16 = 0;
        let mut utf8  = 0;
        for ch in self.text.chars() {
            if utf8 >= offset { break; }
            utf8  += ch.len_utf8();
            utf16 += ch.len_utf16();
        }
        utf16
    }

    fn range_to_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(r.start)..self.offset_to_utf16(r.end)
    }

    fn range_from_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(r.start)..self.offset_from_utf16(r.end)
    }
}

// ── EntityInputHandler ────────────────────────────────────────────────────────
// This is what actually receives typed characters, IME compositions, etc.

impl EntityInputHandler for HelloWorld {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|r| self.range_to_utf16(r))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        self.text = format!(
            "{}{}{}",
            &self.text[..range.start],
            new_text,
            &self.text[range.end..]
        ).into();

        let cursor = range.start + new_text.len();
        self.selected_range = cursor..cursor;
        self.marked_range = None;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        self.text = format!(
            "{}{}{}",
            &self.text[..range.start],
            new_text,
            &self.text[range.end..]
        ).into();

        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(range.start..range.start + new_text.len())
        };

        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|r| r.start + range.start..r.end + range.start)
            .unwrap_or_else(|| {
                let c = range.start + new_text.len();
                c..c
            });

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(bounds.left() + layout.x_for_index(range.start), bounds.top()),
            point(bounds.left() + layout.x_for_index(range.end),   bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        pt: gpui::Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let line_pt = self.last_bounds?.localize(&pt)?;
        let layout  = self.last_layout.as_ref()?;
        let utf8    = layout.index_for_x(pt.x - line_pt.x)?;
        Some(self.offset_to_utf16(utf8))
    }
}

// ── The custom Element ────────────────────────────────────────────────────────
// Responsible for layout, prepaint (cursor/selection quads), and painting.

struct TextElement {
     input: Entity<HelloWorld>,
}

struct PrepaintState {
    line:      Option<ShapedLine>,
    cursor:    Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;
    fn into_element(self) -> Self { self }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> { None }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> { None }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width  = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PrepaintState {
        let input          = self.input.read(cx);
        let content        = input.text.clone();
        let selected_range = input.selected_range.clone();
        let cursor_pos_idx = input.selected_range.end;   // simple: cursor at end of selection
        let style          = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            ("Type here…".into(), hsla(0., 0., 0., 0.35))
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if let Some(marked) = input.marked_range.as_ref() {
            vec![
                TextRun { len: marked.start,                            ..run.clone() },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun { len: display_text.len() - marked.end,        ..run },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_x = line.x_for_index(cursor_pos_idx);

        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_x, bounds.top()),
                        size(px(2.), bounds.size.height),
                    ),
                    gpui::blue(),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(bounds.left() + line.x_for_index(selected_range.start), bounds.top()),
                        point(bounds.left() + line.x_for_index(selected_range.end),   bounds.bottom()),
                    ),
                    rgba(0x3311ff30),
                )),
                None,
            )
        };

        PrepaintState { line: Some(line), cursor, selection }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        prepaint: &mut PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();

        // ← This is the crucial call: registers our entity as the IME/keyboard handler
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        if let Some(sel) = prepaint.selection.take() {
            window.paint_quad(sel);
        }

        let line = prepaint.line.take().unwrap();
        line.paint(bounds.origin, window.line_height(), gpui::TextAlign::Left, None, window, cx)
            .unwrap();

        if focus_handle.is_focused(window) {
            if let Some(cur) = prepaint.cursor.take() {
                window.paint_quad(cur);
            }
        }

        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds  = Some(bounds);
        });
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

impl Render for HelloWorld {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus_handle(cx), cx);
                }),
            )
            .size_full() 
            .bg(rgb(0x333333))
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap_3()
            .text_color(rgb(0xffffff))
            .text_size(px(20.))
            .child(format!("Hello, {}", self.text))
            // ↓ The actual input box — rendered by our custom Element
            .child(
                div()
                    .w(px(320.))
                    .h(px(38.))
                    .bg(white())
                    .text_color(black())
                    .px(px(6.))
                    .py(px(4.))
                    .child(TextElement { input: cx.entity() }),
            )
    }
}