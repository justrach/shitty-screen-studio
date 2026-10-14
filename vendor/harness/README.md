# Harness UI subset

Vendored from https://github.com/justrach/harness at `6984c9715808563982aef241dcf5cf76a833c107`.

- `layered.rs`: the scene-layer wrapper copied from `crates/ui/src/frost.rs` to keep translucent cards and their contents in a stable paint order.
- `icons/`: selected embedded assets from `crates/ui/assets/icons`.
- `theme_tokens.rs`: Harness Dark frosted seeds from `crates/theme/src/builtins.rs`, adapted to Recorder's GPUI controls.
- Layout follows Harness's 44px header, 10px panels, 6px controls, and 4/8/12/16px spacing.

Harness copyright: Standard Harness Pte. Ltd., Rach Pradhan, and Yu Xi Lim. The original Harness license is included in LICENSE. Solar Icons by 480 Design are CC BY 4.0: https://creativecommons.org/licenses/by/4.0/. The graff mark is from Codegraff. Local adaptations are in src/theme.rs and src/icons.rs.

`surface_chrome.rs` adapts the reusable toolbar and field components from `crates/ui/src/surface_chrome.rs`, the native dark-appearance synchronization from `crates/ui/src/appearance.rs`, plus the macOS 0.80 shell / 0.40 card translucency recipes in `crates/ui/src/theme.rs`. Recorder uses its existing native GPUI blurred window as the material behind these surfaces. Recorder vendors Harness’s full `vendor/zui` GPUI fork, including its macOS UnderWindowBackground material fix, Porter-Duff alpha compositing, and scene backdrop blur. `frost.rs` copies Harness’s frosted-card wrapper with its theme check adapted to Recorder’s pinned frosted appearance.

Recorder’s Agents account cards are adapted from `crates/ui/src/settings/accounts.rs`; shared provider credential discovery follows `crates/engine/src/graff_logins.rs` and `codegraff_auth.rs`. Recorder delegates browser authentication and credential storage to graff.
