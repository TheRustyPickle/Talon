use arboard::Clipboard;
use eframe::egui::{vec2, Align, Button, ComboBox, Grid, Label, Layout, ProgressBar, TextEdit, Ui};
use log::info;
use std::thread;

use crate::tg_handler::ProcessStart;
use crate::ui_components::processor::ProcessState;
use crate::ui_components::MainWindow;
use crate::utils::parse_tg_chat;

#[derive(Default, Clone)]
pub struct CounterData {
    session_index: usize,
    start_from: String,
    end_at: String,
    total_message: i32,
    whitelisted_message: i32,
    total_user: i32,
    whitelisted_user: i32,
    deleted_message: i32,
    bar_percentage: f32,
    counting: bool,
}

impl CounterData {
    pub fn get_start_from(&self) -> String {
        self.start_from.to_owned()
    }

    pub fn get_end_at(&self) -> String {
        self.end_at.to_owned()
    }

    pub fn counting_started(&mut self) {
        self.counting = true;
        self.bar_percentage = 0.0;
        self.total_user = 0;
        self.total_message = 0;
        self.whitelisted_message = 0;
        self.whitelisted_user = 0;
        self.deleted_message = 0;
    }

    pub fn counting_ended(&mut self) {
        if self.counting {
            self.counting = false;
            self.bar_percentage = 1.0;
        }
    }

    pub fn set_bar_percentage(&mut self, percentage: f32) {
        self.bar_percentage = percentage
    }

    pub fn set_total_user(&mut self, total_user: i32) {
        self.total_user = total_user;
    }

    pub fn add_one_total_message(&mut self) {
        self.total_message += 1;
    }

    pub fn add_deleted_message(&mut self, to_add: i32) {
        if to_add > 0 {
            self.deleted_message += to_add;
        }
    }
}

impl MainWindow {
    pub fn show_counter_ui(&mut self, ui: &mut Ui) {
        Grid::new("Counter Grid")
            .num_columns(2)
            .spacing([5.0, 10.0])
            .show(ui, |ui| self.show_grid_data(ui));

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            Grid::new("Main")
                .num_columns(2)
                .spacing([5.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Messages Checked:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.total_message));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Whitelisted Messages:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.whitelisted_message));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Users Found:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.total_user));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Whitelisted Users:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.whitelisted_user));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Deleted Message:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.deleted_message));
                    });

                    ui.end_row();
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(80.0);
                if self.is_processing {
                    ui.add_enabled(false, Button::new("Start").min_size(vec2(80.0, 40.0)));
                } else {
                    let start_button = ui.add_sized([80.0, 40.0], Button::new("Start"));
                    if start_button.clicked() {
                        self.start_counting();
                    }
                };
            });
        });

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            let progress_bar = ProgressBar::new(self.counter_data.bar_percentage)
                .show_percentage()
                .animate(self.counter_data.counting);
            ui.add(progress_bar);
        });
    }

    fn show_grid_data(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Selected Session:"));
        });
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let values = {
                let names = self.get_session_names();

                if names.is_empty() {
                    vec!["No Session Found".to_string()]
                } else {
                    names
                }
            };
            ComboBox::from_id_source("Session Box").show_index(
                ui,
                &mut self.counter_data.session_index,
                self.tg_clients.len(),
                |i| &values[i],
            );
        });
        ui.end_row();

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Starting Point:"));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let clear_paste_button = if !self.counter_data.start_from.is_empty() {
                ui.add(Button::new("Clear"))
                    .on_hover_text("Clear text box content")
            } else {
                ui.add(Button::new("Paste"))
                    .on_hover_text("Paste copied content")
            };

            if clear_paste_button.clicked() {
                if self.counter_data.start_from.is_empty() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Ok(copied_content) = clipboard.get_text() {
                            self.counter_data.start_from = copied_content
                        }
                    }
                } else {
                    self.counter_data.start_from = String::new();
                }
            }
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.counter_data.start_from)
                    .hint_text("https://t.me/chat_name/1234"),
            )
            .on_hover_text(
                "The message link from where message counting should start.

Multiple input format is supported:

1. https://t.me/chat_name/1234
2. t.me/chat_name/1234
3. @chat_name
4. chat_name

If message number is not specified, starts from the latest message.
Starting message number will always be bigger than the ending message.
To count all messages in a chat, paste the latest message link.",
            );
        });

        ui.end_row();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Ending Point:"));
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let clear_paste_button = if !self.counter_data.end_at.is_empty() {
                ui.add(Button::new("Clear"))
                    .on_hover_text("Clear text box content")
            } else {
                ui.add(Button::new("Paste"))
                    .on_hover_text("Paste copied content")
            };

            if clear_paste_button.clicked() {
                if self.counter_data.end_at.is_empty() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Ok(copied_content) = clipboard.get_text() {
                            self.counter_data.end_at = copied_content
                        }
                    }
                } else {
                    self.counter_data.end_at = String::new();
                }
            }
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.counter_data.end_at)
                    .hint_text("(Optional) https://t.me/chat_name/1234"),
            )
            .on_hover_text(
                "Optional message link where counting will stop after including it.

Multiple input format is supported:

1. https://t.me/chat_name/1234
2. t.me/chat_name/1234
3. @chat_name
4. chat_name

If message number is not specified or is empty, counts all messages.
Ending message number will always be smaller than the starting message.
To count all messages in a chat, paste the very first message link or keep it empty.",
            );
        });
        ui.end_row();
    }

    fn start_counting(&mut self) {
        let selected_client = self.get_selected_session();

        if selected_client.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        let start_from = self.counter_data.get_start_from();
        let end_at = self.counter_data.get_end_at();

        let (start_chat, start_num) = parse_tg_chat(start_from);
        let (end_chat, end_num) = parse_tg_chat(end_at);

        if start_chat.is_none() {
            self.process_state = ProcessState::InvalidStartChat;
            return;
        }

        let start_chat = start_chat.unwrap();

        if let Some(end_chat) = end_chat {
            if end_chat != start_chat {
                self.process_state = ProcessState::UnmatchedChat;
                return;
            }
        }

        if let (Some(start_num), Some(end_num)) = (start_num, end_num) {
            if start_num < end_num {
                self.process_state = ProcessState::SmallerStartNumber;
                return;
            }
        }

        info!("Starting counting");
        self.user_table.clear_row_data();
        self.charts_data.reset_chart();
        self.process_state = ProcessState::Counting(0);
        self.counter_data.counting_started();
        self.is_processing = true;

        let client = self.tg_clients.get(&selected_client).unwrap().clone();

        thread::spawn(move || {
            client.start_process(ProcessStart::StartCount(start_chat, start_num, end_num));
        });
    }

    /// Returns the session name that is selected on the combo box
    pub fn get_selected_session(&self) -> String {
        let all_sessions = self.get_session_names();
        let selected_index = self.counter_data.session_index;
        if let Some(session) = all_sessions.get(selected_index) {
            session.to_owned()
        } else {
            String::new()
        }
    }
}
