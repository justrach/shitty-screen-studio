//! Small embedded subset of Harness's SVG assets; provenance in vendor/harness.
use gpui::{AssetSource, SharedString, Styled as _, Svg, svg};
use std::borrow::Cow;

macro_rules! assets {
    ($($name:ident => $file:literal),* $(,)?) => {
        $(pub const $name: &str = concat!("icons/", $file, ".svg");)*
        pub struct Assets;
        impl AssetSource for Assets {
            fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
                Ok(match path {
                    $(concat!("icons/", $file, ".svg") => Some(Cow::Borrowed(include_bytes!(concat!("../vendor/harness/icons/", $file, ".svg")).as_slice())),)*
                    _ => None,
                })
            }
            fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
                Ok([$($name),*].into_iter().filter(|p| p.starts_with(path)).map(SharedString::from).collect())
            }
        }
    };
}
assets! {
    MONITOR => "monitor", KEYBOARD => "keyboard", GRAFF => "graff-mark",
    FOLDER => "folder", NEW => "pen-new-square", BACK => "arrow-left",
    SEND => "arrow-up", CLOSE => "close", STOP => "stop", CLOCK => "clock-circle",
    SETTINGS => "settings-minimalistic", MAGIC => "magic-stick-3", AUDIO => "volume-loud",
    WINDOW => "window-restore", CHECK => "check", PLUS => "plus",
    PHONE => "smartphone", TUNING => "tuning", EYE => "eye", REFRESH => "refresh",
}
pub fn icon(path: &'static str) -> Svg {
    svg().path(path).flex_none()
}
