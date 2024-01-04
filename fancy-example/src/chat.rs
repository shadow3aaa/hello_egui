use std::iter::repeat;
use std::sync::Arc;
use std::time::Duration;
use std::usize;

use eframe::emath::Vec2;
use egui::{Align, Frame, Label, Layout, RichText, ScrollArea, Ui, Widget};

use egui_animation::animate_continuous;
use egui_inbox::UiInbox;
use egui_infinite_scroll::InfiniteScroll;

use crate::futures::{sleep, spawn};
use crate::shared_state::SharedState;
use crate::sidebar::Example;

pub const CHAT_HISTORY: &str = include_str!("chat_history.txt");

pub const CHAT_MESSAGES: &str = include_str!("chat.txt");

#[derive(Debug)]
struct HistoryLoader {
    history: Vec<ChatMessage>,
    messages: Vec<(ChatMessage, Duration)>,
}

impl HistoryLoader {
    pub fn new() -> Self {
        let history: Vec<_> = CHAT_HISTORY
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (name, content) = line.split_once(": ").unwrap();

                ChatMessage {
                    content: content.to_string(),
                    from: if name == "me" {
                        None
                    } else {
                        Some(name.to_string())
                    },
                }
            })
            .rev()
            .collect();

        // Repeat the history 5 times to make it longer.
        let history = repeat(history)
            .take(5)
            .flat_map(|history| history.clone())
            .collect();

        Self {
            history,

            messages: CHAT_MESSAGES
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| {
                    let (name, content) = line.split_once(": ").unwrap();

                    let (name, duration) = name.split_once(", ").unwrap();

                    let duration = Duration::from_secs_f32(duration.parse::<f32>().unwrap());

                    (
                        ChatMessage {
                            content: content.to_string(),
                            from: if name == "me" {
                                None
                            } else {
                                Some(name.to_string())
                            },
                        },
                        duration,
                    )
                })
                .collect(),
        }
    }

    pub async fn load(&self, page: Option<usize>) -> (Vec<ChatMessage>, Option<usize>) {
        let page = page.unwrap_or(0);
        sleep(Duration::from_secs_f32(0.7)).await;
        let page_size = 10;
        let start = page * page_size;
        let end = usize::min(start + page_size, self.history.len());

        let has_more = end < self.history.len();

        let messages = self.history[start..end].iter().cloned().rev().collect();

        (messages, if has_more { Some(page + 1) } else { None })
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub content: String,
    pub from: Option<String>,
}

#[derive(Debug)]
pub struct ChatExample {
    messages: InfiniteScroll<ChatMessage, usize>,
    inbox: UiInbox<ChatMessage>,
    history_loader: Arc<HistoryLoader>,
    shown: bool,
    msgs_received: usize,
}

impl ChatExample {
    pub fn new() -> Self {
        let history_loader = Arc::new(HistoryLoader::new());

        let inbox = UiInbox::new();

        let history_loader_clone = history_loader.clone();

        ChatExample {
            messages: InfiniteScroll::new().start_loader(move |cursor, cb| {
                println!("Loading messages...");
                let history_loader = history_loader_clone.clone();
                spawn(async move {
                    let (messages, cursor) = history_loader.load(cursor).await;

                    dbg!(&messages);

                    cb(Ok((messages, cursor)));
                });
            }),
            inbox,
            history_loader,
            shown: false,
            msgs_received: 0,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if !self.shown {
            self.shown = true;

            let inbox_clone = self.inbox.clone();
            self.history_loader
                .messages
                .iter()
                .for_each(|(message, duration)| {
                    let inbox_clone = inbox_clone.clone();
                    let duration = *duration;
                    let message = message.clone();
                    spawn(async move {
                        sleep(duration).await;
                        inbox_clone.send(message);
                    });
                });
        }

        self.inbox.read(ui).for_each(|message| {
            self.messages.items.push(message);
            self.msgs_received += 1;
        });

        ScrollArea::vertical()
            .max_height(200.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.vertical_centered(|ui| {
                    ui.set_visible(self.messages.top_loading_state().loading());
                    ui.spinner();
                });

                let max_msg_width = ui.available_width() - 20.0;
                let inner_margin = 8.0;
                let outer_margin = 8.0;

                self.messages.ui(ui, 10, |ui, _index, item| {
                    let is_message_from_myself = item.from.is_some();

                    // Messages from the user are right-aligned.
                    let layout = if is_message_from_myself {
                        Layout::top_down(Align::Min)
                    } else {
                        Layout::top_down(Align::Max)
                    };

                    ui.with_layout(layout, |ui| {
                        ui.set_max_width(max_msg_width);

                        let mut measure = |text| {
                            let label = Label::new(text);
                            // We need to calculate the text width here to enable the typical
                            // chat bubble layout where the own bubbles are right-aligned and
                            // the text within is left-aligned.
                            let (_pos, galley, _response) =
                                label.layout_in_ui(&mut ui.child_ui(ui.max_rect(), *ui.layout()));
                            let rect = galley.galley.rect;
                            // Calculate the width of the frame based on the width of
                            // the text and add 0.1 to account for floating point errors.
                            f32::min(
                                rect.width() + inner_margin * 2.0 + outer_margin * 2.0 + 0.1,
                                max_msg_width,
                            )
                        };

                        let content = RichText::new(&item.content);
                        let mut msg_width = measure(content.clone());
                        let name = if let Some(from) = &item.from {
                            let name = RichText::new(from).strong();
                            let width = measure(name.clone());
                            msg_width = f32::max(msg_width, width);
                            Some(name)
                        } else {
                            None
                        };

                        // Set the width of the ui to the width of the message.
                        ui.set_min_width(msg_width);

                        // ui.allocate_exact_size(
                        //     Vec2::new(rect.width() + 16.0, 0.0),
                        //     egui::Sense::hover(),
                        // );

                        Frame::none()
                            .rounding(8.0)
                            .inner_margin(8.0)
                            .outer_margin(8.0)
                            .fill(if is_message_from_myself {
                                ui.style().visuals.faint_bg_color
                            } else {
                                ui.style().visuals.extreme_bg_color
                            })
                            .show(ui, |ui| {
                                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                                    if let Some(from) = name {
                                        Label::new(from).ui(ui);
                                    }

                                    ui.label(&item.content);
                                });
                            });
                    });
                });

                if self.msgs_received < self.history_loader.messages.len()
                    && !self.messages.initial_loading()
                {
                    Frame::none()
                        .rounding(8.0)
                        .inner_margin(8.0)
                        .outer_margin(8.0)
                        .fill(ui.style().visuals.faint_bg_color)
                        .show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                let mut dot = |offset| {
                                    let t = animate_continuous(
                                        ui,
                                        egui_animation::easing::sine_in_out,
                                        Duration::from_secs_f32(1.0),
                                        offset,
                                    );

                                    let res = ui
                                        .allocate_response(Vec2::splat(4.0), egui::Sense::hover());

                                    ui.painter().circle_filled(
                                        res.rect.center() + Vec2::Y * t * 4.0,
                                        res.rect.width() / 2.0,
                                        ui.style().visuals.text_color(),
                                    )
                                };

                                dot(0.0);
                                dot(0.3);
                                dot(8.6);
                            });
                        });
                }
            });
    }
}

impl Example for ChatExample {
    fn name(&self) -> &'static str {
        "Chat"
    }

    fn ui(&mut self, ui: &mut Ui, _shared_state: &mut SharedState) {
        self.ui(ui)
    }
}
