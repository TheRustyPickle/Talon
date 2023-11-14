use eframe::egui::{Align, Layout, ScrollArea, SelectableLabel, Sense, Ui};
use egui_extras::{Column, TableBuilder};
use grammers_client::types::{Chat, Message};
use std::collections::{HashMap, HashSet};

use crate::ui_components::{ColumnName, MainWindow, SortBy};

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
    selected_columns: HashSet<ColumnName>,
}

impl UserRowData {
    fn new(name: &str, username: Option<&str>, id: i64, whitelisted: bool) -> Self {
        let username = username.map(|s| s.to_owned());
        UserRowData {
            name: name.to_string(),
            username,
            id,
            total_message: 0,
            total_word: 0,
            total_char: 0,
            average_word: 0,
            average_char: 0,
            whitelisted,
            selected_columns: HashSet::new(),
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
    drag_started_on: Option<(i64, ColumnName)>,
    active_columns: HashSet<ColumnName>,
    last_active_row: Option<i64>,
    last_active_column: Option<ColumnName>,
}

impl UserTableData {
    pub fn add_user(&mut self, user_data: Chat) {
        let user_id = user_data.id();
        self.rows.entry(user_id).or_insert_with(|| {
            let row_data = UserRowData::new(user_data.name(), user_data.username(), user_id, false);
            row_data
        });
    }

    pub fn add_unknown_user(&mut self) {
        self.rows
            .entry(0)
            .or_insert_with(|| UserRowData::new("Anonymous/Unknown", None, 0, false));
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
        let mut row_data: Vec<UserRowData> = self.rows.values().map(|v| v.to_owned()).collect();
        match self.sorted_by {
            SortBy::ID => {
                row_data.sort_by(|a, b| a.id.cmp(&b.id));
            }
            SortBy::Name => {
                row_data.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortBy::Username => {
                row_data.sort_by(|a, b| a.username.cmp(&b.username));
            }
            SortBy::MessageNum => {
                row_data.sort_by(|a, b| a.total_message.cmp(&b.total_message));
            }
            SortBy::WordNum => {
                row_data.sort_by(|a, b| a.total_word.cmp(&b.total_word));
            }
            SortBy::CharNum => {
                row_data.sort_by(|a, b| a.total_char.cmp(&b.total_char));
            }
            SortBy::AverageChar => {
                row_data.sort_by(|a, b| a.average_char.cmp(&b.average_char));
            }
            SortBy::AverageWord => {
                row_data.sort_by(|a, b| a.average_word.cmp(&b.average_word));
            }
            SortBy::Whitelisted => {
                row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted));
            }
        }

        row_data
    }

    fn select_single_row_cell(&mut self, user_id: i64, column_name: &ColumnName) {
        self.active_columns.insert(column_name.clone());
        self.rows
            .get_mut(&user_id)
            .unwrap()
            .selected_columns
            .insert(column_name.clone());
    }

    fn select_dragged_row_cell(&mut self, user_id: i64, column_name: &ColumnName) {
        if self.last_active_row == Some(user_id)
            && self.last_active_column == Some(column_name.clone())
        {
            return;
        }

        self.active_columns.insert(column_name.clone());

        let min_col = self.active_columns.clone().into_iter().min();
        let max_col = self.active_columns.clone().into_iter().max();

        // If column 1 and column 5 is selected, ensure the column in the middle are not missing from selection
        // It can miss in case of fast mouse movement
        if let (Some(min), Some(max)) = (min_col, max_col) {
            let mut current_col = min;
            while current_col != max {
                let next_col = current_col.get_next();
                self.active_columns.insert(next_col.clone());
                current_col = next_col;
            }
        }

        // The rows in the current sorted format
        let all_rows = self.rows();

        // The row the mouse pointer is on
        let current_row = self.rows.get_mut(&user_id).unwrap();

        // If this row already selects the column that we are trying to select, it means the mouse
        // moved backwards from an active column to another active column.
        let row_contains_column = current_row.selected_columns.contains(column_name);

        // If we have some data of the last row and column that the mouse was on, then try to unselect
        if row_contains_column
            && self.last_active_row.is_some()
            && self.last_active_column.is_some()
        {
            let last_active_column = self.last_active_column.clone().unwrap();

            // Remove the last column selection from the current row where the mouse is if
            // the previous row and the current one matches
            if last_active_column != *column_name && self.last_active_row.unwrap() == user_id {
                current_row.selected_columns.remove(&last_active_column);
                self.active_columns.remove(&last_active_column);
            }

            // Get the last row where the mouse was
            let last_row = self.rows.get_mut(&self.last_active_row.unwrap()).unwrap();

            self.last_active_row = Some(user_id);

            // If on the same row as the last row, then unselect the column from all other select row
            if user_id == last_row.id {
                if last_active_column != *column_name {
                    self.last_active_column = Some(column_name.clone());

                    let current_row_index =
                        all_rows.iter().position(|row| row.id == user_id).unwrap();

                    self.remove_row_column_selection(
                        true,
                        &all_rows,
                        current_row_index,
                        &last_active_column,
                    );
                    self.remove_row_column_selection(
                        false,
                        &all_rows,
                        current_row_index,
                        &last_active_column,
                    );
                }
            } else {
                last_row.selected_columns.clear();
            }
        } else {
            self.last_active_row = Some(user_id);
            self.last_active_column = Some(column_name.clone());
            for column in self.active_columns.iter() {
                current_row.selected_columns.insert(column.to_owned());
            }
            let current_row_index = all_rows.iter().position(|row| row.id == user_id).unwrap();

            self.check_row_selection(true, &all_rows, current_row_index);
            self.check_row_selection(false, &all_rows, current_row_index);
        }
    }

    /// Recurve through the given rows by either increasing or decreasing the initial index
    /// till the end point or an unselected row is found. Add all columns that are selected by at least one row as selected
    /// for the rows that have at least one column selected.
    fn check_row_selection(&mut self, check_previous: bool, rows: &Vec<UserRowData>, index: usize) {
        if index == 0 && check_previous {
            return;
        }

        if index + 1 == rows.len() && !check_previous {
            return;
        }

        let index = if check_previous { index - 1 } else { index + 1 };

        let current_row = rows.get(index).unwrap();
        let unselected_row = current_row.selected_columns.is_empty();

        let target_row = self.rows.get_mut(&current_row.id).unwrap();

        if !unselected_row {
            for column in self.active_columns.iter() {
                target_row.selected_columns.insert(column.to_owned());
            }
            if check_previous {
                if index != 0 {
                    self.check_row_selection(check_previous, rows, index);
                }
            } else if index + 1 != rows.len() {
                self.check_row_selection(check_previous, rows, index);
            }
        }
    }

    /// Recurve through the given rows by either increasing or decreasing the initial index
    /// till the end point or an unselected row is found. Remove the given column from selection
    /// from all rows that are selected.
    fn remove_row_column_selection(
        &mut self,
        check_previous: bool,
        rows: &Vec<UserRowData>,
        index: usize,
        target_column: &ColumnName,
    ) {
        if index == 0 && check_previous {
            return;
        }

        if index + 1 == rows.len() && !check_previous {
            return;
        }

        let index = if check_previous { index - 1 } else { index + 1 };

        let current_row = rows.get(index).unwrap();
        let unselected_row = current_row.selected_columns.is_empty();

        let target_row = self.rows.get_mut(&current_row.id).unwrap();

        if !unselected_row {
            target_row.selected_columns.remove(target_column);

            if check_previous {
                if index != 0 {
                    self.remove_row_column_selection(check_previous, rows, index, target_column);
                }
            } else if index + 1 != rows.len() {
                self.remove_row_column_selection(check_previous, rows, index, target_column);
            }
        }
    }

    fn unselected_all(&mut self) {
        for (_, row) in self.rows.iter_mut() {
            row.selected_columns.clear()
        }
        self.active_columns.clear();
        self.last_active_row = None;
        self.last_active_column = None;
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
                        let is_selected = self.user_table.sorted_by == SortBy::Name;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Name"),
                        )
                        .on_hover_text("Telegram name of the user. Click to sort by name").clicked() {
                            self.user_table.sorted_by = SortBy::Name;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::Username;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Username"),
                        )
                        .on_hover_text("Telegram username of the user. Click to sort by username").clicked() {
                            self.user_table.sorted_by = SortBy::Username;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::ID;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "User ID"),
                        )
                        .on_hover_text("Telegram User ID of the user. Click to sort by user ID").clicked() {
                            self.user_table.sorted_by = SortBy::ID;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::MessageNum;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Total Message"),
                        )
                        .on_hover_text("Total messages sent by the user. Click to sort by total message").clicked() {
                            self.user_table.sorted_by = SortBy::MessageNum;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::WordNum;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Total Word"),
                        )
                        .on_hover_text("Total words in the messages. Click to sort by total words").clicked() {
                            self.user_table.sorted_by = SortBy::WordNum;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::CharNum;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Total Char"),
                        )
                        .on_hover_text("Total character in the messages. Click to sort by total character").clicked() {
                            self.user_table.sorted_by = SortBy::CharNum;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::AverageWord;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Average Word"),
                        )
                        .on_hover_text("Average number of words per message. Click to sort by average words").clicked() {
                            self.user_table.sorted_by = SortBy::AverageWord;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::AverageChar;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Average Char"),
                        )
                        .on_hover_text("Average number of characters per message. Click to sort by average characters").clicked() {
                            self.user_table.sorted_by = SortBy::AverageChar;
                            self.user_table.unselected_all();
                        };
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == SortBy::Whitelisted;
                        if ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, "Whitelisted"),
                        )
                        .on_hover_text("Whether this user is whitelisted. Click to sort by whitelist").clicked() {
                            self.user_table.sorted_by = SortBy::Whitelisted;
                            self.user_table.unselected_all();
                        };
                    });
                })
                .body(|mut body| {
                    for row_data in self.user_table.rows() {
                        body.row(25.0, |mut row| {
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Name, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Username, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::UserID, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalMessage, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalWord, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalChar, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::AverageWord, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::AverageChar, &row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Whitelisted, &row_data, ui)
                            });
                        })
                    }
                });
        });
    }

    pub fn create_table_row(
        &mut self,
        column_name: ColumnName,
        row_data: &UserRowData,
        ui: &mut Ui,
    ) {
        let mut show_tooltip = false;
        let row_text = match column_name {
            ColumnName::Name => {
                show_tooltip = true;
                row_data.name.to_owned()
            }
            ColumnName::Username => {
                if let Some(name) = &row_data.username {
                    name.to_string()
                } else {
                    "Empty".to_string()
                }
            }
            ColumnName::UserID => row_data.id.to_string(),
            ColumnName::TotalMessage => row_data.total_message.to_string(),
            ColumnName::TotalWord => row_data.total_word.to_string(),
            ColumnName::TotalChar => row_data.total_char.to_string(),
            ColumnName::AverageWord => row_data.average_word.to_string(),
            ColumnName::AverageChar => row_data.average_char.to_string(),
            ColumnName::Whitelisted => {
                let text = if row_data.whitelisted { "Yes" } else { "No" };
                text.to_string()
            }
        };

        let is_selected = row_data.selected_columns.contains(&column_name);

        let mut label = ui
            .add_sized(
                ui.available_size(),
                SelectableLabel::new(is_selected, &row_text),
            )
            .interact(Sense::drag());

        if show_tooltip {
            label = label.on_hover_text(row_text);
        }

        if label.drag_started() {
            if !ui.ctx().input(|i| i.modifiers.ctrl) {
                self.user_table.unselected_all();
            }
            self.user_table.drag_started_on = Some((row_data.id, column_name.clone()));
        }
        if label.drag_released() {
            self.user_table.last_active_row = None;
            self.user_table.last_active_column = None;
            self.user_table.drag_started_on = None;
        }

        if label.clicked() {
            if !ui.ctx().input(|i| i.modifiers.ctrl) {
                self.user_table.unselected_all();
            }
            self.user_table
                .select_single_row_cell(row_data.id, &column_name);
        }
        if ui.ui_contains_pointer() && self.user_table.drag_started_on.is_some() {
            if let Some(drag_start) = &self.user_table.drag_started_on {
                if drag_start.0 != row_data.id || drag_start.1 != column_name {
                    self.user_table
                        .select_dragged_row_cell(row_data.id, &column_name);
                } else {
                    self.user_table
                        .select_single_row_cell(row_data.id, &column_name);
                }
            }
        }
    }
}
