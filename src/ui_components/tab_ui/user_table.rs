use chrono::{NaiveDate, NaiveDateTime};
use eframe::egui::{
    Align, Event, Key, Label, Layout, Response, RichText, ScrollArea, SelectableLabel, Sense, Ui,
};
use egui_extras::{Column, DatePickerButton, TableBuilder};
use grammers_client::types::{Chat, Message};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::ui_components::processor::{
    ColumnName, DateNavigator, NavigationType, PackedWhitelistedUser, ProcessState, SortOrder,
};
use crate::ui_components::widgets::RowLabel;
use crate::ui_components::MainWindow;
use crate::utils::{entry_insert_user, save_whitelisted_users};

#[derive(Clone)]
pub struct UserRowData {
    name: String,
    username: String,
    id: i64,
    total_message: u32,
    total_word: u32,
    total_char: u32,
    average_word: u32,
    average_char: u32,
    first_seen: NaiveDateTime,
    last_seen: NaiveDateTime,
    whitelisted: bool,
    selected_columns: HashSet<ColumnName>,
    belongs_to: Option<Chat>,
    seen_by: String,
}

impl UserRowData {
    fn new(
        name: &str,
        username: &str,
        id: i64,
        whitelisted: bool,
        belongs_to: Option<Chat>,
        date: NaiveDateTime,
        seen_by: String,
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
            first_seen: date,
            last_seen: date,
            whitelisted,
            selected_columns: HashSet::new(),
            belongs_to,
            seen_by,
        }
    }

    /// Increment total message count by 1
    fn increment_total_message(&mut self) {
        self.total_message += 1;
    }

    /// Increment total message count by `amount`
    fn increase_message_by(&mut self, amount: u32) {
        self.total_message += amount;
    }

    /// Increment total word count by `word_num`
    fn increment_total_word(&mut self, word_num: u32) {
        self.total_word += word_num;
        self.average_word = self.total_word / self.total_message;
    }

    /// Increment total char count by `char_num`
    fn increment_total_char(&mut self, char_num: u32) {
        self.total_char += char_num;
        self.average_char = self.total_char / self.total_message;
    }

    /// Update the date this user was first seen in the chat
    fn set_first_seen(&mut self, date: NaiveDateTime) {
        self.first_seen = date;
    }

    /// Update the date this user was last seen in the chat
    fn set_last_seen(&mut self, date: NaiveDateTime) {
        self.last_seen = date;
    }

    /// Get the current length of a column of this row
    fn get_column_length(&self, column: ColumnName) -> usize {
        match column {
            ColumnName::Name => self.name.len(),
            ColumnName::Username => self.username.len(),
            ColumnName::UserID => self.id.to_string().len(),
            ColumnName::TotalMessage => self.total_message.to_string().len(),
            ColumnName::TotalWord => self.total_word.to_string().len(),
            ColumnName::TotalChar => self.total_char.to_string().len(),
            ColumnName::AverageWord => self.average_word.to_string().len(),
            ColumnName::AverageChar => self.average_char.to_string().len(),
            ColumnName::FirstMessageSeen => self.first_seen.to_string().len(),
            ColumnName::LastMessageSeen => self.last_seen.to_string().len(),
            ColumnName::Whitelisted => self.whitelisted.to_string().len(),
        }
    }

    /// Get the text of a column of this row
    fn get_column_text(&self, column: ColumnName) -> String {
        match column {
            ColumnName::Name => self.name.to_string(),
            ColumnName::Username => self.username.to_string(),
            ColumnName::UserID => self.id.to_string(),
            ColumnName::TotalMessage => self.total_message.to_string(),
            ColumnName::TotalWord => self.total_word.to_string(),
            ColumnName::TotalChar => self.total_char.to_string(),
            ColumnName::AverageWord => self.average_word.to_string(),
            ColumnName::AverageChar => self.average_char.to_string(),
            ColumnName::FirstMessageSeen => self.first_seen.to_string(),
            ColumnName::LastMessageSeen => self.last_seen.to_string(),
            ColumnName::Whitelisted => self.whitelisted.to_string(),
        }
    }
}

#[derive(Default)]
pub struct UserTableData {
    user_data: HashMap<NaiveDate, HashMap<i64, UserRowData>>,
    rows: HashMap<i64, UserRowData>,
    formatted_rows: Vec<UserRowData>,
    sorted_by: ColumnName,
    sort_order: SortOrder,
    drag_started_on: Option<(i64, ColumnName)>,
    active_columns: HashSet<ColumnName>,
    active_rows: HashSet<i64>,
    last_active_row: Option<i64>,
    last_active_column: Option<ColumnName>,
    /// To track whether the mouse pointer went beyond the drag point at least once
    beyond_drag_point: bool,
    indexed_user_ids: HashMap<i64, usize>,
    date_nav: DateNavigator,
}

impl UserTableData {
    /// Clear all the rows
    pub fn clear_row_data(&mut self) {
        *self = UserTableData::default();
    }

    /// Add a user to the table
    pub fn add_user(
        &mut self,
        sender: Option<Chat>,
        date: NaiveDate,
        datetime: NaiveDateTime,
        seen_by: String,
    ) -> (i64, String, String) {
        let mut user_id = 0;
        let full_name;
        let username;
        let mut chat = None;

        if let Some(chat_data) = sender {
            user_id = chat_data.id();
            chat = Some(chat_data.clone());

            if let Chat::User(user) = chat_data {
                // As per grammers lib doc, empty name can be given if it's a deleted account
                full_name = if user.full_name().is_empty() {
                    "Deleted Account".to_string()
                } else {
                    user.full_name()
                };

                username = if let Some(name) = user.username() {
                    name.to_string()
                } else {
                    "Empty".to_string()
                };
            } else {
                full_name = if chat_data.name().is_empty() {
                    "Deleted Account".to_string()
                } else {
                    chat_data.name().to_string()
                };

                username = if let Some(name) = chat_data.username() {
                    name.to_string()
                } else {
                    "Empty".to_string()
                };
            }
        } else {
            // If there is no Chat value then it could be an anonymous user
            full_name = "Anonymous/Unknown".to_string();
            username = "Empty".to_string();
        }

        let user_row = UserRowData::new(
            &full_name, &username, user_id, false, chat, datetime, seen_by,
        );

        entry_insert_user(&mut self.user_data, &mut self.rows, user_row, user_id, date);
        self.formatted_rows.clear();

        (user_id, full_name, username)
    }

    /// Update message related column values of a row
    pub fn count_user_message(
        &mut self,
        user_id: i64,
        message: &Message,
        date: NaiveDate,
        datetime: NaiveDateTime,
    ) {
        // If a user sends multiple messages in a day, that specific day data needs to be updated
        let target_data = self.user_data.get_mut(&date).unwrap();
        let user_row_data_1 = target_data.get_mut(&user_id).unwrap();

        // This is for the initial load where the UI will contain every single data.
        // Update accordingly so it has the correct data
        let user_row_data_2 = self.rows.get_mut(&user_id).unwrap();

        let message_text = message.text();

        // Update last and first seen in this date for this user
        if user_row_data_1.first_seen > datetime {
            user_row_data_1.set_first_seen(datetime);
        }

        if user_row_data_1.last_seen < datetime {
            user_row_data_1.set_last_seen(datetime);
        }

        if user_row_data_2.first_seen > datetime {
            user_row_data_2.set_first_seen(datetime);
        }

        if user_row_data_2.last_seen < datetime {
            user_row_data_2.set_last_seen(datetime);
        }

        self.date_nav.handler().update_dates(date);

        let total_char = message_text.len() as u32;
        let total_word = message_text.split_whitespace().count() as u32;

        user_row_data_1.increment_total_message();
        user_row_data_1.increment_total_word(total_word);
        user_row_data_1.increment_total_char(total_char);
        user_row_data_2.increment_total_message();
        user_row_data_2.increment_total_word(total_word);
        user_row_data_2.increment_total_char(total_char);
        self.formatted_rows.clear();
    }

    pub fn get_total_user(&self) -> i32 {
        self.rows.len() as i32
    }

    /// Returns all existing row in the current sorted format in a vector
    fn rows(&mut self) -> Vec<UserRowData> {
        // It needs to be sorted each load otherwise
        // `self.rows` gets updated with newer data
        // Unless recreated after an update, the UI will show outdated data
        if self.formatted_rows.is_empty() || self.formatted_rows.len() != self.rows.len() {
            self.formatted_rows = self.sort_rows();
        }
        self.formatted_rows.clone()
    }

    /// Recreate the rows that will be shown in the UI. Used only when date picker date is updated
    fn create_rows(&mut self) {
        let mut row_data = HashMap::new();

        // Go by all the data that are within the range and join them together
        for (date, data) in &self.user_data {
            if !self.date_nav.handler().within_range(*date) {
                continue;
            }

            for (id, row) in data {
                if row_data.contains_key(id) {
                    let user_row_data: &mut UserRowData = row_data.get_mut(id).unwrap();
                    if user_row_data.first_seen > row.first_seen {
                        user_row_data.set_first_seen(row.first_seen);
                    }

                    if user_row_data.last_seen < row.last_seen {
                        user_row_data.set_last_seen(row.last_seen);
                    }

                    let total_char = row.total_char;
                    let total_word = row.total_word;
                    let total_message = row.total_message;

                    user_row_data.increase_message_by(total_message);
                    user_row_data.increment_total_word(total_word);
                    user_row_data.increment_total_char(total_char);
                } else {
                    row_data.insert(*id, row.clone());
                }
            }
        }
        self.rows = row_data;
    }

    /// Marks a single column of a row as selected
    fn select_single_row_cell(&mut self, user_id: i64, column_name: ColumnName) {
        self.active_columns.insert(column_name);
        self.active_rows.insert(user_id);

        self.rows
            .get_mut(&user_id)
            .unwrap()
            .selected_columns
            .insert(column_name);
        self.formatted_rows.clear();
    }

    /// Continuously called to select rows and columns when dragging has started
    fn select_dragged_row_cell(
        &mut self,
        user_id: i64,
        column_name: ColumnName,
        is_ctrl_pressed: bool,
    ) {
        // If both same then the mouse is still on the same column on the same row so nothing to process
        if self.last_active_row == Some(user_id) && self.last_active_column == Some(column_name) {
            return;
        }

        self.active_columns.insert(column_name);
        self.beyond_drag_point = true;

        let drag_start = self.drag_started_on.unwrap();

        // number of the column of drag starting point and the current cell that we are trying to select
        let drag_start_num = drag_start.1 as i32;
        let ongoing_column_num = column_name as i32;

        let mut new_column_set = HashSet::new();

        let get_previous = ongoing_column_num > drag_start_num;
        let mut ongoing_val = Some(ColumnName::from_num(drag_start_num));

        // row1: column(drag started here) column column
        // row2: column column column
        // row3: column column column
        // row4: column column column (currently here)
        //
        // The goal of this is to ensure from the drag starting point to all the columns till the currently here
        // are considered selected and the rest are removed from active selection even if it was considered active
        //
        // During fast mouse movement active rows can contain columns that are not in the range we are targeting
        // We go from one point to the other point and ensure except those columns nothing else is selected
        //
        // No active row removal if ctrl is being pressed!
        if is_ctrl_pressed {
            self.active_columns.insert(column_name);
        } else if ongoing_column_num == drag_start_num {
            new_column_set.insert(ColumnName::from_num(drag_start_num));
            self.active_columns = new_column_set;
        } else {
            while ongoing_val.is_some() {
                let col = ongoing_val.unwrap();

                let next_column = if get_previous {
                    col.get_next()
                } else {
                    col.get_previous()
                };

                new_column_set.insert(col);

                if next_column == ColumnName::from_num(ongoing_column_num) {
                    new_column_set.insert(next_column);
                    ongoing_val = None;
                } else {
                    ongoing_val = Some(next_column);
                }
            }
            self.active_columns = new_column_set;
        }

        // The rows in the current sorted format
        let all_rows = self.rows();

        // The row the mouse pointer is on
        let current_row = self.rows.get_mut(&user_id).unwrap();

        // If this row already selects the column that we are trying to select, it means the mouse
        // moved backwards from an active column to another active column.
        //
        // Row: column1 column2 (mouse is here) column3 column4
        //
        // In this case, if column 3 or 4 is also found in the active selection then
        // the mouse moved backwards
        let row_contains_column = current_row.selected_columns.contains(&column_name);

        let mut no_checking = false;
        // If we have some data of the last row and column that the mouse was on, then try to unselect
        if row_contains_column
            && self.last_active_row.is_some()
            && self.last_active_column.is_some()
        {
            let last_active_column = self.last_active_column.unwrap();

            // Remove the last column selection from the current row where the mouse is if
            // the previous row and the current one matches
            //
            // column column column
            // column column column
            // column column (mouse is currently here) column(mouse was here)
            //
            // We unselect the bottom right corner column
            if last_active_column != column_name && self.last_active_row.unwrap() == user_id {
                current_row.selected_columns.remove(&last_active_column);
                self.active_columns.remove(&last_active_column);
            }

            // Get the last row where the mouse was
            let last_row = self.rows.get_mut(&self.last_active_row.unwrap()).unwrap();

            self.last_active_row = Some(user_id);

            // If on the same row as the last row, then unselect the column from all other select row
            if user_id == last_row.id {
                if last_active_column != column_name {
                    self.last_active_column = Some(column_name);
                }
            } else {
                no_checking = true;
                // Mouse went 1 row above or below. So just clear all selection from that previous row
                last_row.selected_columns.clear();
            }
        } else {
            // We are in a new row which we have not selected before
            self.last_active_row = Some(user_id);
            self.last_active_column = Some(column_name);
            current_row
                .selected_columns
                .clone_from(&self.active_columns);
        }

        let current_row_index = self.indexed_user_ids.get(&user_id).unwrap().to_owned();

        // Get the row number where the drag started on
        let drag_start_index = self.indexed_user_ids.get(&drag_start.0).unwrap().to_owned();

        if no_checking {
            self.remove_row_selection(
                &all_rows,
                current_row_index,
                drag_start_index,
                is_ctrl_pressed,
            );
        } else {
            // If drag started on row 1, currently on row 5, check from row 4 to 1 and select all columns
            // else go through all rows till a row without any selected column is found. Applied both by incrementing or decrementing index.
            // In case of fast mouse movement following drag started point mitigates the risk of some rows not getting selected
            self.check_row_selection(true, &all_rows, current_row_index, drag_start_index);
            self.check_row_selection(false, &all_rows, current_row_index, drag_start_index);
            self.remove_row_selection(
                &all_rows,
                current_row_index,
                drag_start_index,
                is_ctrl_pressed,
            );
        }
        self.formatted_rows.clear();
    }

    /// Recursively check the rows by either increasing or decreasing the initial index
    /// till the end point or an unselected row is found. Add active columns to the rows that have at least one column selected.
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
            target_row.selected_columns.clone_from(&self.active_columns);
            self.active_rows.insert(target_row.id);

            if check_previous {
                if index != 0 {
                    self.check_row_selection(check_previous, rows, index, drag_start);
                }
            } else if index + 1 != rows.len() {
                self.check_row_selection(check_previous, rows, index, drag_start);
            }
        }
    }

    /// Checks the active rows and unselects rows that are not within the given range
    fn remove_row_selection(
        &mut self,
        rows: &[UserRowData],
        current_index: usize,
        drag_start: usize,
        is_ctrl_pressed: bool,
    ) {
        let active_ids = self.active_rows.clone();
        for id in active_ids {
            let ongoing_index = self.indexed_user_ids.get(&id).unwrap().to_owned();
            let current_row = rows.get(ongoing_index).unwrap();
            let target_row = self.rows.get_mut(&current_row.id).unwrap();

            if current_index > drag_start {
                if ongoing_index >= drag_start && ongoing_index <= current_index {
                    target_row.selected_columns.clone_from(&self.active_columns);
                } else if !is_ctrl_pressed {
                    target_row.selected_columns = HashSet::new();
                    self.active_rows.remove(&target_row.id);
                }
            } else if ongoing_index <= drag_start && ongoing_index >= current_index {
                target_row.selected_columns.clone_from(&self.active_columns);
            } else if !is_ctrl_pressed {
                target_row.selected_columns = HashSet::new();
                self.active_rows.remove(&target_row.id);
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
        self.active_rows.clear();
        self.formatted_rows.clear();
    }

    /// Select all rows and columns
    fn select_all(&mut self) {
        let mut all_columns = vec![ColumnName::Name];
        let mut current_column = ColumnName::Name.get_next();
        let mut all_rows = Vec::new();

        while current_column != ColumnName::Name {
            all_columns.push(current_column);
            current_column = current_column.get_next();
        }

        for (_, row) in self.rows.iter_mut() {
            row.selected_columns.extend(all_columns.clone());
            all_rows.push(row.id);
        }

        self.active_columns.extend(all_columns);
        self.active_rows.extend(all_rows);
        self.last_active_row = None;
        self.last_active_column = None;
        self.formatted_rows.clear();
    }

    /// Change the value it is currently sorted by. Called on header column click
    fn change_sorted_by(&mut self, sort_by: ColumnName) {
        self.unselected_all();
        self.sorted_by = sort_by;
        self.sort_order = SortOrder::default();
        self.indexed_user_ids.clear();
        self.formatted_rows.clear();
    }

    /// Change the order of row sorting. Called on header column click
    fn change_sort_order(&mut self) {
        self.unselected_all();
        if let SortOrder::Ascending = self.sort_order {
            self.sort_order = SortOrder::Descending;
        } else {
            self.sort_order = SortOrder::Ascending;
        }
        self.indexed_user_ids.clear();
        self.formatted_rows.clear();
    }

    /// Mark a row as whitelisted if exists
    pub fn set_as_whitelisted(&mut self, user_id: i64) {
        if let Some(row) = self.rows.get_mut(&user_id) {
            row.whitelisted = true;
        }
        self.formatted_rows.clear();
    }

    /// Remove whitelist status from a row if exists
    pub fn remove_whitelist(&mut self, user_id: i64) {
        if let Some(row) = self.rows.get_mut(&user_id) {
            row.whitelisted = false;
        }
        self.formatted_rows.clear();
    }

    /// Sorts row data based on the current sort order
    fn sort_rows(&mut self) -> Vec<UserRowData> {
        // Use rayon to parallelize. After logging, rayon resulted in 47% increased performance
        let mut row_data: Vec<UserRowData> = self.rows.par_iter().map(|(_, v)| v.clone()).collect();

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
            ColumnName::FirstMessageSeen => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.first_seen.cmp(&b.first_seen)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.first_seen.cmp(&b.first_seen).reverse());
                }
            },
            ColumnName::LastMessageSeen => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.last_seen.cmp(&b.last_seen)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.last_seen.cmp(&b.last_seen).reverse());
                }
            },
            ColumnName::Whitelisted => match self.sort_order {
                SortOrder::Ascending => row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted)),
                SortOrder::Descending => {
                    row_data.sort_by(|a, b| a.whitelisted.cmp(&b.whitelisted).reverse());
                }
            },
        }

        // Will only be empty when sorting style is changed
        if self.indexed_user_ids.is_empty() || self.indexed_user_ids.len() != row_data.len() {
            let indexed_data = row_data
                .par_iter()
                .enumerate()
                .map(|(index, row)| (row.id, index))
                .collect();

            self.indexed_user_ids = indexed_data;
        }

        row_data
    }
}

impl MainWindow {
    pub fn show_user_table_ui(&mut self, ui: &mut Ui) {
        let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
        let key_a_pressed = ui.ctx().input(|i| i.key_pressed(Key::A));
        let copy_initiated = ui.ctx().input(|i| i.events.contains(&Event::Copy));

        if copy_initiated {
            self.copy_selected_cells(ui);
        }
        if is_ctrl_pressed && key_a_pressed {
            self.table.select_all();
        }

        // Date section remains disabled while data processing is ongoing or the table is empty
        let date_enabled = !self.is_processing && !self.table.user_data.is_empty();

        ui.add_enabled_ui(date_enabled, |ui| {
            ui.horizontal(|ui| {
                let table = &mut self.table;

                ui.label("From:");
                ui.add(
                    DatePickerButton::new(table.date_nav.handler().from()).id_source("1"),
                )
                .on_hover_text("Show data only after this date, including the date itself");

                ui.label("To:");
                ui.add(DatePickerButton::new(table.date_nav.handler().to()).id_source("2"))
                    .on_hover_text("Show data only before this date, including the date itself");

                let reset_button = ui.button("Reset Date Selection").on_hover_text("Reset selected date to the oldest and the newest date with at least 1 data point");
                if reset_button.clicked() {
                    table.date_nav.handler().reset_dates();
                    table.create_rows();
                }

                ui.separator();

                ui.selectable_value(table.date_nav.nav_type(), NavigationType::Day, "Day");
                ui.selectable_value(table.date_nav.nav_type(), NavigationType::Week, "Week");
                ui.selectable_value(table.date_nav.nav_type(), NavigationType::Month, "Month");
                ui.selectable_value(table.date_nav.nav_type(), NavigationType::Year, "Year");

                ui.separator();

                let previous_hover = format!("Go back by 1 {} from the current date. Shortcut key: CTRL + H", table.date_nav.nav_name());
                let next_hover = format!("Go next by 1 {} from the current date. Shortcut key: CTRL + L", table.date_nav.nav_name());

                if ui.button(format!("Previous {}", table.date_nav.nav_name())).on_hover_text(previous_hover).clicked() {
                    table.date_nav.go_previous();
                };

                if ui.button(format!("Next {}", table.date_nav.nav_name())).on_hover_text(next_hover).clicked() {
                    table.date_nav.go_next();
                };
            });
        });

        // Monitor for H and L key presses
        if date_enabled {
            let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
            let key_h_pressed = ui.ctx().input(|i| i.key_pressed(Key::H));

            if key_h_pressed && is_ctrl_pressed {
                self.table.date_nav.go_previous();
            } else {
                let key_l_pressed = ui.ctx().input(|i| i.key_pressed(Key::L));
                if key_l_pressed && is_ctrl_pressed {
                    self.table.date_nav.go_next();
                }
            }
        }

        // recreate the rows if either of dates have changed
        if date_enabled && self.table.date_nav.handler().check_date_change() {
            self.table.create_rows();
        }

        ui.add_space(5.0);

        ScrollArea::horizontal()
            .drag_to_scroll(false)
            .show(ui, |ui| {
                let total_header = 11;
                let mut clip_added = 0;
                let mut current_column = ColumnName::Name;

                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .drag_to_scroll(false)
                    .auto_shrink([false; 2])
                    .min_scrolled_height(0.0)
                    .column(Column::initial(25.0).clip(true));

                for _ in 0..total_header {
                    let mut column = Column::initial(100.0);
                    if clip_added < 2 {
                        column = column.clip(true);
                        clip_added += 1;
                    }
                    table = table.column(column);
                }

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.add_sized(ui.available_size(), Label::new("Num"));
                        });
                        for _ in 0..total_header {
                            header.col(|ui| {
                                self.create_header(current_column, ui);
                            });
                            current_column = current_column.get_next();
                        }
                    })
                    .body(|body| {
                        let table_rows = self.table.rows();
                        body.rows(25.0, table_rows.len(), |mut row| {
                            let index = row.index();
                            let row_data = &table_rows[index];
                            row.col(|ui| {
                                ui.add_sized(
                                    ui.available_size(),
                                    Label::new(format!("{}", index + 1)),
                                );
                            });
                            for _ in 0..total_header {
                                row.col(|ui| self.create_table_row(current_column, row_data, ui));
                                current_column = current_column.get_next();
                            }
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
            ColumnName::FirstMessageSeen => row_data.first_seen.to_string(),
            ColumnName::LastMessageSeen => row_data.last_seen.to_string(),
            ColumnName::Whitelisted => {
                let text = if row_data.whitelisted { "Yes" } else { "No" };
                text.to_string()
            }
        };

        let is_selected = row_data.selected_columns.contains(&column_name);
        let is_whitelisted = row_data.whitelisted;

        let mut label = ui
            .add_sized(
                ui.available_size(),
                RowLabel::new(is_selected, is_whitelisted, &row_text),
            )
            .interact(Sense::drag());

        if show_tooltip {
            label = label.on_hover_text(row_text);
        }

        label.context_menu(|ui| {
            if ui.button("Copy selected rows").clicked() {
                self.copy_selected_cells(ui);
                ui.close_menu();
            };
            if ui.button("whitelist selected rows").clicked() {
                self.whitelist_selected_rows();
                ui.close_menu();
            };
        });

        if label.drag_started() {
            // If CTRL is not pressed down and the mouse right click is not pressed, unselect all cells
            if !ui.ctx().input(|i| i.modifiers.ctrl)
                && !ui.ctx().input(|i| i.pointer.secondary_clicked())
            {
                self.table.unselected_all();
            }
            self.table.drag_started_on = Some((row_data.id, column_name));
        }
        if label.drag_stopped() {
            self.table.last_active_row = None;
            self.table.last_active_column = None;
            self.table.drag_started_on = None;
            self.table.beyond_drag_point = false;
        }

        // Drag part handling has ended, need to handle click event from here.
        // For some reason if both are added at once, only the one added later responds
        label = label.interact(Sense::click());

        if label.clicked() {
            // If CTRL is not pressed down and the mouse right click is not pressed, unselect all cells
            if !ui.ctx().input(|i| i.modifiers.ctrl)
                && !ui.ctx().input(|i| i.pointer.secondary_clicked())
            {
                self.table.unselected_all();
            }
            self.table.select_single_row_cell(row_data.id, column_name);
        }

        if ui.ui_contains_pointer() && self.table.drag_started_on.is_some() {
            if let Some(drag_start) = &self.table.drag_started_on {
                // Only call drag either when not on the starting drag row/column or went beyond the
                // drag point at least once. Otherwise normal click would be considered as drag
                if drag_start.0 != row_data.id
                    || drag_start.1 != column_name
                    || self.table.beyond_drag_point
                {
                    let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
                    self.table
                        .select_dragged_row_cell(row_data.id, column_name, is_ctrl_pressed);
                }
            }
        }
    }

    /// Create a header column
    fn create_header(&mut self, column_name: ColumnName, ui: &mut Ui) {
        let is_selected = self.table.sorted_by == column_name;
        let (label_text, hover_text) = self.get_header_text(column_name);

        let response = ui
            .add_sized(
                ui.available_size(),
                SelectableLabel::new(is_selected, label_text),
            )
            .on_hover_text(hover_text);

        self.handle_header_selection(&response, is_selected, column_name);
    }

    /// Handles sort order and value on header click
    fn handle_header_selection(
        &mut self,
        response: &Response,
        is_selected: bool,
        sort_type: ColumnName,
    ) {
        if response.clicked() {
            if is_selected {
                self.table.change_sort_order();
            } else {
                self.table.change_sorted_by(sort_type);
            }
        }
    }

    fn get_header_text(&mut self, header_type: ColumnName) -> (RichText, String) {
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
            ColumnName::FirstMessageSeen => (
                "First Message Seen".to_string(),
                "The day the first message that was sent by this user was observed".to_string(),
            ),
            ColumnName::LastMessageSeen => (
                "Last Message Seen".to_string(),
                "The day the last message that was sent by this user was observed".to_string(),
            ),
            ColumnName::Whitelisted => (
                "Whitelisted".to_string(),
                "Whether this user is whitelisted. Click to sort by whitelist".to_string(),
            ),
        };

        if header_type == self.table.sorted_by {
            match self.table.sort_order {
                SortOrder::Ascending => text.push('🔽'),
                SortOrder::Descending => text.push('🔼'),
            };
        }
        (RichText::new(text).strong(), hover_text)
    }

    /// Copy the selected rows in an organized manner
    fn copy_selected_cells(&mut self, ui: &mut Ui) {
        let all_rows = self.table.rows();
        let mut selected_rows = Vec::new();

        let mut column_max_length = HashMap::new();

        // Iter through all the rows and find the rows that have at least one column as selected
        // Keep track of the biggest length of a value of a column
        for row in all_rows {
            if !row.selected_columns.is_empty() {
                for column in &self.table.active_columns {
                    if row.selected_columns.contains(column) {
                        let field_length = row.get_column_length(*column);
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
                if self.table.active_columns.contains(&ongoing_column)
                    && row.selected_columns.contains(&ongoing_column)
                {
                    total_cells += 1;
                    let column_text = row.get_column_text(ongoing_column);
                    row_text += &format!(
                        "{:<width$}",
                        column_text,
                        width = column_max_length[&ongoing_column] + 1
                    );
                } else if self.table.active_columns.contains(&ongoing_column)
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
        let all_rows = self.table.rows();
        let mut selected_rows = Vec::new();

        for row in all_rows {
            if !row.selected_columns.is_empty() && row.name != "Anonymous/Unknown" {
                selected_rows.push(row);
            }
        }
        let total_to_whitelist = selected_rows.len();
        let mut packed_chats = Vec::new();

        for row in selected_rows {
            let cloned_row = row.clone();
            self.table.set_as_whitelisted(row.id);
            self.whitelist.add_to_whitelist(
                row.name,
                row.username,
                row.id,
                row.belongs_to.clone().unwrap(),
                row.seen_by,
            );
            let hex_value = cloned_row.belongs_to.unwrap().pack().to_hex();
            packed_chats.push(PackedWhitelistedUser::new(hex_value, cloned_row.seen_by));
        }
        self.chart.reset_saved_bars();

        save_whitelisted_users(packed_chats, false);
        self.process_state = ProcessState::UsersWhitelisted(total_to_whitelist);
    }
}
