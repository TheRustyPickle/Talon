use eframe::egui::Ui;

use crate::ui_components::MainWindow;

#[derive(Default)]
pub struct WhitelistData {}

impl MainWindow {
    pub fn show_whitelist_ui(&mut self, ui: &mut Ui) {
        ui.label("Work in progress");
    }
}