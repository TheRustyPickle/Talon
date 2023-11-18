mod tg_handler;
mod ui_components;
mod utils;

use crate::ui_components::MainWindow;
use dotenvy::dotenv;
use eframe::egui;
use eframe::Theme;
use egui::vec2;
use log::{info, LevelFilter};
use std::env;

fn main() {
    dotenv().ok();
    let mut builder = pretty_env_logger::formatted_timed_builder();

    builder.format_timestamp_millis();

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
        initial_window_size: Some(vec2(500.0, 350.0)),
        default_theme: Theme::Light,
        resizable: true,
        ..Default::default()
    };
    eframe::run_native(
        "Talon",
        native_options,
        Box::new(|_cc| Box::<MainWindow>::default()),
    )
    .unwrap();
}
