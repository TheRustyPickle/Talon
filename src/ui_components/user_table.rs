use eframe::egui::{Align, Key, Layout, Response, ScrollArea, SelectableLabel, Sense, Ui};
use egui_extras::{Column, TableBuilder};
use grammers_client::types::{Chat, Message};
use std::collections::{HashMap, HashSet};

use crate::ui_components::{ColumnName, MainWindow, ProcessState, SortOrder};

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

    fn get_column_length(&self, column: &ColumnName) -> usize {
        match column {
            ColumnName::Name => self.name.len(),
            ColumnName::Username => self.username.as_ref().map_or(5, |u| u.len()),
            ColumnName::UserID => self.id.to_string().len(),
            ColumnName::TotalMessage => self.total_message.to_string().len(),
            ColumnName::TotalWord => self.total_word.to_string().len(),
            ColumnName::TotalChar => self.total_char.to_string().len(),
            ColumnName::AverageWord => self.average_word.to_string().len(),
            ColumnName::AverageChar => self.average_char.to_string().len(),
            ColumnName::Whitelisted => self.whitelisted.to_string().len(),
        }
    }

    fn get_column_text(&self, column: &ColumnName) -> String {
        match column {
            ColumnName::Name => self.name.to_string(),
            ColumnName::Username => self
                .username
                .as_ref()
                .map_or("Empty".to_string(), |u| u.to_string()),
            ColumnName::UserID => self.id.to_string().to_string(),
            ColumnName::TotalMessage => self.total_message.to_string().to_string(),
            ColumnName::TotalWord => self.total_word.to_string().to_string(),
            ColumnName::TotalChar => self.total_char.to_string().to_string(),
            ColumnName::AverageWord => self.average_word.to_string().to_string(),
            ColumnName::AverageChar => self.average_char.to_string().to_string(),
            ColumnName::Whitelisted => self.whitelisted.to_string().to_string(),
        }
    }
}

#[derive(Default)]
pub struct UserTableData {
    rows: HashMap<i64, UserRowData>,
    sorted_by: ColumnName,
    sort_order: SortOrder,
    drag_started_on: Option<(i64, ColumnName)>,
    active_columns: HashSet<ColumnName>,
    last_active_row: Option<i64>,
    last_active_column: Option<ColumnName>,
    // To track whether the mouse pointer went beyond the drag point at least once
    beyond_drag_point: bool,
}

impl UserTableData {
    pub fn clear_row_data(&mut self) {
        self.rows.clear();
        self.sorted_by = ColumnName::default();
        self.sort_order = SortOrder::default();
        self.drag_started_on = None;
        self.active_columns.clear();
        self.last_active_row = None;
        self.last_active_column = None;
        self.beyond_drag_point = false;
    }

    pub fn add_user(&mut self, sender: Option<Chat>) -> Option<i64> {
        let mut to_return = None;
        if let Some(chat_data) = sender {
            let user_id = chat_data.id();
            to_return = Some(user_id);

            if let Chat::User(user) = chat_data {
                let full_name = user.full_name();

                let chat_name = if full_name.is_empty() {
                    "Deleted Account"
                } else {
                    &full_name
                };

                let user_name = user.username();

                self.rows
                    .entry(user_id)
                    .or_insert_with(|| UserRowData::new(chat_name, user_name, user_id, false));
            } else {
                let chat_name = chat_data.name();

                let chat_name = if chat_name.is_empty() {
                    "Deleted Account"
                } else {
                    chat_name
                };

                self.rows.entry(user_id).or_insert_with(|| {
                    UserRowData::new(chat_name, chat_data.username(), user_id, false)
                });
            }
        } else {
            self.rows
                .entry(0)
                .or_insert_with(|| UserRowData::new("Anonymous/Unknown", None, 0, false));
        }

        to_return
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
        let mut row_data: Vec<UserRowData> = self.rows.values().cloned().collect();

        match self.sorted_by {
            ColumnName::UserID => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.id.cmp(&b.id)),
                SortOrder::Descending => row_data.sort_by(|a, b| a.id.cmp(&b.id).reverse()),
            },
            ColumnName::Name => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.name.cmp(&b.name)),
                SortOrder::Descending => row_data.sort_by(|a, b| a.name.cmp(&b.name).reverse()),
            },
            ColumnName::Username => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.username.cmp(&b.username)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.username.cmp(&b.username).reverse())
                }
            },
            ColumnName::TotalMessage => match self.sort_order {
                SortOrder::Ascending => {
                    row_data.sort_by(|a, b| a.total_message.cmp(&b.total_message))
                }
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.total_message.cmp(&b.total_message).reverse())
                }
            },
            ColumnName::TotalWord => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.total_word.cmp(&b.total_word)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.total_word.cmp(&b.total_word).reverse())
                }
            },
            ColumnName::TotalChar => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.total_char.cmp(&b.total_char)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.total_char.cmp(&b.total_char).reverse())
                }
            },
            ColumnName::AverageChar => match self.sort_order {
                SortOrder::Ascending => {
                    row_data.sort_by(|a, b| a.average_char.cmp(&b.average_char))
                }
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.average_char.cmp(&b.average_char).reverse())
                }
            },
            ColumnName::AverageWord => match self.sort_order {
                SortOrder::Ascending => {
                    row_data.sort_by(|a, b| a.average_word.cmp(&b.average_word))
                }
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.average_word.cmp(&b.average_word).reverse())
                }
            },
            ColumnName::Whitelisted => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted).reverse())
                }
            },
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
        self.beyond_drag_point = true;

        // Ensure the starting drag cell is selected
        let drag_start = self.drag_started_on.clone().unwrap();
        self.select_single_row_cell(drag_start.0, &drag_start.1);

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

            let drag_start_index = all_rows
                .iter()
                .position(|row| row.id == drag_start.0)
                .unwrap();

            self.check_row_selection(true, &all_rows, current_row_index, drag_start_index);
            self.check_row_selection(false, &all_rows, current_row_index, drag_start_index);
        }
    }

    /// Recursively check the rows by either increasing or decreasing the initial index
    /// till the end point or an unselected row is found. Add all columns that are selected by at least one row as selected
    /// for the rows that have at least one column selected.
    fn check_row_selection(
        &mut self,
        check_previous: bool,
        rows: &Vec<UserRowData>,
        index: usize,
        drag_start: usize,
    ) {
        if index == 0 && check_previous {
            return;
        }

        if index + 1 == rows.len() && !check_previous {
            return;
        }

        let index = if check_previous { index - 1 } else { index + 1 };

        let current_row = rows.get(index).unwrap();
        let mut unselected_row = current_row.selected_columns.is_empty();

        // if for example drag started on row 5 and ended on row 10 but missed drag on row 7
        // Mark the rows as selected till the drag start row is hit (if recursively going that way)
        if (check_previous && index >= drag_start) || (!check_previous && index <= drag_start) {
            unselected_row = false
        }

        let target_row = self.rows.get_mut(&current_row.id).unwrap();

        if !unselected_row {
            for column in self.active_columns.iter() {
                target_row.selected_columns.insert(column.to_owned());
            }
            if check_previous {
                if index != 0 {
                    self.check_row_selection(check_previous, rows, index, drag_start);
                }
            } else if index + 1 != rows.len() {
                self.check_row_selection(check_previous, rows, index, drag_start);
            }
        }
    }

    /// Recursively check the given rows by either increasing or decreasing the initial index
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

    fn select_all(&mut self) {
        let mut all_columns = vec![ColumnName::Name];
        let mut current_column = ColumnName::Name.get_next();

        while current_column != ColumnName::Name {
            all_columns.push(current_column.clone());
            current_column = current_column.get_next()
        }

        for (_, row) in self.rows.iter_mut() {
            row.selected_columns.extend(all_columns.clone());
        }
        self.active_columns.extend(all_columns);
        self.last_active_row = None;
        self.last_active_column = None;
    }

    fn change_sorted_by(&mut self, sort_by: ColumnName) {
        self.unselected_all();
        self.sorted_by = sort_by;
        self.sort_order = SortOrder::default()
    }

    fn change_sort_order(&mut self) {
        if let SortOrder::Ascending = self.sort_order {
            self.sort_order = SortOrder::Descending;
        } else {
            self.sort_order = SortOrder::Ascending
        }
    }
}

impl MainWindow {
    pub fn show_user_table_ui(&mut self, ui: &mut Ui) {
        let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
        let key_a_pressed = ui.ctx().input(|i| i.key_pressed(Key::A));
        let key_c_pressed = ui.ctx().input(|i| i.key_pressed(Key::C));

        if is_ctrl_pressed && key_c_pressed {
            self.copy_selected_cells(ui);
        } else if is_ctrl_pressed && key_a_pressed {
            self.user_table.select_all();
        }

        ScrollArea::horizontal().drag_to_scroll(false).show(ui, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::initial(100.0).clip(true))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .column(Column::initial(100.0))
                .drag_to_scroll(false)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0);

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::Name;
                        let label_text = self.get_header_text(ColumnName::Name);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Telegram name of the user. Click to sort by name"); 

                        self.handle_header_selection(response, is_selected, ColumnName::Name);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::Username;
                        let label_text = self.get_header_text(ColumnName::Username);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Telegram username of the user. Click to sort by username");

                        self.handle_header_selection(response, is_selected, ColumnName::Username);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::UserID;
                        let label_text = self.get_header_text(ColumnName::UserID);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Telegram User ID of the user. Click to sort by user ID");

                        self.handle_header_selection(response, is_selected, ColumnName::UserID);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::TotalMessage;
                        let label_text = self.get_header_text(ColumnName::TotalMessage);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Total messages sent by the user. Click to sort by total message");

                        self.handle_header_selection(response, is_selected, ColumnName::TotalMessage);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::TotalWord;
                        let label_text = self.get_header_text(ColumnName::TotalWord);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Total words in the messages. Click to sort by total words");

                        self.handle_header_selection(response, is_selected, ColumnName::TotalWord);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::TotalChar;
                        let label_text = self.get_header_text(ColumnName::TotalChar);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Total character in the messages. Click to sort by total character"); 

                        self.handle_header_selection(response, is_selected, ColumnName::TotalChar);

                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::AverageWord;
                        let label_text = self.get_header_text(ColumnName::AverageWord);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Average number of words per message. Click to sort by average words");

                        self.handle_header_selection(response, is_selected, ColumnName::AverageWord);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::AverageChar;
                        let label_text = self.get_header_text(ColumnName::AverageChar);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Average number of characters per message. Click to sort by average characters");

                        self.handle_header_selection(response, is_selected, ColumnName::AverageChar);
                    });
                    header.col(|ui| {
                        let is_selected = self.user_table.sorted_by == ColumnName::Whitelisted;
                        let label_text = self.get_header_text(ColumnName::Whitelisted);
                        let response = ui.add_sized(
                            ui.available_size(),
                            SelectableLabel::new(is_selected, label_text),
                        )
                        .on_hover_text("Whether this user is whitelisted. Click to sort by whitelist");

                        self.handle_header_selection(response, is_selected, ColumnName::Whitelisted);
                    });
                })
                .body(|body| {
                    let table_rows = self.user_table.rows();
                    body.rows(25.0, table_rows.len(), |row_index, mut row| {
                        //for row_data in self.user_table.rows() {
                            let row_data = &table_rows[row_index];
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Name, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Username, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::UserID, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalMessage, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalWord, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalChar, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::AverageWord, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::AverageChar, row_data, ui)
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Whitelisted, row_data, ui)
                            });
                        //}
                    })

                    /*for row_data in self.user_table.rows() {
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
                    }*/
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
            self.user_table.beyond_drag_point = false;
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
                // Only call drag either when not on the starting drag row/column or went beyond the
                // drag point at least once. Otherwise normal click would be considered as drag
                if drag_start.0 != row_data.id
                    || drag_start.1 != column_name
                    || self.user_table.beyond_drag_point
                {
                    self.user_table
                        .select_dragged_row_cell(row_data.id, &column_name);
                }
            }
        }
    }

    pub fn handle_header_selection(
        &mut self,
        response: Response,
        is_selected: bool,
        sort_type: ColumnName,
    ) {
        if response.clicked() {
            if is_selected {
                self.user_table.change_sort_order()
            } else {
                self.user_table.change_sorted_by(sort_type)
            }
        }
    }

    pub fn get_header_text(&mut self, header_type: ColumnName) -> String {
        let mut text = match header_type {
            ColumnName::Name => "Name".to_string(),
            ColumnName::Username => "Username".to_string(),
            ColumnName::UserID => "User ID".to_string(),
            ColumnName::TotalMessage => "Total Messages".to_string(),
            ColumnName::TotalWord => "Total Word".to_string(),
            ColumnName::TotalChar => "Total Char".to_string(),
            ColumnName::AverageWord => "Average Word".to_string(),
            ColumnName::AverageChar => "Average Char".to_string(),
            ColumnName::Whitelisted => "Whitelisted".to_string(),
        };

        if header_type == self.user_table.sorted_by {
            match self.user_table.sort_order {
                SortOrder::Ascending => text.push('🔽'),
                SortOrder::Descending => text.push('🔼'),
            };
        }
        text
    }

    fn copy_selected_cells(&mut self, ui: &mut Ui) {
        let all_rows = self.user_table.rows();
        let mut selected_rows = Vec::new();

        let mut column_max_length = HashMap::new();

        for row in all_rows.into_iter() {
            if !row.selected_columns.is_empty() {
                for column in self.user_table.active_columns.iter() {
                    if row.selected_columns.contains(column) {
                        let field_length = row.get_column_length(column);
                        let entry = column_max_length.entry(column).or_insert(0);
                        if field_length > *entry {
                            column_max_length.insert(column, field_length);
                        }
                    }
                }
                selected_rows.push(row);
            }
        }

        let mut to_copy = String::new();
        let mut total_cells = 0;
        for row in selected_rows.into_iter() {
            let mut ongoing_column = ColumnName::Name;
            let mut row_text = String::new();
            loop {
                if self.user_table.active_columns.contains(&ongoing_column)
                    && row.selected_columns.contains(&ongoing_column)
                {
                    total_cells += 1;
                    let column_text = row.get_column_text(&ongoing_column);
                    row_text += &format!(
                        "{:<width$}",
                        column_text,
                        width = column_max_length[&ongoing_column] + 1
                    );
                } else if self.user_table.active_columns.contains(&ongoing_column)
                    && !row.selected_columns.contains(&ongoing_column)
                {
                    row_text += &format!(
                        "{:<width$}",
                        "",
                        width = column_max_length[&ongoing_column] + 1
                    );
                }

                if let ColumnName::Whitelisted = ongoing_column {
                    break;
                }
                ongoing_column = ongoing_column.get_next();
            }
            to_copy.push_str(&row_text);
            to_copy.push('\n');
        }

        ui.ctx().output_mut(|i| i.copied_text = to_copy);
        self.process_state = ProcessState::DataCopied(total_cells);
    }
}
