use super::*;
use crate::{
    agent::{self, Event, Proposal},
    composer::Composer,
    icons::{self, icon},
};
use gpui::Entity;

pub(super) struct Assistant {
    pub visible: bool,
    pub understand_video: bool,
    pub input: Entity<Composer>,
    pub run: Option<agent::Run>,
    pub status: String,
    pub proposal: Option<Proposal>,
    pub baseline: Option<(Vec<ZoomSegment>, Timeline, Style)>,
}

fn style_changes(before: Style, after: Style) -> Vec<String> {
    let mut changes = Vec::new();
    let on_off = |v: bool| if v { "On" } else { "Off" };
    let background = |i| match crate::style::fill(i) {
        crate::style::Fill::Gradient(bg) => bg.name,
        crate::style::Fill::Wallpaper(i) => crate::style::WALLPAPERS[i],
    };
    let corner = |i: u8| ["Top left", "Top right", "Bottom left", "Bottom right"]
        .get(i as usize).copied().unwrap_or("Unknown");
    let keys_position = |i: u8| ["Top left", "Top", "Top right", "Bottom left", "Bottom", "Bottom right"]
        .get(i as usize).copied().unwrap_or("Unknown");
    macro_rules! change {
        ($field:ident, $label:literal, $format:expr) => {
            if before.$field != after.$field {
                let format = $format;
                changes.push(format!("{}: {} → {}", $label, format(before.$field), format(after.$field)));
            }
        };
    }
    change!(background, "Background", background);
    change!(aspect, "Canvas", |v: crate::style::Aspect| v.label());
    change!(padding, "Padding", |v: f32| format!("{v:.0}px"));
    change!(radius, "Corners", |v: f32| format!("{v:.0}px"));
    change!(shadow, "Shadow", |v: f32| format!("{:.0}%", v * 100.));
    change!(zoom, "Default zoom", |v: f32| format!("{v:.1}×"));
    change!(auto_zoom, "Automatic zoom", on_off);
    change!(cursor_style, "Cursor style", |v: crate::style::CursorStyle| v.label());
    change!(cursor_visible, "Show cursor", on_off);
    change!(cursor_size, "Cursor size", |v: f32| format!("{v:.1}×"));
    change!(cursor_smoothing, "Cursor smoothing", |v: f32| format!("{:.0}%", v * 100.));
    change!(click_ripple, "Click ripple", on_off);
    change!(motion_blur, "Motion blur", |v: f32| format!("{:.0}%", v * 100.));
    change!(camera_speed, "Camera speed", |v: f32| format!("{:.0}%", v * 100.));
    change!(camera_visible, "Show camera", on_off);
    change!(camera_size, "Camera size", |v: f32| format!("{:.0}%", v * 100.));
    change!(camera_corner, "Camera position", corner);
    change!(camera_circle, "Circular camera", on_off);
    change!(keys_visible, "Show shortcuts", on_off);
    change!(keys_position, "Shortcut position", keys_position);
    changes
}

impl Editor {
    pub(super) fn send_agent(&mut self, cx: &mut Context<Self>) {
        if self.assistant.run.is_some() || self.export.is_some() {
            return;
        }
        let request = self.assistant.input.read(cx).text().trim().to_owned();
        if request.is_empty() {
            return;
        }
        self.persist();
        self.player.pause();
        let prompt = agent::prompt(
            &request,
            self.project.rec.duration,
            &self.timeline,
            &self.zooms,
            self.style,
        );
        self.assistant.proposal = None;
        self.assistant.baseline = Some((self.zooms.clone(), self.timeline.clone(), self.style));
        self.assistant.status = "Connecting to Codegraff…".into();
        self.assistant.run = Some(agent::Run::start(
            self.project.dir.clone(),
            prompt,
            self.project.rec.duration,
            self.assistant
                .understand_video
                .then(|| self.project.video()),
        ));
        cx.notify();
    }

    pub(super) fn poll_agent(&mut self) -> bool {
        let mut changed = false;
        let mut done = None;
        if let Some(run) = &self.assistant.run {
            while let Ok(event) = run.events.try_recv() {
                changed = true;
                match event {
                    Event::Status(s) => self.assistant.status = s,
                    Event::Progress => {}
                    Event::Done(result) => done = Some(result),
                }
            }
        }
        if let Some(result) = done {
            self.assistant.run = None;
            match result {
                Ok(p) => {
                    self.assistant.status = p.summary.clone();
                    self.assistant.proposal = Some(p);
                }
                Err(e) => self.assistant.status = format!("{e:#}"),
            }
        }
        changed
    }

    fn apply_agent(&mut self, cx: &mut Context<Self>) {
        if self.export.is_some() {
            return;
        }
        let Some(p) = self.assistant.proposal.clone() else {
            return;
        };
        if self.assistant.baseline.as_ref()
            != Some(&(self.zooms.clone(), self.timeline.clone(), self.style))
        {
            self.assistant.status="Your edit changed while Codegraff was working. Send the request again to use the latest timeline.".into();
            self.assistant.proposal = None;
            cx.notify();
            return;
        }
        if let Err(e) = p.validate(self.project.rec.duration) {
            self.assistant.status = e.to_string();
            cx.notify();
            return;
        }
        let before = (self.zooms.clone(), self.timeline.clone(), self.style);
        if let Some(t) = p.timeline {
            self.timeline = t;
        }
        if let Some(z) = p.zooms {
            self.zooms = z;
        }
        if let Some(s) = p.style {
            self.style = s;
        }
        self.project.rec.zooms = Some(self.zooms.clone());
        self.project.rec.timeline = Some(self.timeline.clone());
        self.project.rec.style = Some(self.style);
        if let Err(e) = self.project.save() {
            self.zooms = before.0;
            self.timeline = before.1;
            self.style = before.2;
            self.project.rec.zooms = Some(self.zooms.clone());
            self.project.rec.timeline = Some(self.timeline.clone());
            self.project.rec.style = Some(self.style);
            self.assistant.status = format!("Could not save edit: {e}");
        } else {
            self.history.push(before);
            if self.history.len() > 100 {
                self.history.remove(0);
            }
            self.redo_history.clear();
            self.assistant.proposal = None;
            self.selected = None;
            self.selected_slice = None;
            self.refresh();
            self.player.seek(0.);
            self.assistant.status = "Applied to your timeline and preview. Use Undo or ⌘Z to restore the previous edit.".into();
        }
        cx.notify();
    }

    pub(super) fn render_agent(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let busy = self.assistant.run.is_some();
        let mut body = div()
            .id("agent-response")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Make the next edit."),
            )
            .child(small(
                "Gemini 3.8 Flash understands frames. MiMo 2.6 Pro proposes edits. Both use your Codegraff login.",
            ))
            .child(
                button(
                    "understand-video",
                    if self.assistant.understand_video {
                        "Video understanding: On"
                    } else {
                        "Video understanding: Off"
                    },
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    if this.assistant.run.is_none() {
                        this.assistant.understand_video = !this.assistant.understand_video;
                        cx.notify();
                    }
                })),
            );
        if self.assistant.status.is_empty() {
            for (id, label, prompt) in [
                (
                    "agent-trim",
                    "Trim the first second",
                    "Cut the first second from this recording.",
                ),
                (
                    "agent-portrait",
                    "Make it portrait",
                    "Set the canvas to vertical with a Midnight background and 60px padding.",
                ),
                (
                    "agent-zoom",
                    "Give it a cleaner look",
                    "Use Graphite, rounded corners, a subtle shadow, and smooth the cursor.",
                ),
            ] {
                body = body.child(button(id, label).w_full().justify_start().on_click(
                    cx.listener(move |this, _, window, cx| {
                        this.assistant
                            .input
                            .update(cx, |input, cx| input.set(prompt.into(), cx));
                        let focus = this.assistant.input.read(cx).focus.clone();
                        window.focus(&focus, cx);
                        cx.notify();
                    }),
                ));
            }
        } else {
            body = body.child(
                div()
                    .p_3()
                    .rounded(px(10.))
                    .bg(alpha(0xffffff, 0.055))
                    .text_size(px(BODY))
                    .child(self.assistant.status.clone()),
            );
        }
        if let Some(p) = self.assistant.proposal.clone() {
            let mut review = div()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(section_title("Proposed changes"));
            if let Some(t) = p.timeline.filter(|t| *t != self.timeline) {
                review = review.child(small(format!(
                    "Timeline: {} clips · {} → {} clips · {}",
                    self.timeline.slices.len(), fmt_time(self.timeline.duration()),
                    t.slices.len(), fmt_time(t.duration())
                )));
                for (label, timeline) in [("Before", &self.timeline), ("After", &t)] {
                    review = review.child(small(label).text_color(rgb(ACCENT)));
                    for (i, slice) in timeline.slices.iter().enumerate() {
                        review = review.child(small(format!("Clip {}: {}–{} · {}×", i + 1,
                            fmt_time(slice.start), fmt_time(slice.end), slice.speed)));
                    }
                }
            }
            if let Some(z) = p.zooms.filter(|z| *z != self.zooms) {
                review = review.child(small(format!("Zooms: {} → {} segments", self.zooms.len(), z.len())));
                for (label, zooms) in [("Before", &self.zooms), ("After", &z)] {
                    review = review.child(small(label).text_color(rgb(ACCENT)));
                    if zooms.is_empty() { review = review.child(small("No zooms")); }
                    for zoom in zooms {
                        review = review.child(small(format!("{}–{} · {:.1}×", fmt_time(zoom.start), fmt_time(zoom.end), zoom.scale)));
                    }
                }
            }
            if let Some(s) = p.style {
                for change in style_changes(self.style, s) {
                    review = review.child(small(change));
                }
            }
            review = review.child(small("Clip and zoom intervals refer to the original recording."));
            body = body.child(group().flex_none().child(review));
        }
        let mut composer = div()
            .flex_none()
            .p_4()
            .border_t_1()
            .border_color(separator())
            .flex()
            .flex_col()
            .gap_3()
            .when(self.assistant.proposal.is_some(), |d| {
                d.child(primary_button("apply-agent", "Apply edits to timeline")
                    .w_full()
                    .on_click(cx.listener(|this, _, _, cx| this.apply_agent(cx))))
            })
            .child(self.assistant.input.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(small("MiMo 2.6 Pro"))
                    .child(
                        button("send-agent", if busy { "Cancel" } else { "Send" }).on_click(
                            cx.listener(|this, _, _, cx| {
                                if this.assistant.run.is_some() {
                                    this.assistant.run = None;
                                    this.assistant.status = "Edit cancelled.".into();
                                    cx.notify();
                                } else {
                                    this.send_agent(cx);
                                }
                            }),
                        ),
                    ),
            );
        composer = composer.child(small(
            "Video understanding sends up to 12 sampled frames to Gemini via Codegraff. MiMo receives observations and edit settings. Audio is not sent.",
        ));
        frosted(
            16.,
            MENU_BLUR,
            div()
                .w(px(300.))
                .flex_none()
                .bg(alpha(CHROME, 0.26))
                .m_3()
                .ml_0()
                .rounded(px(16.))
                .border_1()
                .border_color(separator())
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(48.))
                        .px_4()
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(separator())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(icon(icons::GRAFF).size(px(18.)).text_color(rgb(ACCENT)))
                                .child(section_title("Codegraff")),
                        )
                        .child(toolbar_button("close-agent", "×").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.assistant.visible = false;
                                cx.notify();
                            },
                        ))),
                )
                .child(body)
                .child(composer),
        )
        .into_any_element()
    }
}
