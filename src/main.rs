use eframe::{egui, Frame};
use eframe::{App, Theme};
use egui::{
    vec2, Align, Button, CentralPanel, Context, Grid, Label, Layout, TextEdit, Ui, ProgressBar, Rounding
};

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(400.0, 280.0)),
        min_window_size: Some(vec2(400.0, 280.0)),
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

struct MainWindow {
    start_from: String,
    end_at: String,
    current_at: TabState,
    num: String,
}

#[derive(PartialEq)]
enum TabState {
    Counter,
    Session,
    UserTable,
    Chart,
    Whitelist,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            start_from: String::new(),
            end_at: String::new(),
            current_at: TabState::Counter,
            num: String::new()
        }
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_at, TabState::Counter, "Counter");
                ui.selectable_value(&mut self.current_at, TabState::UserTable, "User Table");
                ui.selectable_value(&mut self.current_at, TabState::Chart, "Charts");
                ui.selectable_value(&mut self.current_at, TabState::Whitelist, "Whitelist");
                ui.selectable_value(&mut self.current_at, TabState::Session, "Sessions");
            });
            ui.separator();
            Grid::new("my grid")
                .num_columns(2)
                .spacing([5.0, 10.0])
                .show(ui, |ui| self.show_content(ui));

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                Grid::new("status").num_columns(2).spacing([5.0, 10.0]).show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("0");
                        ui.label("Messages Checked:");
                    });
                    
                    ui.end_row();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("0");
                        ui.label("Whitelisted Messages:");
                    });
                    ui.end_row();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("0");
                        ui.label("Users Found:");
                    });
                    ui.end_row();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("0");
                        ui.label("Whitelisted Users:");
                    });
                    ui.end_row();
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(50.0);
                    ui.add_sized([80.0,40.0], Button::new("Start"));
                });
                
                
            });
            
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                
                ui.label("Status: Idle");
                ui.separator();
                ui.add_space(5.0);
                ui.add(ProgressBar::new(0.4).animate(true).show_percentage());
            });
            


        });
    }
}

impl MainWindow {
    fn show_content(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Starting Point:"));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Button::new("Clear"));
            ui.add(Button::new("Paste"));
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.start_from)
                    .hint_text("Start counting from this message"),
            );
        });

        ui.end_row();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Ending Point:"));
        });
        
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Button::new("Clear"));
            ui.add(Button::new("Paste"));
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.end_at).hint_text("End counting at this message"),
            );
        });
        ui.end_row();
    }

    fn show_status(&mut self, ui: &mut Ui) {
        
        ui.end_row();

        
        ui.end_row();

    }
}
