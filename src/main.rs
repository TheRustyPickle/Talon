mod tg_handler;
mod ui_components;
mod utils;

use crate::ui_components::MainWindow;
use dirs::data_local_dir;
use eframe::{egui, NativeOptions};
use egui::{vec2, ViewportBuilder};
use env::set_current_dir;
use log::{info, LevelFilter};
use std::env;
use std::fs;

fn main() {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    builder.format_timestamp_millis();

    // If RUST_LOG present, set debug else info log for this crate only
    if env::var("RUST_LOG").is_ok() {
        builder
            .filter_module(env!("CARGO_BIN_NAME"), LevelFilter::Debug)
            .init();
    } else {
        builder
            .filter_module(env!("CARGO_BIN_NAME"), LevelFilter::Info)
            .init();
    }

    let working_path = data_local_dir();

    if let Some(location) = working_path {
        let mut target_location = location;

        target_location.push("Talon");

        fs::create_dir_all(&target_location).unwrap();
        set_current_dir(target_location).unwrap();

        info!("Starting app");
        let viewport = ViewportBuilder::default()
            .with_title("Talon")
            .with_inner_size(vec2(600.0, 450.0))
            .with_resizable(true)
            .with_maximize_button(false);
        let native_options = NativeOptions {
            viewport,
            ..Default::default()
        };
        eframe::run_native(
            "Talon",
            native_options,
            Box::new(|cc| Ok(Box::new(MainWindow::new(cc)))),
        )
        .unwrap();
    } else {
        println!("Failed to get local data directory. Exiting");
        std::process::exit(1);
    }
}
