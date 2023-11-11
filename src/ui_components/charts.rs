use eframe::egui::Ui;

use crate::ui_components::MainWindow;

#[derive(Default)]
pub struct ChartsData {}

impl MainWindow {
    pub fn show_charts_ui(&mut self, ui: &mut Ui) {
        ui.label("Work in progress");
    }
}
