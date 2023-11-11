mod tg_handler;
mod ui_components;
mod utils;

use crate::ui_components::MainWindow;
use dotenvy::dotenv;
use eframe::egui;
use eframe::Theme;
use egui::vec2;
use log::{info, LevelFilter};
use pretty_env_logger;
use std::env;

fn main() {
    dotenv().ok();
    let mut builder = pretty_env_logger::formatted_timed_builder();

    // Prevent logs from all crates
    builder
        .format_timestamp_millis()
        .filter(None, LevelFilter::Off);

    // If RUST_LOG present, set debug else info log for this crate only
    if env::var("RUST_LOG").is_ok() {
        builder
            .filter_module(env!("CARGO_PKG_NAME"), LevelFilter::Debug)
            .init()
    } else {
        builder
            .filter_module(env!("CARGO_PKG_NAME"), LevelFilter::Info)
            .init()
    };

    info!("Starting app");
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(500.0, 300.0)),
        min_window_size: Some(vec2(500.0, 300.0)),
        max_window_size: Some(vec2(600.0, 350.0)),
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
