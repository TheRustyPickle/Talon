use eframe::egui::{
    Align, CentralPanel, Color32, Layout, RichText, ScrollArea, TextEdit, Ui, vec2,
};
use serde::{Deserialize, Serialize};

use crate::ui::MainWindow;
use crate::ui::state::AppState;
use crate::ui::widgets::{Card, PrimaryButton};
use crate::utils::save_api_keys;

const TG_LINK: &str = "https://my.telegram.org/";

#[derive(Deserialize, Serialize, Default)]
pub struct TGKeys {
    pub api_id: String,
    pub api_hash: String,
}

impl TGKeys {
    /// Telegram API IDs are plain numbers
    fn id_error(&self) -> Option<&'static str> {
        let id = self.api_id.trim();
        if id.is_empty() {
            None
        } else if !id.chars().all(|c| c.is_ascii_digit()) {
            Some("The API ID only contains digits")
        } else {
            None
        }
    }

    /// Telegram API hashes are 32 hexadecimal characters
    fn hash_error(&self) -> Option<&'static str> {
        let hash = self.api_hash.trim();
        if hash.is_empty() {
            None
        } else if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            Some("The API hash only contains 0-9 and a-f")
        } else if hash.len() != 32 {
            Some("The API hash is 32 characters long")
        } else {
            None
        }
    }

    fn is_valid(&self) -> bool {
        !self.api_id.trim().is_empty()
            && !self.api_hash.trim().is_empty()
            && self.id_error().is_none()
            && self.hash_error().is_none()
    }
}

impl MainWindow {
    pub fn show_tg_keys_ui(&mut self, ui: &mut Ui) {
        CentralPanel::default().show(ui, |ui| {
            // Keep the form at a readable width on wide windows
            let width = ui.available_width().min(520.0);
            let side = ((ui.available_width() - width) / 2.0).max(0.0);

            // The window starts small, so let the form scroll instead of being cut off
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.set_width(width);
                            self.api_keys_form(ui);
                        });
                    });
                });
        });
    }

    fn api_keys_form(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(RichText::new("Connect to Telegram").heading());
        ui.label(
            RichText::new(
                "Talon needs your own Telegram API key to read your chats. \
                It is saved on this device only and never shared.",
            )
            .weak(),
        );
        ui.add_space(12.0);

        Card::new().title("GET YOUR KEY").show(ui, |ui| {
            let steps = [
                "Log in with your phone number",
                "Open API development tools",
                "Create a new application, any name works",
                "Copy the api_id and api_hash it shows",
            ];
            for (i, step) in steps.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}.", i + 1)).weak());
                    ui.label(*step);
                });
            }
            ui.add_space(6.0);
            if ui.link(format!("Open {TG_LINK}")).clicked() {
                let _ = open::that(TG_LINK);
            }
        });

        ui.add_space(8.0);

        Card::new().title("YOUR KEY").show(ui, |ui| {
            ui.label("API ID");
            ui.add(
                TextEdit::singleline(&mut self.tg_keys.api_id)
                    .hint_text("12345678")
                    .desired_width(f32::INFINITY),
            );
            field_error(ui, self.tg_keys.id_error());

            ui.add_space(6.0);

            ui.label("API Hash");
            ui.add(
                TextEdit::singleline(&mut self.tg_keys.api_hash)
                    .hint_text("0123456789abcdef0123456789abcdef")
                    .password(true)
                    .desired_width(f32::INFINITY),
            );
            field_error(ui, self.tg_keys.hash_error());
        });

        ui.add_space(14.0);

        let valid = self.tg_keys.is_valid();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let button =
                PrimaryButton::new("Save and Continue").size(vec2(ui.available_width(), 40.0));
            let response = ui.add_enabled(valid, button);
            let response = if valid {
                response
            } else {
                response.on_disabled_hover_text("Enter a valid API ID and API hash first")
            };
            if response.clicked() {
                self.tg_keys.api_id = self.tg_keys.api_id.trim().to_string();
                self.tg_keys.api_hash = self.tg_keys.api_hash.trim().to_string();
                save_api_keys(&self.tg_keys);
                self.app_state = AppState::InitializedUI;
            }
        });
    }
}

/// Small red hint under a field, takes no space when there is nothing to say
fn field_error(ui: &mut Ui, error: Option<&str>) {
    if let Some(error) = error {
        ui.label(
            RichText::new(error)
                .small()
                .color(Color32::from_rgb(214, 90, 90)),
        );
    }
}
