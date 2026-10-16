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

Microphone, Camera, and Keyboard Shortcuts are optional and off by default. **Check inputs** requests camera/microphone access for a live preflight check; **Start Recording** also checks enabled inputs before the window hides. Declined-access messages explain how to enable access or record without that input. Keyboard Shortcuts uses Input Monitoring and may require restarting.

Existing recordings remain in the original `Movies/Shitty Screen Studio` library so renaming the app preserves them. The recording library and demo open without requesting screen access.

## Edit with Codegraff

Open a recording and click **Codegraff** in the editor toolbar. Describe an edit or select a starter prompt, then **Send**. The embedded `graff acp` processes explicitly select Gemini 3.8 Flash (`codegraff/gemini-3.8-flash`) for visual understanding and MiMo 2.6 Pro (`codegraff/mimo-v2.6-pro`) for edits, using your existing graff credentials. If needed, complete `graff login` or sign in through Harness first.

Supported edits include trimming and splitting by time, clip speed, zoom segments, backgrounds and aspect ratio, and cursor/camera styling. Starter prompts target clip pacing, the current zoom/playhead, and presentation polish. Requests include the selected clip/zoom, edited and source playhead times, recording dimensions/fps, track-presence flags, and current edit settings. Send a follow-up to refine a pending, unapplied proposal rather than starting from scratch.

The panel remembers the last **4 approved, successfully saved edits** (bounded request/result excerpts), only in memory for the current editor session. Cancelled, failed, and unapplied proposals do not enter this history. **Clear recent context** clears it when idle; a visible pending proposal and its request remain available for refinement. Current settings remain authoritative after undo or manual edits.

With **Video understanding: On**, Recorder sends **at most 12 targeted, timestamped sampled frames** to Gemini through Codegraff, prioritizing the selected clip and playhead with broader coverage. This is sampled visual understanding, not full-video or audio analysis. MiMo receives visual observations plus your request, current edit context, recent approved history, and any pending proposal being refined. With understanding off, no new frames or audio are sent; MiMo still receives the edit context/history. Context excludes recording paths, raw recorded keypress/cursor data, and login data. Sampled frames can contain anything visible in the recording.

The agent proposes validated JSON with local tools disabled, with **one validation-repair retry only for malformed or invalid output**. Validation checks structure and bounds, not whether a visual interpretation is correct; review scene-based edits carefully. It cannot directly write recording files through this integration. Review **Proposed changes**, then explicitly **Apply edits to timeline**; nothing applies automatically. No-op proposals are surfaced without applying or recording a successful edit. **Undo** / **⌘Z** restores the previous edit. Cancel stops the running request. If the timeline or style changed since a request was sent, applying is blocked until you send again against the latest edit.

[Diffusion Studio](https://github.com/diffusionstudio/editor) informed the workflow of reviewing agent edits in the same editor as manual edits. Recorder currently supports the timeline and styling operations above. It does not yet offer transcription, subtitles, or generative assets.

## Credits and vendored material

- Original recorder and GPU editing engine: Li Yang (@hewliyang).
- Harness theme and UI patterns: Standard Harness Pte. Ltd., Rach Pradhan, and Yu Xi Lim. Provenance and license: `vendor/harness/`.
- Solar Icons by 480 Design, [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
- Codegraff: Rach Pradhan and Yu Xi Lim. Bundled license: `vendor/codegraff/LICENSE`.
- Capture-screen artwork: generated with Codex imagegen; source brief in `assets/studio-light.prompt.txt`.

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

### Live input check

The **Before you roll** panel previews the selected webcam and meters the selected microphone in either capture mode. Enable the inputs you want, then click **Check inputs**. Monitoring is explicit: opening Recorder does not activate devices or ask for camera/microphone access.

- Live webcam preview with actual dimensions, measured preview frame rate, optional rule-of-thirds guides, and preview-only mirroring. Guides and mirroring do not alter recordings.
- Native AVFoundation audio metering on a −60 to 0 dBFS scale: average level, white peak-hold marker, maximum peak with reset, and a two-second near-clipping warning at −1 dBFS. The meter shows the loudest channel; it measures digital signal level, not room loudness. Adjust the microphone's input level in macOS or on the device.
- Changing enabled inputs or selected devices restarts the check. Missing/stale signals and permission failures show actionable messages; disconnected selections never silently fall back.
- **Stop input check**, opening the editor/Agents, and starting recording release the monitoring session. Recording waits for hardware release before opening its own session.

Preview uses Apple's AVFoundation capture APIs and CoreVideo/Metal surfaces. The input check creates no files, sends no media, and does not play microphone audio through speakers.

CLI: `recorder --devices` lists device IDs; `recorder --record 10 --event --mic --camera-device <id> --mic-device <id>` records an event take. Omit the device flags to use system defaults.
