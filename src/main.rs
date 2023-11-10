mod tg_handler;
mod ui_components;
mod utils;

use crate::ui_components::MainWindow;
use dotenvy::dotenv;
use eframe::egui;
use eframe::Theme;
use egui::vec2;
use tracing::info;

fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("Starting app");

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(400.0, 300.0)),
        min_window_size: Some(vec2(400.0, 300.0)),
        max_window_size: Some(vec2(500.0, 300.0)),
        default_theme: Theme::Light,
        ..Default::default()
    };
    eframe::run_native(
        "Talon",
        native_options,
        Box::new(|_cc| Box::new(MainWindow::default())),
    )
    .unwrap();
}
