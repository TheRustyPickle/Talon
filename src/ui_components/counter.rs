use eframe::egui;
use egui::{Align, Button, Grid, Label, Layout, ProgressBar, TextEdit, Ui};

use crate::ui_components::MainWindow;

#[derive(Default, Clone)]
pub struct CounterData {
    selected_session: String,
    start_from: String,
    end_at: String,
    num: u32,
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
                        ui.label(format!("{}", &mut self.counter_data.num));
                        ui.label("Messages Checked:");
                    });

                    ui.end_row();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.num));
                        ui.label("Whitelisted Messages:");
                    });
                    ui.end_row();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.num));
                        ui.label("Users Found:");
                    });
                    ui.end_row();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.counter_data.num));
                        ui.label("Whitelisted Users:");
                    });
                    ui.end_row();
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(80.0);
                let start_button = ui.add_sized([80.0, 40.0], Button::new("Start"));
                if start_button.clicked() {
                    self.start_counting();
                }
            });
        });

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.label("Status: Idle");
            ui.separator();
            ui.add_space(5.0);
            let mut progress_bar =
                ProgressBar::new((self.counter_data.num / 100) as f32).show_percentage();
            if self.counter_data.num > 1 {
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
            ui.add(Label::new(format!(
                "{}",
                &mut self.counter_data.selected_session
            )));
        });
        ui.end_row();

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Starting Point:"));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let clear_button = ui.add(Button::new("Clear"));
            if clear_button.clicked() {
                self.counter_data.start_from = String::new();
            }
            clear_button.on_hover_text("Clear text box content");
            let text_box = ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.counter_data.start_from)
                    .hint_text("https://t.me/chat_name/1234"),
            );
            text_box.on_hover_text(
                "The message link from where message counting should start.

Multiple input format is supported:

1. https://t.me/chat_name/1234
2. t.me/chat_name/1234
3. @chat_name
4. chat_name

If message number is not specified, starts from the latest message.",
            );
        });

        ui.end_row();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Ending Point:"));
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let clear_button = ui.add(Button::new("Clear"));
            if clear_button.clicked() {
                println!("clicked");
                self.counter_data.end_at = String::new();
            }
            clear_button.on_hover_text("Clear text box content");
            let text_box = ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.counter_data.end_at)
                    .hint_text("(Optional) https://t.me/chat_name/1234"),
            );
            text_box.on_hover_text(
                "Optional message link where counting will stop after including it.

Multiple input format is supported:

1. https://t.me/chat_name/1234
2. t.me/chat_name/1234
3. @chat_name
4. chat_name

If message number is not specified or is empty, counts all messages.",
            );
        });
        ui.end_row();
    }
}
