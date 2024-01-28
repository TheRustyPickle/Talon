use eframe::egui::{
    Align, Key, Layout, Response, RichText, ScrollArea, SelectableLabel, Sense, Ui,
};
use egui_extras::{Column, TableBuilder};
use grammers_client::types::{Chat, Message};
use std::collections::{HashMap, HashSet};

use crate::ui_components::processor::{ColumnName, ProcessState, SortOrder};
use crate::ui_components::widgets::RowLabel;
use crate::ui_components::MainWindow;
use crate::utils::save_whitelisted_users;

#[derive(Clone)]
struct UserRowData {
    name: String,
    username: String,
    id: i64,
    total_message: u32,
    total_word: u32,
    total_char: u32,
    average_word: u32,
    average_char: u32,
    whitelisted: bool,
    selected_columns: HashSet<ColumnName>,
    belongs_to: Option<Chat>,
}

impl UserRowData {
    fn new(
        name: &str,
        username: &str,
        id: i64,
        whitelisted: bool,
        belongs_to: Option<Chat>,
    ) -> Self {
        let username = username.to_string();

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
            belongs_to,
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

    /// Get the current length of a column of this row
    fn get_column_length(&self, column: &ColumnName) -> usize {
        match column {
            ColumnName::Name => self.name.len(),
            ColumnName::Username => self.username.len(),
            ColumnName::UserID => self.id.to_string().len(),
            ColumnName::TotalMessage => self.total_message.to_string().len(),
            ColumnName::TotalWord => self.total_word.to_string().len(),
            ColumnName::TotalChar => self.total_char.to_string().len(),
            ColumnName::AverageWord => self.average_word.to_string().len(),
            ColumnName::AverageChar => self.average_char.to_string().len(),
            ColumnName::Whitelisted => self.whitelisted.to_string().len(),
        }
    }

    /// Get the text of a column of this row
    fn get_column_text(&self, column: &ColumnName) -> String {
        match column {
            ColumnName::Name => self.name.to_string(),
            ColumnName::Username => self.username.to_string(),
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
    /// Clear all the rows
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

    /// Add a user to the table
    pub fn add_user(&mut self, sender: Option<Chat>) -> (i64, String, String) {
        let mut user_id = 0;
        let full_name;
        let user_name;

        if let Some(chat_data) = sender {
            user_id = chat_data.id();

            if let Chat::User(user) = chat_data.clone() {
                // As per grammers lib, empty name can be given if it's a deleted account
                full_name = if user.full_name().is_empty() {
                    "Deleted Account".to_string()
                } else {
                    user.full_name()
                };

                user_name = if let Some(name) = user.username() {
                    name.to_string()
                } else {
                    "Empty".to_string()
                };

                self.rows.entry(user_id).or_insert_with(|| {
                    UserRowData::new(&full_name, &user_name, user_id, false, Some(chat_data))
                });
            } else {
                full_name = if chat_data.name().is_empty() {
                    "Deleted Account".to_string()
                } else {
                    chat_data.name().to_string()
                };

                user_name = if let Some(name) = chat_data.username() {
                    name.to_string()
                } else {
                    "Empty".to_string()
                };

                self.rows.entry(user_id).or_insert_with(|| {
                    UserRowData::new(
                        &full_name,
                        &user_name,
                        user_id,
                        false,
                        Some(chat_data.clone()),
                    )
                });
            }
        } else {
            // If there is no Chat value then it could be an anonymous user
            full_name = "Anonymous/Unknown".to_string();
            user_name = "Empty".to_string();

            self.rows
                .entry(0)
                .or_insert_with(|| UserRowData::new(&full_name, &user_name, user_id, false, None));
        }

        (user_id, full_name, user_name)
    }

    /// Update message related column values of a row
    pub fn count_user_message(&mut self, user_id: i64, message: &Message) {
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

    /// Returns all existing row in the current sorted format in a vector
    fn rows(&self) -> Vec<UserRowData> {
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
                    row_data.sort_by(|a, b| a.username.cmp(&b.username).reverse());
                }
            },
            ColumnName::TotalMessage => match self.sort_order {
                SortOrder::Ascending => {
                    row_data.sort_by(|a, b| a.total_message.cmp(&b.total_message));
                }
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.total_message.cmp(&b.total_message).reverse());
                }
            },
            ColumnName::TotalWord => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.total_word.cmp(&b.total_word)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.total_word.cmp(&b.total_word).reverse());
                }
            },
            ColumnName::TotalChar => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.total_char.cmp(&b.total_char)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.total_char.cmp(&b.total_char).reverse());
                }
            },
            ColumnName::AverageChar => match self.sort_order {
                SortOrder::Ascending => {
                    row_data.sort_by(|a, b| a.average_char.cmp(&b.average_char));
                }
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.average_char.cmp(&b.average_char).reverse());
                }
            },
            ColumnName::AverageWord => match self.sort_order {
                SortOrder::Ascending => {
                    row_data.sort_by(|a, b| a.average_word.cmp(&b.average_word));
                }
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.average_word.cmp(&b.average_word).reverse());
                }
            },
            ColumnName::Whitelisted => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted).reverse());
                }
            },
        }

        row_data
    }

    /// Marks a single column of a row as selected
    fn select_single_row_cell(&mut self, user_id: i64, column_name: &ColumnName) {
        self.active_columns.insert(column_name.clone());
        self.rows
            .get_mut(&user_id)
            .unwrap()
            .selected_columns
            .insert(column_name.clone());
    }

    /// Continuously called to select rows and columns when dragging has started
    fn select_dragged_row_cell(&mut self, user_id: i64, column_name: &ColumnName) {
        // If both same then the mouse is still on the same column on the same row so nothing to process
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
            //
            // column column column
            // column column column
            // column column (mouse is currently here) column(mouse was here)
            //
            // We unselect the bottom right corner column
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

                    // Position where the mouse is
                    let current_row_index =
                        all_rows.iter().position(|row| row.id == user_id).unwrap();

                    // This solution is not perfect.
                    // In case of fast mouse movement it fails to call this function as if mouse was not over that cell

                    // If current position is row 5 column 2 then check row from 4 to 1 or 4 till a row with no active column is found
                    // Remove the last selected column from the selection from all the rows are that found
                    self.remove_row_column_selection(
                        true,
                        &all_rows,
                        current_row_index,
                        &last_active_column,
                    );
                    // If current position is row 5 column 2 then check row from 6 to final row or 6 till a row with no active column is found
                    // Remove the last selected column from the selection from all the rows are that found
                    self.remove_row_column_selection(
                        false,
                        &all_rows,
                        current_row_index,
                        &last_active_column,
                    );
                }
            } else {
                // Mouse went 1 row above or below. So just clear all selection from that previous row
                last_row.selected_columns.clear();
            }
        } else {
            // We are in a new row which we have not selected before
            self.last_active_row = Some(user_id);
            self.last_active_column = Some(column_name.clone());
            for column in &self.active_columns {
                current_row.selected_columns.insert(column.to_owned());
            }
            let current_row_index = all_rows.iter().position(|row| row.id == user_id).unwrap();

            // Get the row number where the drag started on
            let drag_start_index = all_rows
                .iter()
                .position(|row| row.id == drag_start.0)
                .unwrap();

            // If drag started on row 1, currently on row 5, check from row 4 to 1 and select all columns
            // else go through all rows till a row without any selected column is found. Applied both by incrementing or decrementing index.
            // In case of fast mouse movement following drag started point mitigates the risk of some rows not getting selected
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
            unselected_row = false;
        }

        let target_row = self.rows.get_mut(&current_row.id).unwrap();

        if !unselected_row {
            for column in &self.active_columns {
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

    /// Unselect all rows and columns
    fn unselected_all(&mut self) {
        for (_, row) in self.rows.iter_mut() {
            row.selected_columns.clear();
        }
        self.active_columns.clear();
        self.last_active_row = None;
        self.last_active_column = None;
    }

    /// Select all rows and columns
    fn select_all(&mut self) {
        let mut all_columns = vec![ColumnName::Name];
        let mut current_column = ColumnName::Name.get_next();

        while current_column != ColumnName::Name {
            all_columns.push(current_column.clone());
            current_column = current_column.get_next();
        }

        for (_, row) in self.rows.iter_mut() {
            row.selected_columns.extend(all_columns.clone());
        }
        self.active_columns.extend(all_columns);
        self.last_active_row = None;
        self.last_active_column = None;
    }

    /// Change the value it is currently sorted by. Called on header column click
    fn change_sorted_by(&mut self, sort_by: ColumnName) {
        self.unselected_all();
        self.sorted_by = sort_by;
        self.sort_order = SortOrder::default();
    }

    /// Change the order of row sorting. Called on header column click
    fn change_sort_order(&mut self) {
        if let SortOrder::Ascending = self.sort_order {
            self.sort_order = SortOrder::Descending;
        } else {
            self.sort_order = SortOrder::Ascending;
        }
    }

    /// Mark a row as whitelisted if exists
    pub fn set_as_whitelisted(&mut self, user_id: &i64) {
        if let Some(row) = self.rows.get_mut(user_id) {
            row.whitelisted = true;
        }
    }

    /// Remove whitelist status from a row if exists
    pub fn remove_whitelist(&mut self, user_id: &i64) {
        if let Some(row) = self.rows.get_mut(user_id) {
            row.whitelisted = false;
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

        ScrollArea::horizontal()
            .drag_to_scroll(false)
            .show(ui, |ui| {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::initial(100.0).clip(true))
                    .column(Column::initial(100.0).clip(true))
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
                            self.create_header(ColumnName::Name, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::Username, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::UserID, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::TotalMessage, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::TotalWord, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::TotalChar, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::AverageWord, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::AverageChar, ui);
                        });
                        header.col(|ui| {
                            self.create_header(ColumnName::Whitelisted, ui);
                        });
                    })
                    .body(|body| {
                        let table_rows = self.user_table.rows();
                        body.rows(25.0, table_rows.len(), |mut row| {
                            let row_data = &table_rows[row.index()];
                            row.col(|ui| self.create_table_row(ColumnName::Name, row_data, ui));
                            row.col(|ui| self.create_table_row(ColumnName::Username, row_data, ui));
                            row.col(|ui| self.create_table_row(ColumnName::UserID, row_data, ui));
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalMessage, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalWord, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::TotalChar, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::AverageWord, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::AverageChar, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_table_row(ColumnName::Whitelisted, row_data, ui);
                            });
                        });
                    });
            });
    }

    /// Create a table row from a column name and the row data
    fn create_table_row(&mut self, column_name: ColumnName, row_data: &UserRowData, ui: &mut Ui) {
        let mut show_tooltip = false;
        let row_text = match column_name {
            ColumnName::Name => {
                show_tooltip = true;
                row_data.name.clone()
            }
            ColumnName::Username => {
                show_tooltip = true;
                row_data.username.clone()
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
        let is_whitelisted = row_data.whitelisted;

        // On normal click both drag and click are returned for handling
        // A single click can return 2-3 result, 1 click and the rest are drag
        let mut label = ui
            .add_sized(
                ui.available_size(),
                RowLabel::new(is_selected, is_whitelisted, &row_text),
            )
            .interact(Sense::drag())
            .context_menu(|ui| {
                if ui.button("Copy selected rows").clicked() {
                    self.copy_selected_cells(ui);
                    ui.close_menu();
                };
                if ui.button("whitelist selected rows").clicked() {
                    self.whitelist_selected_rows();
                    ui.close_menu();
                };
            });

        if show_tooltip {
            label = label.on_hover_text(row_text);
        }

        if label.drag_started() {
            // If CTRL is not pressed down and the mouse right click is not pressed, unselect all cells
            if !ui.ctx().input(|i| i.modifiers.ctrl)
                && !ui.ctx().input(|i| i.pointer.secondary_clicked())
            {
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
            // If CTRL is not pressed down and the mouse right click is not pressed, unselect all cells
            if !ui.ctx().input(|i| i.modifiers.ctrl)
                && !ui.ctx().input(|i| i.pointer.secondary_clicked())
            {
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

    /// Create a header column
    fn create_header(&mut self, column_name: ColumnName, ui: &mut Ui) {
        let is_selected = self.user_table.sorted_by == column_name;
        let (label_text, hover_text) = self.get_header_text(&column_name);

        let response = ui
            .add_sized(
                ui.available_size(),
                SelectableLabel::new(is_selected, label_text),
            )
            .on_hover_text(hover_text);

        self.handle_header_selection(response, is_selected, column_name);
    }

    /// Handles sort order and value on header click
    fn handle_header_selection(
        &mut self,
        response: Response,
        is_selected: bool,
        sort_type: ColumnName,
    ) {
        if response.clicked() {
            if is_selected {
                self.user_table.change_sort_order();
            } else {
                self.user_table.change_sorted_by(sort_type);
            }
        }
    }

    fn get_header_text(&mut self, header_type: &ColumnName) -> (RichText, String) {
        let (mut text, hover_text) = match header_type {
            ColumnName::Name => (
                "Name".to_string(),
                "Telegram name of the user. Click to sort by name".to_string(),
            ),
            ColumnName::Username => (
                "Username".to_string(),
                "Telegram username of the user. Click to sort by username".to_string(),
            ),
            ColumnName::UserID => (
                "User ID".to_string(),
                "Telegram User ID of the user. Click to sort by user ID".to_string(),
            ),
            ColumnName::TotalMessage => (
                "Total Messages".to_string(),
                "Total messages sent by the user. Click to sort by total message".to_string(),
            ),
            ColumnName::TotalWord => (
                "Total Word".to_string(),
                "Total words in the messages. Click to sort by total words".to_string(),
            ),
            ColumnName::TotalChar => (
                "Total Char".to_string(),
                "Total character in the messages. Click to sort by total character".to_string(),
            ),
            ColumnName::AverageWord => (
                "Average Word".to_string(),
                "Average number of words per message. Click to sort by average words".to_string(),
            ),
            ColumnName::AverageChar => (
                "Average Char".to_string(),
                "Average number of characters per message. Click to sort by average characters"
                    .to_string(),
            ),
            ColumnName::Whitelisted => (
                "Whitelisted".to_string(),
                "Whether this user is whitelisted. Click to sort by whitelist".to_string(),
            ),
        };

        if header_type == &self.user_table.sorted_by {
            match self.user_table.sort_order {
                SortOrder::Ascending => text.push('🔽'),
                SortOrder::Descending => text.push('🔼'),
            };
        }
        (RichText::new(text).strong(), hover_text)
    }

    /// Copy the selected rows in an organized manner
    fn copy_selected_cells(&mut self, ui: &mut Ui) {
        let all_rows = self.user_table.rows();
        let mut selected_rows = Vec::new();

        let mut column_max_length = HashMap::new();

        // Iter through all the rows and find the rows that have at least one column as selected
        // Keep track of the biggest length of a value of a column
        for row in all_rows {
            if !row.selected_columns.is_empty() {
                for column in &self.user_table.active_columns {
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

        // Target is to ensure a fixed length after each column value of a row
        // If for example highest len is 10 but the current row's
        // column value is 5, we will add the column value and add 5 more space after that
        // to ensure alignment
        for row in selected_rows {
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

    /// Marks all the rows with at least 1 column selected as whitelisted
    fn whitelist_selected_rows(&mut self) {
        let all_rows = self.user_table.rows();
        let mut selected_rows = Vec::new();

        for row in all_rows {
            if !row.selected_columns.is_empty() && row.name != "Anonymous/Unknown" {
                selected_rows.push(row);
            }
        }
        let total_to_whitelist = selected_rows.len();
        let mut packed_chats = Vec::new();

        for row in selected_rows {
            self.user_table.set_as_whitelisted(&row.id);
            self.whitelist_data.add_to_whitelist(
                row.name,
                row.username,
                row.id,
                row.belongs_to.clone().unwrap(),
            );
            packed_chats.push(row.belongs_to.unwrap().pack().to_hex());
        }

        save_whitelisted_users(packed_chats, false);
        self.process_state = ProcessState::UsersWhitelisted(total_to_whitelist);
    }
}
