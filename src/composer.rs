//! Native text input for the editing assistant, including macOS composition.
use crate::theme::*;
use gpui::{
    Bounds, ClipboardItem, Context, ElementInputHandler, EntityInputHandler, EventEmitter,
    FocusHandle, IntoElement, KeyDownEvent, Pixels, Point, Render, Styled, UTF16Selection,
    Window, canvas, div, px, rgb, prelude::*,
};
use std::ops::Range;

pub struct Submit;
pub struct Composer {
    pub focus: FocusHandle,
    text: String,
    selected: Range<usize>,
    marked: Option<Range<usize>>,
}
impl EventEmitter<Submit> for Composer {}

fn byte_offset(text: &str, utf16: usize) -> usize {
    let mut n = 0;
    for (i, c) in text.char_indices() {
        if n >= utf16 {
            return i;
        }
        n += c.len_utf16();
    }
    text.len()
}
fn utf16_offset(text: &str, bytes: usize) -> usize {
    text[..bytes].encode_utf16().count()
}

impl Composer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            text: String::new(),
            selected: 0..0,
            marked: None,
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set(&mut self, text: String, cx: &mut Context<Self>) {
        self.text = text;
        self.selected = self.text.len()..self.text.len();
        self.marked = None;
        cx.notify();
    }
    fn range(&self, r: &Range<usize>) -> Range<usize> {
        byte_offset(&self.text, r.start)..byte_offset(&self.text, r.end)
    }
    fn utf16(&self, r: &Range<usize>) -> Range<usize> {
        utf16_offset(&self.text, r.start)..utf16_offset(&self.text, r.end)
    }
    fn replace(&mut self, r: Range<usize>, text: &str) {
        self.text.replace_range(r.clone(), text);
        self.selected = r.start + text.len()..r.start + text.len();
        self.marked = None;
    }
    fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let cmd = ev.keystroke.modifiers.platform;
        match ev.keystroke.key.as_str() {
            "enter" => {
                cx.emit(Submit);
                cx.stop_propagation();
                return;
            }
            "a" if cmd => self.selected = 0..self.text.len(),
            "c" if cmd => cx.write_to_clipboard(ClipboardItem::new_string(
                self.text[self.selected.clone()].into(),
            )),
            "x" if cmd => {
                cx.write_to_clipboard(ClipboardItem::new_string(
                    self.text[self.selected.clone()].into(),
                ));
                self.replace(self.selected.clone(), "");
            }
            "v" if cmd => {
                if let Some(text) = cx.read_from_clipboard().and_then(|i| i.text()) {
                    self.replace(self.selected.clone(), &text);
                }
            }
            "backspace" => {
                let mut r = self.selected.clone();
                if r.is_empty() {
                    r.start = self.text[..r.start]
                        .char_indices()
                        .last()
                        .map_or(0, |(i, _)| i);
                }
                self.replace(r, "");
            }
            "delete" => {
                let mut r = self.selected.clone();
                if r.is_empty() {
                    r.end = self.text[r.end..]
                        .chars()
                        .next()
                        .map_or(r.end, |c| r.end + c.len_utf8());
                }
                self.replace(r, "");
            }
            "left" => {
                let i = self.text[..self.selected.start]
                    .char_indices()
                    .last()
                    .map_or(0, |(i, _)| i);
                self.selected = i..i;
            }
            "right" => {
                let i = self.text[self.selected.end..]
                    .chars()
                    .next()
                    .map_or(self.selected.end, |c| self.selected.end + c.len_utf8());
                self.selected = i..i;
            }
            "escape" => {
                window.blur();
            }
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}

impl EntityInputHandler for Composer {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.range(&r);
        *actual = Some(self.utf16(&r));
        Some(self.text[r].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.utf16(&self.selected),
            reversed: false,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|r| self.utf16(r))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = r
            .map(|r| self.range(&r))
            .or(self.marked.clone())
            .unwrap_or(self.selected.clone());
        self.replace(r, text);
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        sel: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = r
            .map(|r| self.range(&r))
            .or(self.marked.clone())
            .unwrap_or(self.selected.clone());
        let start = r.start;
        self.replace(r, text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        if let Some(sel) = sel {
            self.selected =
                start + byte_offset(text, sel.start)..start + byte_offset(text, sel.end);
        }
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        _: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(bounds)
    }
    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(utf16_offset(&self.text, self.selected.end))
    }
}

impl Render for Composer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = cx.entity();
        let focus = self.focus.clone();
        let focused = focus.is_focused(window);
        let display = if self.text.is_empty() && !focused {
            "Describe an edit…".into()
        } else {
            let mut text = self.text.clone();
            if focused && self.selected.is_empty() {
                text.insert_str(self.selected.end, "│");
            }
            text
        };
        div()
            .id("agent-composer")
            .track_focus(&self.focus)
            .key_context("AgentComposer")
            .on_key_down(cx.listener(Self::key))
            .on_click(cx.listener(|this, _, window, cx| {
                window.focus(&this.focus, cx);
                cx.notify();
            }))
            .relative()
            .w_full()
            .min_h(px(72.))
            .p_3()
            .rounded(px(10.))
            .bg(glass_inset())
            .border_1()
            .border_color(if focused {
                alpha(ACCENT, 0.6)
            } else {
                separator()
            })
            .cursor_text()
            .text_size(px(BODY))
            .text_color(rgb(if self.text.is_empty() { TERTIARY } else { TEXT }))
            .child(display)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        window.handle_input(
                            &focus,
                            ElementInputHandler::new(bounds, input.clone()),
                            cx,
                        )
                    },
                )
                .absolute()
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_unicode_input_offsets() {
        let text = "a🙂界";
        assert_eq!(byte_offset(text, 1), 1);
        assert_eq!(byte_offset(text, 3), 5);
        assert_eq!(utf16_offset(text, 5), 3);
    }
}
