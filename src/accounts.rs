//! Harness Agents account cards adapted for Recorder; login stays in graff's shared store.
use super::*;
use crate::agent;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

const PROVIDERS: &[(&str, &str, &str)] = &[
    ("codegraff", "Codegraff", "Gemini 3.8 Flash · MiMo 2.6 Pro"),
    ("chatgpt", "ChatGPT", "Shared ChatGPT agent sign-in"),
    ("kimi", "Kimi", "Shared Kimi Code sign-in"),
    ("xai", "xAI", "Shared Grok sign-in"),
    ("zai", "Z.AI", "Shared coding plan sign-in"),
];

fn saved_login(provider: &str) -> bool {
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    match provider {
        "codegraff" => {
            std::env::var("CODEGRAFF_API_KEY").is_ok_and(|s| !s.trim().is_empty())
                || std::fs::read(home.join(".simple-harness-codegraff.json"))
                    .ok()
                    .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
                    .is_some_and(|v| v["api_key"].as_str().is_some_and(|k| !k.is_empty()))
        }
        "chatgpt" => {
            home.join(".graff/credentials/chatgpt-new.json").is_file()
                || home.join(".openai/credentials/graff-oauth.json").is_file()
        }
        id => home
            .join(format!(".{id}/credentials/graff-oauth.json"))
            .is_file(),
    }
}
struct Login {
    cancel: Arc<AtomicBool>,
    events: mpsc::Receiver<(String, bool)>,
}
impl Drop for Login {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Login {
    fn start(provider: &'static str) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let (tx, events) = mpsc::channel();
        std::thread::spawn(move || {
            let Some(binary) = agent::binary() else {
                let _ = tx.send(("Install graff through Harness first.".into(), true));
                return;
            };
            let mut command = Command::new(binary);
            command.arg("login");
            if provider != "codegraff" {
                command.arg(provider);
            }
            let Ok(mut child) = command
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            else {
                let _ = tx.send(("Could not start sign-in.".into(), true));
                return;
            };
            let stdout = child.stdout.take().unwrap();
            let logs = tx.clone();
            let output = std::thread::spawn(move || {
                let mut confirmed = false;
                for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                    // Display only graff's device-flow instructions, never credentials.
                    let line = line.trim();
                    confirmed |= line.starts_with('✓');
                    if line.starts_with("https://codegraff.com/cli/auth")
                        || line.starts_with("https://auth.openai.com/")
                        || line.contains("enter code:")
                        || line.contains("confirm the code:")
                        || line.starts_with("✓")
                        || line.starts_with("✗")
                    {
                        let _ = logs.send((line.to_owned(), false));
                    }
                }
                confirmed
            });
            let started = std::time::Instant::now();
            loop {
                if flag.load(Ordering::Relaxed) || started.elapsed() > Duration::from_secs(600) {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let confirmed = output.join().unwrap_or(false);
                        let message = if status.success() && confirmed && saved_login(provider) {
                            "Login saved. Shared with Harness."
                        } else {
                            "Sign-in did not finish. Try again in Harness."
                        };
                        let _ = tx.send((message.into(), true));
                        break;
                    }
                    Err(_) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        let _ = tx.send(("Sign-in stopped.".into(), true));
                        break;
                    }
                    _ => std::thread::sleep(Duration::from_millis(100)),
                }
            }
        });
        Self { cancel, events }
    }
}

pub(super) struct Agents {
    studio: WeakEntity<Studio>,
    login: Option<Login>,
    status: String,
    _poll: Task<()>,
}
impl Agents {
    pub fn new(studio: WeakEntity<Studio>, cx: &mut Context<Self>) -> Self {
        let poll = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                if this
                    .update(cx, |this: &mut Self, cx| {
                        let mut done = false;
                        if let Some(login) = &this.login {
                            while let Ok((message, finished)) = login.events.try_recv() {
                                this.status = message;
                                done |= finished;
                            }
                        }
                        if done {
                            this.login = None;
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            studio,
            login: None,
            status: String::new(),
            _poll: poll,
        }
    }
}
impl Render for Agents {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cards =
            PROVIDERS
                .iter()
                .map(|&(id, name, detail)| {
                    let saved = saved_login(id);
                    frosted(16., MENU_BLUR, glass_card()
                    .p_5()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .size(px(40.))
                            .rounded(px(12.))
                            .bg(alpha(ACCENT, 0.10))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                crate::icons::icon(crate::icons::GRAFF)
                                    .size(px(22.))
                                    .text_color(rgb(ACCENT)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(name),
                            )
                            .child(small(detail))
                            .child(
                                div()
                                    .mt_2()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().size(px(5.)).rounded_full().bg(rgb(if saved {
                                        SUCCESS
                                    } else {
                                        TERTIARY
                                    })))
                                    .child(small(if saved {
                                        "Saved login · shared with Harness"
                                    } else {
                                        "Not connected"
                                    })),
                            ),
                    )
                    .child(
                        button(id, if saved { "Reconnect" } else { "Connect" }).on_click(
                            cx.listener(move |this, _, _, cx| {
                                if this.login.is_none() {
                                    this.status = format!(
                                        "Signing in to {name}… Complete approval in your browser."
                                    );
                                    this.login = Some(Login::start(id));
                                    cx.notify();
                                }
                            }),
                        ),
                    ))
                })
                .collect::<Vec<_>>();
        layered(div().size_full().bg(glass_shell()).flex().flex_col()
            .child(chrome::toolbar().h(px(52.)).pl(px(84.)).gap_3()
                .child(toolbar_button("back-agents", "← Capture").on_click(cx.listener(|this, _, _, cx| {
                    let _ = this.studio.update(cx, |studio, cx| studio.go_home(cx));
                })))
                .child(small("/"))
                .child(section_title("Agents")))
            .child(div().id("accounts-scroll").flex_1().min_h_0().overflow_y_scroll().p_6()
                .child(div().w_full().max_w(px(760.)).mx_auto().flex().flex_col().gap_3()
                    .child(div().pt_4().pb_5().flex().flex_col().gap_2()
                        .child(small("CONNECTED TO YOUR WORKFLOW").text_color(rgb(ACCENT)).font_weight(FontWeight::SEMIBOLD))
                        .child(div().text_size(px(30.)).font_weight(FontWeight::SEMIBOLD).child("Your agents. Already at home."))
                        .child(small("The same accounts you use in Harness. Connect once, use them here.")))
                    .children(cards)
                    .child(small("Codegraff powers Gemini video understanding and MiMo edits. Credentials stay in graff’s shared store.").mt_3())
                    .when(!self.status.is_empty(), |d| d.child(chrome::input().h_auto().p_3().child(small(self.status.clone()))))
                    .when(self.login.is_some(), |d| d.child(button("cancel-login", "Cancel sign-in").on_click(cx.listener(|this, _, _, cx| {
                        this.login = None;
                        this.status = "Sign-in cancelled.".into();
                        cx.notify();
                    }))))))
        )
    }
}
