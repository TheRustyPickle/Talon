use arboard::Clipboard;
use eframe::egui::{vec2, Align, Button, Grid, Label, Layout, ProgressBar, TextEdit, Ui};

use crate::ui_components::MainWindow;

#[derive(Default, Clone)]
pub struct CounterData {
    selected_session: String,
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
    pub fn get_selected_session(&self) -> String {
        self.selected_session.to_owned()
    }

    pub fn update_selected_session(&mut self, name: String) {
        self.selected_session = name;
    }

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
        self.deleted_message += to_add;
    }
}

impl MainWindow {
    pub fn show_counter_ui(&mut self, ui: &mut Ui) {
        Grid::new("my grid")
            .num_columns(2)
            .spacing([5.0, 10.0])
            .show(ui, |ui| self.show_grid_data(ui));

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            Grid::new("status")
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
            let mut progress_bar =
                ProgressBar::new(self.counter_data.bar_percentage).show_percentage();
            if self.is_processing {
                progress_bar = progress_bar.animate(true);
            }
            ui.add(progress_bar);
        });
    }

    fn show_grid_data(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Selected Session:"));
        });
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add(Label::new(self.counter_data.selected_session.to_string()));
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
}
