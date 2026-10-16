//! Capture-room preflight UI. Monitoring never writes or plays back media.
use super::*;
use crate::icons::{self, icon};
use crate::monitor::meter_fraction;
use gpui::{ObjectFit, relative, surface};

impl Studio {
    pub(super) fn render_preflight(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let camera = self.tab == Tab::Event || self.options.camera;
        let mic = self.options.mic;
        let (frame, dimensions, average, peak, max_peak, video_live, audio_live, clipping, waiting, error, fps) =
            if self.monitoring {
                let state = self.monitor.as_ref().unwrap().shared.lock().unwrap();
                (state.frame.clone(), state.dimensions, state.average, state.peak, state.max_peak,
                    state.video_live(), state.audio_live(), state.clipping(), state.waiting_too_long(camera, mic), state.error.clone(), state.fps())
            } else {
                (None, (0, 0), -60., -60., -60., false, false, false, false, None, 0.)
            };
        let live = self.monitoring && error.is_none() && (!camera || video_live) && (!mic || audio_live);
        let status = if error.is_some() { "Check devices" } else if live { "Inputs live" } else if waiting { "No signal" } else if self.monitoring { "Starting inputs…" } else { "Not monitoring" };
        let camera_name = self.options.camera_device.as_ref()
            .and_then(|id| self.cameras.iter().find(|device| &device.id == id))
            .map(|d| d.name.as_str()).unwrap_or("System default camera");
        let mic_name = self.options.mic_device.as_ref()
            .and_then(|id| self.microphones.iter().find(|device| &device.id == id))
            .map(|d| d.name.as_str()).unwrap_or("System default microphone");
        let picture: AnyElement = match frame.filter(|_| camera && video_live) {
            Some(frame) => surface(frame.to_gpui()).size_full().object_fit(ObjectFit::Contain).into_any_element(),
            None => div().size_full().flex().flex_col().items_center().justify_center().gap_2()
                .child(div().size(px(44.)).rounded(px(14.)).border_1().border_color(alpha(ACCENT, 0.2)).bg(alpha(ACCENT, 0.07))
                    .flex().items_center().justify_center()
                    .child(icon(icons::EYE).size(px(22.)).text_color(rgb(ACCENT))))
                .child(small(if !camera { "Camera is off" } else if !self.monitoring { "Check your framing before the take" } else if waiting { "No camera frames — check connection" } else { "Waiting for camera…" }).text_size(px(11.)))
                .into_any_element(),
        };
        let aspect = if dimensions.1 > 0 { dimensions.0 as f32 / dimensions.1 as f32 } else { 16. / 9. };
        let visible_average = if audio_live { average } else { -60. };
        let visible_peak = if audio_live { peak } else { -60. };
        let meter_color = if clipping { RED } else if visible_peak >= -6. { ACCENT } else { SUCCESS };
        let sound_note = if !mic { "Microphone is off" } else if !self.monitoring { "Say a few words during the input check" } else if !audio_live { "Waiting for microphone signal…" } else if clipping { "Near clipping — lower your input level" } else if average <= -45. { "Quiet input — speak at your recording volume" } else { "Signal detected · aim for peaks around −12 to −6 dBFS" };
        glass_card().w_full().p_4().flex().flex_col().gap_3()
            .child(small("02  /  CHECK YOUR INPUTS").text_size(px(10.)).font_weight(FontWeight::SEMIBOLD).text_color(rgb(ACCENT)))
            .child(div().flex().items_center().justify_between().gap_2()
                .child(div().text_size(px(18.)).font_weight(FontWeight::SEMIBOLD).child("Before you roll"))
                .child(div().flex_none().flex().items_center().gap_1().px_2().py_1().rounded_full().bg(alpha(if live { SUCCESS } else { MUTED }, 0.08))
                    .child(div().size(px(5.)).rounded_full().bg(rgb(if live { SUCCESS } else { MUTED })))
                    .child(small(status).text_size(px(9.)))))
            .child(small("A private preview. A confident first take."))
            .child(div().relative().w_full().aspect_ratio(aspect).rounded(px(10.)).overflow_hidden().border_1().border_color(alpha(0xffffff, 0.07)).bg(rgb(0x08090c))
                .child(picture)
                .when(camera && video_live && self.framing_guides, |d| d
                    .children([1. / 3., 2. / 3.].map(|at| div().absolute().left(relative(at)).top_0().bottom_0().w(px(1.)).bg(alpha(0xffffff, 0.28))))
                    .children([1. / 3., 2. / 3.].map(|at| div().absolute().top(relative(at)).left_0().right_0().h(px(1.)).bg(alpha(0xffffff, 0.28)))))
                .when(video_live, |d| d.child(div().absolute().top_2().left_2().px_2().py_1().rounded(px(4.)).bg(alpha(0x000000, 0.65))
                    .child(small("LIVE · NOT RECORDING").text_color(rgb(0xffffff)).text_size(px(9.)))))
                .when(video_live && self.mirror_preview, |d| d.child(div().absolute().bottom_2().right_2().px_2().py_1().rounded(px(4.)).bg(alpha(0x000000, 0.65))
                    .child(small("MIRRORED PREVIEW").text_color(rgb(0xffffff)).text_size(px(9.))))))
            .when(camera, |d| d.child(div().flex().flex_col().gap_1()
                .child(small(camera_name.to_string()).text_color(rgb(TEXT)))
                .child(small(if video_live { format!("{} × {} · {:.0} fps · preview only", dimensions.0, dimensions.1, fps) } else { "Selected webcam · no media saved".into() }).font_features(tabular()).text_size(px(10.)))))
            .when(camera, |d| d.child(div().flex().flex_col().gap_2()
                .child(group_row(true).id("framing-guides").cursor_default()
                    .child(small("Rule-of-thirds guides").text_color(rgb(TEXT))).child(switch(self.framing_guides))
                    .on_click(cx.listener(|this, _, _, cx| { this.framing_guides = !this.framing_guides; cx.notify(); })))
                .child(group_row(false).id("mirror-preview").cursor_default()
                    .child(small("Mirror preview only").text_color(rgb(TEXT))).child(switch(self.mirror_preview))
                    .on_click(cx.listener(|this, _, _, cx| { this.mirror_preview = !this.mirror_preview; this.refresh_monitoring(cx); })))
                .child(small("Guides and mirroring never change the recorded video.").text_size(px(10.)))))
            .child(div().pt_2().border_t_1().border_color(separator()).flex().flex_col().gap_2()
                .child(div().flex().justify_between().items_center()
                    .child(group_title("Microphone · dBFS"))
                    .child(small(if !mic || !audio_live { "—".to_string() } else if average <= -60. { "≤ −60 dBFS".into() } else { format!("{average:.0} dBFS") }).font_features(tabular()).text_color(rgb(meter_color))))
                .child(div().relative().h(px(18.)).w_full().flex().gap(px(2.))
                    .children((0..30).map(|i| {
                        let db = -60. + (i + 1) as f32 * 2.;
                        let color = if db > -6. { RED } else if db > -18. { ACCENT } else { SUCCESS };
                        div().flex_1().h_full().rounded(px(2.)).bg(alpha(color, if meter_fraction(visible_average) * 30. >= (i + 1) as f32 { 0.95 } else { 0.12 }))
                    }))
                    .when(audio_live && visible_peak > -60., |d| d.child(div().absolute().left(relative(meter_fraction(visible_peak).min(0.99))).top_0().bottom_0().w(px(2.)).bg(rgb(0xffffff)))))
                .child(div().flex().justify_between().children(["−60", "−36", "−18", "−6", "0"].map(|label| small(label).text_size(px(9.)).font_features(tabular()))))
                .child(div().flex().justify_between().items_center().gap_2()
                    .child(small(if max_peak > -60. { format!("Max peak {max_peak:.0} dBFS") } else { "Max peak —".into() }).text_size(px(10.)).font_features(tabular()))
                    .child(button("reset-audio-peak", "Reset peak").on_click(cx.listener(|this, _, _, cx| {
                        if let Some(monitor) = &this.monitor { monitor.reset_peak(); }
                        cx.notify();
                    }))))
                .child(small(sound_note).text_color(rgb(if clipping { RED } else { MUTED })))
                .when(mic, |d| d.child(small(mic_name.to_string()).text_size(px(10.)))))
            .when_some(error.clone(), |d, error| d.child(div().p_3().rounded(px(8.)).bg(alpha(RED, 0.1)).text_size(px(11.)).text_color(rgb(RED)).flex().flex_col().gap_2()
                .child(error)
                .child(button("input-privacy-settings", "Open Privacy Settings")
                    .on_click(cx.listener(|this, _, _, cx| this.open_privacy_settings(cx))))))
            .when(waiting && error.is_none(), |d| d.child(small("An input stopped sending data. Check its connection, refresh devices, or retry the check.").text_color(rgb(RED))))
            .child(div().flex().gap_2()
                .child(primary_button("check-inputs", if self.monitoring { "Stop input check" } else { "Check inputs" }).flex_1()
                    .when(!camera && !mic, |d| d.opacity(0.4))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if camera || mic { this.toggle_monitoring(cx); }
                    })))
                .when(self.monitoring && (error.is_some() || waiting), |d| d.child(button("retry-input-check", "Retry")
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_monitoring(cx))))))
            .when(!camera && !mic, |d| d.child(small("Enable Camera or Microphone in Inputs to check them.")))
            .child(small("Uses Apple AVFoundation. Nothing is saved or played through your speakers. Devices are released before recording or leaving capture.").text_size(px(10.)))
            .into_any_element()
    }
}
