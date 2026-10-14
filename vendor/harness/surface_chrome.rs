//! Adapted from Harness crates/ui/src/surface_chrome.rs and theme.rs.
//! Shares Harness's frosted shell, field, and toolbar recipes with Recorder.
use crate::theme::{BG, CHROME, alpha, separator};

/// Adapted from Harness appearance::sync_ns_appearance. Match the native
/// vibrancy material to the pinned dark palette instead of the OS light mode.
pub fn sync_native_appearance() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSAppearance, NSAppearanceNameDarkAqua, NSApplication};
    if let Some(main) = MainThreadMarker::new() {
        let appearance = NSAppearance::appearanceNamed(unsafe { NSAppearanceNameDarkAqua });
        NSApplication::sharedApplication(main).setAppearance(appearance.as_deref());
    }
}
use gpui::{Div, Hsla, div, prelude::*, px};

pub const HEADER_HEIGHT: f32 = 44.0;
pub const CONTROL_SIZE: f32 = 30.0;
pub const CONTROL_RADIUS: f32 = 8.0;
pub const EDGE_INSET: f32 = 16.0;

/// Harness's macOS shell coverage, over GPUI's native blurred window.
pub fn shell() -> Hsla {
    alpha(CHROME, 0.98)
}

/// Harness card_glass_bg: retain the window material through inset cards.
pub fn card() -> Div {
    div()
        .rounded(px(16.))
        .bg(alpha(0x202024, 0.98))
        .border_1()
        .border_color(alpha(0xffffff, 0.10))
}

/// Harness surface field ink; fields remain distinct without an opaque slab.
pub fn input() -> Div {
    div()
        .h(px(CONTROL_SIZE))
        .min_w_0()
        .flex_1()
        .px(px(8.))
        .rounded(px(CONTROL_RADIUS))
        .bg(alpha(0xffffff, 0.035))
        .flex()
        .items_center()
        .gap(px(6.))
        .text_size(px(11.5))
}

pub fn toolbar() -> Div {
    div()
        .h(px(HEADER_HEIGHT))
        .w_full()
        .flex_none()
        .px(px(EDGE_INSET))
        .flex()
        .items_center()
        .gap(px(4.))
        .border_b_1()
        .border_color(separator())
        .bg(alpha(CHROME, 0.98))
}

pub fn inset() -> Hsla {
    alpha(BG, 0.72)
}
