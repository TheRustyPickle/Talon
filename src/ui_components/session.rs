use eframe::egui::Ui;

use crate::ui_components::MainWindow;

#[derive(Default)]
pub struct SessionData {}

impl MainWindow {
    pub fn show_session_ui(&mut self, ui: &mut Ui) {
        ui.label("Work in progress");
    }
}