use eframe::egui::{Align, Button, Grid, Layout, Ui, RichText};
use eframe::epaint::Color32;
use egui_dropdown::DropDownBox;
use std::collections::HashMap;

use crate::ui_components::processor::{ChartTiming, ChartType};
use crate::ui_components::MainWindow;
use crate::utils::get_next_color;

#[derive(Default)]
pub struct ChartsData {
    available_users: Vec<String>,
    dropdown_user: String,
    chart_type: ChartType,
    chart_timing: ChartTiming,
    added_to_chart: Vec<String>,
    button_sizes: HashMap<String, Option<f32>>,
    button_colors: HashMap<String, Color32>,
    last_color: Option<Color32>,
}

impl ChartsData {
    fn add_to_chart(&mut self) {
        let target_position = self
            .available_users
            .iter()
            .position(|a| a == &self.dropdown_user)
            .unwrap();
        let to_add = self.dropdown_user.to_owned();
        self.added_to_chart.push(self.dropdown_user.to_owned());
        self.dropdown_user.clear();
        self.available_users.remove(target_position);
        self.button_sizes.insert(to_add.to_owned(), None);

        if let Some(color) = self.last_color {
            let new_color = get_next_color(&color);
            self.last_color = Some(new_color);
            self.button_colors.insert(to_add, new_color);
        } else {
            self.button_colors.insert(to_add, Color32::BLACK);
            self.last_color = Some(Color32::BLACK);
        }
    }
}

impl MainWindow {
    pub fn show_charts_ui(&mut self, ui: &mut Ui) {
        self.charts_data.available_users = vec![
            "User 1", "User 2", "User 3", "User 4", "User 5", "User 6", "User 7", "User 8",
            "User 9", "User 10", "User 11", "User 12", "User 13", "User 14", "User 15", "User 16",
            "User 17", "User 18", "User 19", "User 20", "User 21", "User 22", "User 23", "User 24",
            "User 25", "User 26", "User 27", "User 28", "User 29", "User 30"
        ]
        .iter()
        .map(|a| a.to_string())
        .collect();
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.charts_data.chart_type,
                ChartType::Message,
                "Message",
            );
            ui.separator();
            ui.selectable_value(
                &mut self.charts_data.chart_type,
                ChartType::MessageWeekDay,
                "Message Weekday",
            );
            ui.separator();
            ui.selectable_value(
                &mut self.charts_data.chart_type,
                ChartType::ActiveUser,
                "Active User",
            );
        });
        if self.charts_data.chart_type != ChartType::MessageWeekDay {
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Hourly,
                    "Hourly",
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Daily,
                    "Daily",
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Weekly,
                    "Weekly",
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Monthly,
                    "Monthly",
                );
            });
            ui.separator();
        } else {
            ui.separator();
        }
        Grid::new("Chart Grid")
            .num_columns(1)
            .spacing([5.0, 10.0])
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let add_button = Button::new("Add to chart");

                    if self.charts_data.dropdown_user.is_empty()
                        || !self
                            .charts_data
                            .available_users
                            .contains(&self.charts_data.dropdown_user)
                    {
                        ui.add_enabled(false, add_button);
                    } else if ui.add(add_button).clicked() {
                        self.charts_data.add_to_chart();
                    };

                    ui.add_sized(
                        ui.available_size(),
                        DropDownBox::from_iter(
                            &self.charts_data.available_users,
                            "DropDown",
                            &mut self.charts_data.dropdown_user,
                            |ui, text| ui.selectable_label(false, text),
                        ),
                    );
                });
            });

        if !self.charts_data.added_to_chart.is_empty() {
            ui.separator();

            ui.vertical(|ui| {
                let mut to_add: Vec<String> = Vec::new();
                let mut already_added = 0.0;
                let max_size = ui.available_width();
                for (index, user) in self.charts_data.added_to_chart.iter().enumerate() {
                    // Check if the button size is saved previously or try to estimate a size
                    let button_size =
                        if let Some(size) = self.charts_data.button_sizes.get(user).unwrap() {
                            size.to_owned()
                        } else {
                            (user.len() as f32 * (ui.style().spacing.button_padding.x * 2.0))
                                + ui.spacing().item_spacing.x
                        };
                    already_added += button_size;
                    to_add.push(user.to_string());

                    // When total size is above the max width, place the buttons in the horizontal layout
                    if already_added >= max_size {
                        ui.horizontal(|ui| {
                            // If max size is 500, after the latest addition it became 550, the last button should not be in this layout
                            // Pop it and use it in the next horizontal layout
                            let last_value = to_add.pop().unwrap();
                            for button in &to_add {
                                let button_color =
                                    self.charts_data.button_colors.get(button).unwrap();
                                let mut text_data = RichText::new(button);

                                if ui.visuals().dark_mode {
                                    text_data = text_data.color(button_color.to_owned())
                                } else {
                                    text_data = text_data.background_color(button_color.to_owned());
                                }

                                let resp = ui.selectable_label(false, text_data);
                                *self.charts_data.button_sizes.get_mut(button).unwrap() =
                                    Some(resp.rect.width() + ui.spacing().item_spacing.x);
                            }
                            to_add.clear();
                            to_add.push(last_value);
                            // The size of the last value will be used for the next horizontal layout
                            already_added = button_size;
                        });
                    }

                    // If any pending button remains, add them
                    if index == self.charts_data.added_to_chart.len() - 1 {
                        ui.horizontal(|ui| {
                            for button in &to_add {
                                let button_color =
                                    self.charts_data.button_colors.get(button).unwrap();
                                let mut text_data = RichText::new(button);

                                if ui.visuals().dark_mode {
                                    text_data = text_data.color(button_color.to_owned())
                                } else {
                                    text_data = text_data.background_color(button_color.to_owned());
                                }

                                let resp = ui.selectable_label(false, text_data);
                                *self.charts_data.button_sizes.get_mut(button).unwrap() =
                                    Some(resp.rect.width() + ui.spacing().item_spacing.x);
                            }
                        });
                    }
                }
            });
            ui.separator();
        }
    }
}
