use super::*;
use crate::icons::{self, icon};

impl Studio {
    pub(super) fn render_home(&mut self, cx: &mut Context<Self>) -> AnyElement {
        layered(div().size_full().flex().bg(glass_shell())
            .child(self.render_library(cx))
            .child(div().flex_1().min_w_0().h_full().flex().flex_col()
                .child(chrome::toolbar().h(px(52.)).justify_between()
                    .child(div().flex().items_center().gap_2()
                        .child(icon(icons::MONITOR).size(px(16.)).text_color(rgb(MUTED)))
                        .child(section_title("Capture")))
                    .child(div().flex().items_center().gap_2().px_3().py_1().rounded_full().bg(alpha(SUCCESS, 0.06))
                        .child(div().size(px(5.)).rounded_full().bg(rgb(if self.screen_access || self.tab == Tab::Event { SUCCESS } else { ACCENT })))
                        .child(small(if self.screen_access || self.tab == Tab::Event { "Ready to record" } else { "Screen access needed" }))))
                .child(div().id("home").flex_1().min_h_0().overflow_y_scroll().p_6()
                    .child(div().w_full().max_w(px(920.)).mx_auto().flex().flex_col().gap_6()
                        .child(div().flex().flex_col().gap_3().pt_4().pb_2()
                            .child(div().flex().items_center().gap_2()
                                .child(div().size(px(6.)).rounded_full().bg(rgb(ACCENT)))
                                .child(small("YOUR NEXT GREAT TAKE").text_color(rgb(ACCENT)).font_weight(FontWeight::SEMIBOLD)))
                            .child(div().text_size(px(34.)).font_weight(FontWeight::SEMIBOLD).child("A little capture. A lot of possibility."))
                            .child(div().text_size(px(14.)).text_color(rgb(MUTED)).child("Capture your screen or record yourself at an event. Shape your story and share it.")))
                        .child(div().flex().gap_4()
                            .child(frosted(16., MENU_BLUR, self.render_new_recording(cx)))
                            .child(frosted(16., MENU_BLUR, self.render_editing_intro(cx)))))))
            ).into_any_element()
    }

    fn render_editing_intro(&mut self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w(px(288.))
            .flex_none()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                glass_card()
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(icon(icons::MAGIC).size(px(18.)).text_color(rgb(ACCENT)))
                            .child(section_title("From take to final cut")),
                    )
                    .child(
                        div()
                            .h(px(142.))
                            .p_4()
                            .rounded(px(10.))
                            .bg(gpui::linear_gradient(
                                135.,
                                gpui::linear_color_stop(alpha(ACCENT, 0.22), 0.),
                                gpui::linear_color_stop(alpha(PURPLE, 0.05), 1.),
                            ))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .w_full()
                                    .h(px(110.))
                                    .p_3()
                                    .rounded(px(8.))
                                    .bg(alpha(0xffffff, 0.065))
                                    .border_1()
                                    .border_color(alpha(ACCENT, 0.25))
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(div().flex().gap_1().children(
                                        [0..3].into_iter().flatten().map(|_| {
                                            div().size(px(4.)).rounded_full().bg(rgb(MUTED))
                                        }),
                                    ))
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                icon(icons::MONITOR)
                                                    .size(px(32.))
                                                    .text_color(rgb(ACCENT)),
                                            ),
                                    )
                                    .child(div().flex().gap_1().h(px(8.))
                                        .child(div().flex_1().rounded(px(3.)).bg(alpha(ACCENT, 0.7)))
                                        .child(div().w(px(38.)).rounded(px(3.)).bg(alpha(PURPLE, 0.55)))
                                        .child(div().w(px(24.)).rounded(px(3.)).bg(alpha(ACCENT, 0.35)))),
                            ),
                    )
                    .children(
                        [
                            (icons::CLOCK, "Trim, split, and change speed"),
                            (icons::MAGIC, "Automatic zooms and smooth cursors"),
                            (icons::PHONE, "Landscape, portrait, or square"),
                            (icons::GRAFF, "Refine your edit with Codegraff"),
                        ]
                        .map(|(i, label)| {
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(icon(i).size(px(16.)).text_color(rgb(MUTED)))
                                .child(small(label).text_color(rgb(TEXT)))
                        }),
                    )
                    .child(
                        button("try-editor", "Explore the editor")
                            .w_full()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_demo(window, cx)),
                            ),
                    ),
            )
            .child(
                div()
                    .px_2()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        small("Recorder")
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT)),
                    )
                    .child(small(
                        "Built on Shitty Screen Studio by Li Yang (@hewliyang).",
                    ))
                    .child(small("Harness UI · Codegraff editing")),
            )
            .into_any_element()
    }

    fn render_library(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let rows: Vec<_> = self
            .recordings
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let dir = r.dir.clone();
                div()
                    .id(("recent", i))
                    .flex()
                    .items_center()
                    .px(px(10.))
                    .py(px(10.))
                    .gap_3()
                    .rounded(px(8.))
                    .cursor_default()
                    .hover(|s| s.bg(alpha(0xffffff, 0.07)))
                    .active(|s| s.bg(alpha(ACCENT, 0.18)))
                    .child(div().size(px(32.)).flex_none().rounded(px(9.))
                        .bg(alpha(ACCENT, 0.08)).border_1().border_color(alpha(ACCENT, 0.12))
                        .flex().items_center().justify_center()
                        .child(icon(icons::MONITOR).size(px(16.)).text_color(rgb(ACCENT))))
                    .child(div().flex_1().min_w_0().flex().flex_col().gap_1().child(
                        div()
                            .text_size(px(BODY))
                            .text_color(rgb(TEXT))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(r.title.clone()),
                    )
                    .child(small(r.detail.clone()).font_features(tabular()).text_size(px(10.))))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_project(dir.clone(), window, cx)
                    }))
            })
            .collect();
        div()
            .w(px(220.))
            .flex_none()
            .h_full()
            .flex()
            .flex_col()
            .bg(alpha(CHROME, 0.98))
            .border_r_1()
            .border_color(separator())
            .child(
                div()
                    .h(px(80.))
                    .flex_none()
                    .px_5()
                    .pt(px(38.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .size(px(18.))
                            .rounded(px(6.))
                            .bg(rgb(ACCENT))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().size(px(7.)).rounded_full().bg(rgb(0xffffff))),
                    )
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Recorder"),
                    ),
            )
            .child(
                div().p_3().child(
                    div().id("new-recording").flex().items_center().h(px(36.)).px_3().rounded(px(9.))
                        .w_full()
                        .justify_start()
                        .gap_2()
                        .bg(alpha(ACCENT, 0.12))
                        .text_color(rgb(ACCENT)).text_size(px(BODY)).font_weight(FontWeight::MEDIUM)
                        .child(icon(icons::PLUS).size(px(16.)))
                        .child("New recording"),
                ),
            )
            .child(
                div().px_3().pb_3().child(
                    div().id("agents").flex().items_center().h(px(36.)).px_3().rounded(px(9.))
                        .w_full()
                        .justify_start()
                        .gap_2()
                        .text_size(px(BODY)).text_color(rgb(MUTED)).cursor_default()
                        .hover(|s| s.bg(alpha(0xffffff, 0.05)).text_color(rgb(TEXT)))
                        .child(icon(icons::GRAFF).size(px(16.)))
                        .child("Agents")
                        .on_click(cx.listener(|this, _, _, cx| this.open_agents(cx))),
                ),
            )
            .child(group_title("Library").px(px(20.)).pt_3().pb_2())
            .child(
                div()
                    .id("library")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(10.))
                    .flex()
                    .flex_col()
                    .gap(px(1.))
                    .when(rows.is_empty(), |d| {
                        d.child(small("No recordings yet.").px(px(10.)).py_1())
                    })
                    .children(rows),
            )
            .child(
                div()
                    .flex_none()
                    .p(px(10.))
                    .border_t_1()
                    .border_color(separator())
                    .child(
                        toolbar_button("demo", "Open Demo Recording").on_click(
                            cx.listener(|this, _, window, cx| this.open_demo(window, cx)),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn render_new_recording(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let can_record = self.source().is_some();
        glass_card()
            .flex_1()
            .min_w_0()
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(px(18.)).font_weight(FontWeight::BOLD).child("New recording"))
                    .child(
                        div()
                            .text_size(px(BODY))
                            .text_color(rgb(MUTED))
                            .child("Choose a source and the inputs for this take."),
                    ),
            )
            .child(segments()
                .child(segment("mode-screen", "Screen", self.tab != Tab::Event).flex_1()
                    .on_click(cx.listener(|this, _, _, cx| { this.tab = Tab::Display; cx.notify(); })))
                .child(segment("mode-event", "Event · Self recorder", self.tab == Tab::Event).flex_1()
                    .on_click(cx.listener(|this, _, _, cx| {
                        if this.tab != Tab::Event { this.options.mic = true; }
                        this.tab = Tab::Event;
                        this.cameras = crate::camera::list_devices(false);
                        this.microphones = crate::camera::list_devices(true);
                        cx.notify();
                    }))))
            .child(self.render_source(cx))
            .child(self.render_inputs(cx))
            .when_some(self.error.clone(), |d, e| {
                d.child(
                    div()
                        .p_3()
                        .rounded(px(10.))
                        .bg(alpha(RED, 0.12))
                        .border_1()
                        .border_color(alpha(RED, 0.35))
                        .text_size(px(BODY))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(e)
                        .when(self.permission_error, |d| d.child(button("error-settings", "Open Privacy Settings")
                            .on_click(cx.listener(|this, _, _, cx| this.open_privacy_settings(cx))))),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .id("record")
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .h(px(42.))
                            .rounded(px(10.))
                            .bg(rgb(ACCENT))
                            .border_t_1()
                            .border_color(alpha(0xffffff, 0.2))
                            .text_size(px(BODY))
                            .text_color(rgb(0x14121f))
                            .font_weight(FontWeight::SEMIBOLD)
                            .cursor_default()
                            .child(icon(icons::MONITOR).size(px(16.)).text_color(rgb(0x14121f)))
                            .child("Start recording")
                            .when(can_record, |d| d.hover(|s| s.bg(rgb(ACCENT_HOVER))))
                            .when(!can_record, |d| d.opacity(0.4))
                            .on_click(cx.listener(|this, _, window, cx| this.start_recording(window, cx))),
                    )
                    .child(
                        small(if self.tab == Tab::Event { format!("Records your webcam full-frame at up to 30 fps with your selected microphone. Press {} to stop, or use the floating bar.", crate::tray::STOP_SHORTCUT) } else { format!("Records at up to 120 fps. The app hides itself from the capture. Press {} to stop, or use the floating bar or the menu bar.", crate::tray::STOP_SHORTCUT) })
                            .text_color(rgb(TERTIARY))
                            .px(px(2.)),
                    ),
            )
            .into_any_element()
    }

    fn render_source(&mut self, cx: &mut Context<Self>) -> AnyElement {
        if self.tab == Tab::Event {
            return group().child(div().p_4().flex().flex_col().gap_2()
                .child(section_title("Your event, your perspective"))
                .child(small("Record a talk, an interview, or a personal update with your webcam as the full video. Choose your inputs below; your take opens in the editor when you stop.")))
                .into_any_element();
        }
        if !self.screen_access {
            return div()
                .flex()
                .flex_col()
                .gap_2()
                .child(group_title("Screen access"))
                .child(group().child(div().p_3().flex().flex_col().gap_3()
                    .child("Choose what you capture. Screen access lets Recorder list your displays and windows.")
                    .child(small("Enable Recorder in System Settings → Privacy & Security → Screen Recording. Then return and check access again. macOS may ask you to quit and reopen the app."))
                    .child(div().flex().flex_wrap().gap_2()
                        .when(!self.screen_request_attempted, |d| d.child(button("allow-screen", "Allow Screen Access…")
                            .on_click(cx.listener(|this, _, _, cx| this.request_screen_access(cx)))))
                        .when(self.loading_sources, |d| d.child(small("Waiting for macOS…")))
                        .child(button("screen-settings", "Open Settings")
                            .on_click(cx.listener(|this, _, _, cx| this.open_privacy_settings(cx))))
                        .child(button("check-screen", "Check Access Again")
                            .on_click(cx.listener(|this, _, _, cx| this.load_sources(cx)))))))
                .into_any_element();
        }
        let tabs = [
            (Tab::Display, "Display"),
            (Tab::Window, "Window"),
            (Tab::Area, "Area"),
        ]
        .map(|(tab, label)| {
            segment(label, label, self.tab == tab)
                .flex_1()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.tab = tab;
                    if tab == Tab::Window {
                        this.load_sources(cx);
                    }
                    cx.notify();
                }))
        });
        let row = |id: gpui::ElementId, first: bool, selected: bool, content: Div| {
            group_row(first)
                .id(id)
                .gap_2()
                .cursor_default()
                .hover(|s| s.bg(alpha(0xffffff, 0.04)))
                .child(content.flex_1().min_w_0().flex().items_center().gap_2())
                .child(check(selected))
        };
        let note = |s: String| {
            group()
                .child(group_row(true).child(small(s)))
                .into_any_element()
        };
        let sources = match &self.sources {
            None => Err("Looking for screens and windows…".to_string()),
            Some(Err(e)) => Err(e.clone()),
            Some(Ok(s)) => Ok(s.clone()),
        };
        let body = match (self.tab, sources) {
            (_, Err(msg)) if self.tab != Tab::Area => note(msg),
            (Tab::Display, Ok(s)) => {
                let current = self.display.or(s.displays.first().map(|d| d.id));
                group()
                    .children(s.displays.iter().enumerate().map(|(i, d)| {
                        let id = d.id;
                        let content = div()
                            .child(icon(icons::MONITOR).size(px(18.)).text_color(rgb(ACCENT)))
                            .child(div().flex_1().child(if d.main {
                                "Main Display".to_string()
                            } else {
                                format!("Display {}", i + 1)
                            }))
                            .child(
                                small(format!("{:.0} × {:.0}", d.width, d.height))
                                    .font_features(tabular()),
                            );
                        row(("display", i).into(), i == 0, current == Some(id), content).on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.display = Some(id);
                                cx.notify();
                            }),
                        )
                    }))
                    .into_any_element()
            }
            (Tab::Window, Ok(s)) if s.windows.is_empty() => note("No windows found.".into()),
            (Tab::Window, Ok(s)) => group()
                .child(
                    div()
                        .id("windows")
                        .max_h(px(260.))
                        .overflow_y_scroll()
                        .children(s.windows.iter().enumerate().map(|(i, w)| {
                            let id = w.id;
                            let content = div()
                                .child(match w.icon.clone() {
                                    Some(icon) => {
                                        img(icon).flex_none().size(px(20.)).into_any_element()
                                    }
                                    None => div()
                                        .flex_none()
                                        .size(px(20.))
                                        .rounded(px(5.))
                                        .bg(alpha(0xffffff, 0.1))
                                        .into_any_element(),
                                })
                                .child(div().flex_none().whitespace_nowrap().child(w.app.clone()))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .text_color(rgb(MUTED))
                                        .child(w.title.clone()),
                                );
                            row(
                                ("window", id as usize).into(),
                                i == 0,
                                self.window == Some(id),
                                content,
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.window = Some(id);
                                    cx.notify();
                                },
                            ))
                        })),
                )
                .into_any_element(),
            (_, _) => {
                let label = match self.area {
                    Some(Source::Area { w, h, x, y, .. }) => {
                        format!("{w:.0} × {h:.0} at {x:.0}, {y:.0}")
                    }
                    _ => "No area selected".into(),
                };
                group()
                    .child(
                        group_row(true)
                            .child(
                                div()
                                    .text_color(rgb(if self.area.is_some() { TEXT } else { MUTED }))
                                    .font_features(tabular())
                                    .child(label),
                            )
                            .child(
                                button(
                                    "pick-area",
                                    if self.area.is_some() {
                                        "Reselect…"
                                    } else {
                                        "Select Area…"
                                    },
                                )
                                .on_click(cx.listener(|this, _, _, cx| this.pick_area(cx))),
                            ),
                    )
                    .into_any_element()
            }
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(group_title("Source"))
            .child(segments().children(tabs))
            .child(body)
            .when(matches!(self.sources, Some(Err(_))), |d| {
                d.child(
                    button("retry-sources", "Check Access Again")
                        .on_click(cx.listener(|this, _, _, cx| this.load_sources(cx))),
                )
            })
            .into_any_element()
    }

    fn render_inputs(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let item = |id: &'static str,
                    label: &'static str,
                    first: bool,
                    on: bool,
                    f: fn(&mut RecordOptions)| {
            group_row(first)
                .id(id)
                .cursor_default()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            icon(match id {
                                "sys-audio" | "mic" => icons::AUDIO,
                                "keys" => icons::KEYBOARD,
                                _ => icons::EYE,
                            })
                            .size(px(15.))
                            .text_color(rgb(MUTED)),
                        )
                        .child(label),
                )
                .child(switch(on))
                .on_click(cx.listener(move |this, _, _, cx| {
                    f(&mut this.options);
                    cx.notify();
                }))
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(group_title("Inputs"))
            .child(
                group()
                    .child(item("mic", "Microphone", true, self.options.mic, |o| o.mic = !o.mic))
                    .when(self.tab != Tab::Event, |d| d.child(item("sys-audio", "System Audio", false, self.options.system_audio, |o| o.system_audio = !o.system_audio)))
                    .when(self.tab != Tab::Event, |d| d.child(item("camera", "Camera", false, self.options.camera, |o| o.camera = !o.camera)))
                    .when(self.tab != Tab::Event, |d| d.child(item("keys", "Keyboard Shortcuts", false, self.options.keys, |o| o.keys = !o.keys))),
            )
            .when(self.tab == Tab::Event || self.options.camera, |d| d.child(self.render_device_choices(false, cx)))
            .when(self.options.mic, |d| d.child(self.render_device_choices(true, cx)))
            .child(button("refresh-inputs", "Refresh cameras & microphones")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.cameras = crate::camera::list_devices(false);
                    this.microphones = crate::camera::list_devices(true);
                    cx.notify();
                })))
            .child(small(if self.tab == Tab::Event {
                "macOS asks for camera and microphone access when you start. Refresh inputs after connecting a device. Turn off Microphone for a silent take."
            } else {
                "macOS asks for camera and microphone access when you start. Keyboard Shortcuts uses Input Monitoring; plain typing is not saved."
            }))
            .into_any_element()
    }

    fn render_device_choices(&self, mic: bool, cx: &mut Context<Self>) -> AnyElement {
        let devices = if mic { &self.microphones } else { &self.cameras };
        let selected = if mic { &self.options.mic_device } else { &self.options.camera_device };
        let title = if mic { "Microphone device" } else { "Webcam" };
        let prefix = if mic { "microphone-device" } else { "camera-device" };
        div().flex().flex_col().gap_2().child(group_title(title))
            .child(group()
                .child(group_row(true).id((prefix, 0usize)).cursor_default()
                    .child("System default").child(check(selected.is_none()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if mic { this.options.mic_device = None; } else { this.options.camera_device = None; }
                        cx.notify();
                    })))
                .children(devices.iter().enumerate().map(|(i, device)| {
                    let id = device.id.clone();
                    group_row(false).id((prefix, i + 1)).cursor_default()
                        .child(div().flex_1().min_w_0().child(device.name.clone()))
                        .child(check(selected.as_deref() == Some(device.id.as_str())))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if mic { this.options.mic_device = Some(id.clone()); } else { this.options.camera_device = Some(id.clone()); }
                            cx.notify();
                        }))
                }))
                .when(devices.is_empty(), |d| d.child(group_row(false).child(small("No devices listed. Allow access when starting, then refresh inputs.")))))
            .when(selected.as_ref().is_some_and(|id| !devices.iter().any(|d| &d.id == id)), |d| {
                d.child(small("Selected device is disconnected. Choose a connected input or System default.").text_color(rgb(RED)))
            })
            .into_any_element()
    }

}
