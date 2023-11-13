use eframe::egui::{Align, Layout, ScrollArea, SelectableLabel, Ui};
use egui_extras::{Column, TableBuilder};
use grammers_client::types::{Chat, Message};
use std::collections::HashMap;

use crate::ui_components::{MainWindow, SortBy};

#[derive(Clone)]
pub struct UserRowData {
    name: String,
    username: Option<String>,
    id: i64,
    total_message: u32,
    total_word: u32,
    total_char: u32,
    average_word: u32,
    average_char: u32,
    whitelisted: bool,
}

impl UserRowData {
    fn new(name: &str, username: Option<&str>, id: i64, whitelisted: bool) -> Self {
        let username = username.map(|s| s.to_owned());
        UserRowData {
            name: name.to_string(),
            username: username,
            id,
            total_message: 0,
            total_word: 0,
            total_char: 0,
            average_word: 0,
            average_char: 0,
            whitelisted,
        }
    }

    fn increment_total_message(&mut self) {
        self.total_message += 1;
    }

    fn increment_total_word(&mut self, word_num: u32) {
        self.total_word += word_num;
        self.average_word = self.total_word / self.total_message;
    }

    fn increment_total_char(&mut self, char_num: u32) {
        self.total_char += char_num;
        self.average_char = self.total_char / self.total_message;
    }
}

#[derive(Default)]
pub struct UserTableData {
    rows: HashMap<i64, UserRowData>,
    sorted_by: SortBy,
}

impl UserTableData {
    pub fn add_user(&mut self, user_data: Chat) {
        let user_id = user_data.id();
        if !self.rows.contains_key(&user_id) {
            let row_data = UserRowData::new(user_data.name(), user_data.username(), user_id, false);
            self.rows.insert(user_id, row_data);
        }
    }

    pub fn add_unknown_user(&mut self) {
        if !self.rows.contains_key(&0) {
            let row_data = UserRowData::new("Anonymous/Unknown", None, 0, false);
            self.rows.insert(0, row_data);
        }
    }

    pub fn count_user_message(&mut self, user_id: Option<i64>, message: &Message) {
        let user_id = if let Some(num) = user_id { num } else { 0 };

        let user_row_data = self.rows.get_mut(&user_id).unwrap();

        let message_text = message.text();

        let total_char = message_text.len() as u32;
        let total_word = message_text.split_whitespace().count() as u32;

        user_row_data.increment_total_message();
        user_row_data.increment_total_word(total_word);
        user_row_data.increment_total_char(total_char);
    }

    pub fn get_total_user(&self) -> i32 {
        self.rows.len() as i32
    }

    pub fn rows(&self) -> Vec<UserRowData> {
        let mut row_data: Vec<UserRowData> = self.rows.iter().map(|(_, v)| v.to_owned()).collect();
        match self.sorted_by {
            SortBy::SortByID => {
                row_data.sort_by(|a, b| a.id.cmp(&b.id));
            }
            SortBy::SortByName => {
                row_data.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortBy::SortByUsername => {
                row_data.sort_by(|a, b| a.username.cmp(&b.username));
            }
            SortBy::SortByMessageNum => {
                row_data.sort_by(|a, b| a.total_message.cmp(&b.total_message));
            }
            SortBy::SortByWordNum => {
                row_data.sort_by(|a, b| a.total_word.cmp(&b.total_word));
            }
            SortBy::SortByCharNum => {
                row_data.sort_by(|a, b| a.total_char.cmp(&b.total_char));
            }
            SortBy::SortByAverageChar => {
                row_data.sort_by(|a, b| a.average_char.cmp(&b.average_char));
            }
            SortBy::SortByAverageWord => {
                row_data.sort_by(|a, b| a.average_word.cmp(&b.average_word));
            }
            SortBy::SortByWhitelisted => {
                row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted));
            }
        }

        row_data
    }
}

impl MainWindow {
    pub fn show_user_table_ui(&mut self, ui: &mut Ui) {
        ScrollArea::horizontal().drag_to_scroll(false).show(ui, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .auto_shrink([true; 2])
                .resizable(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::initial(100.0).clip(true))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .column(Column::initial(80.0))
                .drag_to_scroll(false)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0);

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByName;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Name"),
                        )
                        .on_hover_text("Telegram name of the user. Click to sort by name").clicked() {
                            self.user_table.sorted_by = SortBy::SortByName;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByUsername;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Username"),
                        )
                        .on_hover_text("Telegram username of the user. Click to sort by username").clicked() {
                            self.user_table.sorted_by = SortBy::SortByUsername;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByID;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "User ID"),
                        )
                        .on_hover_text("Telegram User ID of the user. Click to sort by user ID").clicked() {
                            self.user_table.sorted_by = SortBy::SortByID;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByMessageNum;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Total Message"),
                        )
                        .on_hover_text("Total messages sent by the user. Click to sort by total message").clicked() {
                            self.user_table.sorted_by = SortBy::SortByMessageNum;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByWordNum;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Total Word"),
                        )
                        .on_hover_text("Total words in the messages. Click to sort by total words").clicked() {
                            self.user_table.sorted_by = SortBy::SortByWordNum;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByCharNum;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Total Char"),
                        )
                        .on_hover_text("Total character in the messages. Click to sort by total character").clicked() {
                            self.user_table.sorted_by = SortBy::SortByCharNum;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByAverageWord;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Average Word"),
                        )
                        .on_hover_text("Average number of words per message. Click to sort by average words").clicked() {
                            self.user_table.sorted_by = SortBy::SortByAverageWord;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByAverageChar;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Average Char"),
                        )
                        .on_hover_text("Average number of characters per message. Click to sort by average characters").clicked() {
                            self.user_table.sorted_by = SortBy::SortByAverageChar;
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::SortByWhitelisted;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Whitelisted"),
                        )
                        .on_hover_text("Whether this user is whitelisted. Click to sort by whitelist").clicked() {
                            self.user_table.sorted_by = SortBy::SortByWhitelisted;
                        };
                    });
                })
                .body(|mut body| {
                    for row_data in self.user_table.rows() {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&row_data.name).on_hover_text(row_data.name);
                            });
                            row.col(|ui| {
                                let username = if let Some(name) = &row_data.username {
                                    name.to_string()
                                } else {
                                    String::from("Empty")
                                };
                                ui.label(username);
                            });
                            row.col(|ui| {
                                ui.label(row_data.id.to_string());
                            });
                            row.col(|ui| {
                                ui.label(row_data.total_message.to_string());
                            });
                            row.col(|ui| {
                                ui.label(row_data.total_word.to_string());
                            });
                            row.col(|ui| {
                                ui.label(row_data.total_char.to_string());
                            });
                            row.col(|ui| {
                                ui.label(row_data.average_word.to_string());
                            });
                            row.col(|ui| {
                                ui.label(row_data.average_char.to_string());
                            });
                            row.col(|ui| {
                                let whitelist_text =
                                    if row_data.whitelisted { "Yes" } else { "No" };
                                ui.label(whitelist_text);
                            });
                        })
                    }
                });
        });
    }
}
