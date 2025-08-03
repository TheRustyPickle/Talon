use eframe::egui::{vec2, Align, Button, CentralPanel, Context, Grid, Label, Layout, TextEdit};
use serde::{Deserialize, Serialize};

use crate::ui_components::processor::AppState;
use crate::ui_components::MainWindow;
use crate::utils::save_api_keys;

#[derive(Deserialize, Serialize, Default)]
pub struct TGKeys {
    pub api_id: String,
    pub api_hash: String,
}

impl MainWindow {
    pub fn show_tg_keys_ui(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            let tg_link = "https://my.telegram.org/";
            ui.label("A valid Telegram API key pair is necessary for the app to work properly. \
                The key will be saved locally and will never be shared. \
                To get your API key, visit the link below ➡ API development tools ➡ Create a new application");

            ui.add_space(20.0);
            Grid::new("Keys Grid")
                .num_columns(2)
                .spacing([5.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Label::new("API Link:"));
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if ui.link(tg_link).clicked() {
                            open::that(tg_link).unwrap();
                        }
                    });
                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Label::new("API ID:"));
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.tg_keys.api_id)
                                .hint_text("12345678")
                                .min_size(ui.available_size()),
                        );
                    });
                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Label::new("API Hash:"));
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.tg_keys.api_hash)
                                .hint_text("12345abcdef")
                                .min_size(ui.available_size()),
                        );
                    });
                    ui.end_row();
                });
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(vec2(80.0, 40.0), Button::new("Save Keys"))
                    .clicked()
                    && !self.tg_keys.api_hash.is_empty()
                    && !self.tg_keys.api_id.is_empty()
                {
                    save_api_keys(&self.tg_keys);
                    self.app_state = AppState::InitializedUI;
                }
            });
        });
    }
}
