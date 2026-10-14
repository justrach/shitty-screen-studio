# Recorder

A native macOS screen recorder and editor, built on [Shitty Screen Studio](https://github.com/hewliyang/shitty-screen-studio) by [Li Yang (@hewliyang)](https://github.com/hewliyang). Recorder uses a vendored subset of [Harness](https://github.com/justrach/harness)'s frosted Harness Dark theme, translucent surface components, and icons, and embeds [Codegraff](https://github.com/justrach/codegraff) for editing assistance.

Requires macOS 13 or later on Apple silicon, Rust 1.98.1 (pinned), and Xcode command line tools.

## Build and run

```sh
./script/build_and_run.sh
```

The Codex **Run** action runs the same command. It builds and opens `~/Applications/Recorder.app`. Set `DEST` for another destination. `--verify` checks that the process starts.

### Signing, as in Harness

Recorder discovers the first valid **Apple Development** identity from Keychain, just like Harness's `scripts/run-macos-dev.sh`. The local build uses the fixed ID `recorder.codegraff.app.dev`; release bundles use `recorder.codegraff.app` so development and release permissions stay independent.

For a specific certificate, use:

```sh
export RECORDER_DEV_CODESIGN_IDENTITY='Apple Development: Your Name (TEAMID)'
./script/build_and_run.sh
```

`HARNESS_DEV_CODESIGN_IDENTITY` and `SIGN_IDENTITY` are also accepted. An explicit invalid identity fails signing. Without an identity, the script warns and signs ad hoc; macOS can then require fresh permission after a rebuild. Once your Apple Development certificate is installed, the next build picks it up automatically.

The script bundles the installed `graff` executable, signs it and Recorder with the same identity, verifies the bundle, and refreshes Launch Services. Override the graff path with `GRAFF_BINARY`. Always launch the app bundle so Screen Recording belongs to Recorder rather than the terminal.

## Recording permissions

Click **Allow Screen Access…** to request Screen Recording access. In System Settings → Privacy & Security → Screen & System Audio Recording, enable **Recorder**, then return and click **Check Access Again**. Follow any macOS request to quit and reopen it. If Recorder is absent from the list, add `~/Applications/Recorder.app` with the **+** button.

To request access explicitly on launch:

```sh
open -n "$HOME/Applications/Recorder.app" --args --request-screen-access
```

For a stale development grant, quit Recorder, reset only its screen permission, then request it again:

```sh
tccutil reset ScreenCapture recorder.codegraff.app.dev
open -n "$HOME/Applications/Recorder.app" --args --request-screen-access
```

Microphone, Camera, and Keyboard Shortcuts are optional and off by default. Checks run when you click **Start Recording**, before the window hides. Declined-access messages explain how to enable access or record without that input. Keyboard Shortcuts uses Input Monitoring and may require restarting.

Existing recordings remain in the original `Movies/Shitty Screen Studio` library so renaming the app preserves them. The recording library and demo open without requesting screen access.

## Edit with Codegraff

Open a recording and click **Codegraff** in the editor toolbar. Describe an edit or select a starter prompt, then **Send**. The embedded `graff acp` processes explicitly select Gemini 3.8 Flash (`codegraff/gemini-3.8-flash`) for visual understanding and MiMo 2.6 Pro (`codegraff/mimo-v2.6-pro`) for edits, using your existing graff credentials. If needed, complete `graff login` or sign in through Harness first.

Supported edits include trimming and splitting by time, changing clip speed, adjusting zoom segments, changing backgrounds and aspect ratio, and styling the cursor or camera overlay. Review **Proposed changes**, then click **Apply edits** to update the timeline and preview. **Undo assistant edit** restores the previous timeline and style; making a subsequent timeline edit clears that separate undo control.

With **Video understanding: On**, Recorder sends up to 12 evenly spaced, timestamped JPEG frames (maximum 960×540) to Gemini through Codegraff. This is sampled visual understanding, not full-video or audio analysis. MiMo receives the request, current edit settings, and Gemini’s observations. With understanding off, MiMo receives only the request and edit settings. Recorder does not send audio, recorded keys, or cursor samples. The agent produces a validated JSON proposal with local tools disabled; it cannot directly write recording files through this integration. Cancel stops its process. If you change the timeline while a proposal is being prepared, Recorder asks you to send again against the latest edit.

[Diffusion Studio](https://github.com/diffusionstudio/editor) informed the workflow of reviewing agent edits in the same editor as manual edits. Recorder currently supports the timeline and styling operations above. It does not yet offer transcription, subtitles, or generative assets.

## Credits and vendored material

- Original recorder and GPU editing engine: Li Yang (@hewliyang).
- Harness theme and UI patterns: Standard Harness Pte. Ltd., Rach Pradhan, and Yu Xi Lim. Provenance and license: `vendor/harness/`.
- Solar Icons by 480 Design, [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
- Codegraff: Rach Pradhan and Yu Xi Lim. Bundled license: `vendor/codegraff/LICENSE`.

## Validation

```sh
cargo test --release
bash -n scripts/bundle.sh script/build_and_run.sh
```

The tests include proposal bounds, invalid and overlapping clips, Unicode composer offsets, ACP handshake/streaming, denial of tool permissions, and cancellation without provider calls.

## Release

The tag workflow builds a release-ID bundle and DMG. To create a release bundle locally:

```sh
RECORDER_BUILD_FLAVOR=release DEST="$PWD/dist" scripts/bundle.sh
```

Cursor effects: the Motion & Cursor inspector controls shutter-based motion blur, spring smoothing, click ripples, cursor size, visibility, and black/white/amber artwork. The same Metal compositor applies these settings in preview and export; older recordings retain the default black cursor.

Model routing: Recorder explicitly selects the Codegraff gateway, using the same Codegraff login as Harness. Use `graff route gemini-3.8-flash mimo-v2.6-pro` to inspect routing without calling a model. A missing/expired Codegraff login must be fixed in graff or Harness; Recorder never silently falls back to ChatGPT or another model.

Recorder’s Agents page adapts the Harness account-card layout and uses graff’s existing shared login stores. Sign in opens graff’s browser device flow; no separate Recorder credentials are created. Video and edit model routes are pinned to `codegraff/gemini-3.8-flash` and `codegraff/mimo-v2.6-pro`. Recorder prefers Harness’s managed `~/.harness/bin/graff` when installed.

### Event self recorder

Choose **Event · Self recorder** on the capture screen to record a full-frame webcam take for talks, interviews, and personal updates. Select a named **Webcam** and **Microphone device**, or use **System default**. Microphone starts enabled in Event mode and can be turned off for a silent take. Use **Refresh cameras & microphones** after connecting an external device; disconnected selections fail clearly instead of switching devices silently.

Event mode requires camera access and microphone access when enabled; it does not require Screen Recording or Input Monitoring. Stop with the floating bar, menu bar, or the usual stop shortcut. The take opens in the editor and is saved in the same library, with a clean edge-to-edge canvas. Screen mode also supports the same device selectors for its webcam overlay and microphone.

CLI: `recorder --devices` lists device IDs; `recorder --record 10 --event --mic --camera-device <id> --mic-device <id>` records an event take. Omit the device flags to use system defaults.
