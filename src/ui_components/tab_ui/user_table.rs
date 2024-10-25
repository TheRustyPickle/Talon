use chrono::{NaiveDate, NaiveDateTime};
use eframe::egui::{
    Align, Button, ComboBox, Key, Layout, Response, RichText, SelectableLabel, Sense, Ui,
};
use egui_extras::{Column, DatePickerButton};
use egui_selectable_table::{
    ColumnOperations, ColumnOrdering, SelectableRow, SelectableTable, SortOrder,
};
use grammers_client::types::{Chat, Message};
use log::info;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::env::current_dir;
use strum::IntoEnumIterator;

use crate::ui_components::processor::{
    ColumnName, DateNavigator, NavigationType, PackedBlacklistedUser, PackedWhitelistedUser,
    ProcessState,
};
use crate::ui_components::widgets::{AnimatedLabel, RowLabel};
use crate::ui_components::MainWindow;
use crate::utils::{entry_insert_user, export_table_data, to_chart_name};

#[derive(Default)]
pub struct Config {
    whitelist_rows: bool,
    blacklisted_rows: bool,
    copy_selected: bool,
}

#[derive(Clone, Serialize)]
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
    #[serde(skip_serializing)]
    belongs_to: Option<Chat>,
    #[serde(skip_serializing)]
    seen_by: String,
}

impl ColumnOperations<UserRowData, ColumnName, Config> for ColumnName {
    fn column_text(&self, row: &UserRowData) -> String {
        match self {
            ColumnName::Name => row.name.to_string(),
            ColumnName::Username => row.username.to_string(),
            ColumnName::UserID => row.id.to_string(),
            ColumnName::TotalMessage => row.total_message.to_string(),
            ColumnName::TotalWord => row.total_word.to_string(),
            ColumnName::TotalChar => row.total_char.to_string(),
            ColumnName::AverageWord => row.average_word.to_string(),
            ColumnName::AverageChar => row.average_char.to_string(),
            ColumnName::FirstMessageSeen => row.first_seen.to_string(),
            ColumnName::LastMessageSeen => row.last_seen.to_string(),
            ColumnName::Whitelisted => row.whitelisted.to_string(),
        }
    }
    fn create_header(
        &self,
        ui: &mut eframe::egui::Ui,
        sort_order: Option<SortOrder>,
        _table: &mut SelectableTable<UserRowData, ColumnName, Config>,
    ) -> Option<Response> {
        let mut label_text = self.to_string();
        let hover_text = match self {
            ColumnName::Name => "Telegram name of the user. Click to sort by name".to_string(),
            ColumnName::Username => {
                "Telegram username of the user. Click to sort by username".to_string()
            }
            ColumnName::UserID => {
                "Telegram User ID of the user. Click to sort by user ID".to_string()
            }
            ColumnName::TotalMessage => {
                "Total messages sent by the user. Click to sort by total message".to_string()
            }
            ColumnName::TotalWord => {
                "Total words in the messages. Click to sort by total words".to_string()
            }
            ColumnName::TotalChar => {
                "Total character in the messages. Click to sort by total character".to_string()
            }
            ColumnName::AverageWord => {
                "Average number of words per message. Click to sort by average words".to_string()
            }
            ColumnName::AverageChar => {
                "Average number of characters per message. Click to sort by average characters"
                    .to_string()
            }

            ColumnName::FirstMessageSeen => {
                "The day the first message that was sent by this user was observed".to_string()
            }
            ColumnName::LastMessageSeen => {
                "The day the last message that was sent by this user was observed".to_string()
            }
            ColumnName::Whitelisted => {
                "Whether this user is whitelisted. Click to sort by whitelist".to_string()
            }
        };

        let is_selected = if let Some(direction) = sort_order {
            match direction {
                SortOrder::Ascending => label_text.push('↓'),
                SortOrder::Descending => label_text.push('↑'),
            }
            true
        } else {
            false
        };

        let label_text = RichText::new(label_text).strong();

        let response = ui
            .add_sized(
                ui.available_size(),
                SelectableLabel::new(is_selected, label_text),
            )
            .on_hover_text(hover_text);
        Some(response)
    }
    fn create_table_row(
        &self,
        ui: &mut Ui,
        row: &SelectableRow<UserRowData, ColumnName>,
        column_selected: bool,
        table: &mut SelectableTable<UserRowData, ColumnName, Config>,
    ) -> Response {
        let row_data = &row.row_data;
        let mut show_tooltip = false;
        let row_text = match self {
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
        let is_selected = column_selected;
        let is_whitelisted = row_data.whitelisted;

        let mut label = ui
            .add_sized(
                ui.available_size(),
                RowLabel::new(is_selected, is_whitelisted, &row_text),
            )
            .interact(Sense::drag());

        if show_tooltip {
            label = label.on_hover_text(row_text);
        };
        label.context_menu(|ui| {
            if ui.button("Copy selected rows").clicked() {
                table.config.copy_selected = true;
                ui.close_menu();
            };
            if ui.button("Whitelist selected rows").clicked() {
                table.config.whitelist_rows = true;
                ui.close_menu();
            };

            if ui.button("Blacklist selected rows").clicked() {
                table.config.blacklisted_rows = true;
                ui.close_menu();
            };
        });
        label
    }
}

impl ColumnOrdering<UserRowData> for ColumnName {
    fn order_by(&self, row_1: &UserRowData, row_2: &UserRowData) -> std::cmp::Ordering {
        match self {
            ColumnName::Name => row_1.name.cmp(&row_2.name),
            ColumnName::Username => row_1.username.cmp(&row_2.username),
            ColumnName::UserID => row_1.id.cmp(&row_2.id),
            ColumnName::TotalMessage => row_1.total_message.cmp(&row_2.total_message),
            ColumnName::TotalWord => row_1.total_word.cmp(&row_2.total_word),
            ColumnName::TotalChar => row_1.total_char.cmp(&row_2.total_char),
            ColumnName::AverageWord => row_1.average_word.cmp(&row_2.average_word),
            ColumnName::AverageChar => row_1.average_char.cmp(&row_2.average_char),
            ColumnName::FirstMessageSeen => row_1.first_seen.cmp(&row_2.first_seen),
            ColumnName::LastMessageSeen => row_1.last_seen.cmp(&row_2.last_seen),
            ColumnName::Whitelisted => row_1.whitelisted.cmp(&row_2.whitelisted),
        }
    }
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
}

pub struct UserTableData {
    /// Key: The Date where at least one message/User was found
    /// Value: A hashmap of the founded User with their user id as the key
    /// Contains all data points and UI points are recreated from here
    user_data: HashMap<NaiveDate, HashMap<i64, UserRowData>>,
    table: SelectableTable<UserRowData, ColumnName, Config>,
    date_nav: DateNavigator,
    total_whitelisted_user: u32,
    total_message: u32,
    total_whitelisted_message: u32,
    reload_count: u8,
}

impl Default for UserTableData {
    fn default() -> Self {
        let table = SelectableTable::new(ColumnName::iter().collect())
            .auto_scroll()
            .serial_column();
        Self {
            user_data: HashMap::new(),
            table,
            date_nav: DateNavigator::default(),
            total_whitelisted_message: 0,
            total_message: 0,
            total_whitelisted_user: 0,
            reload_count: 0,
        }
    }
}

impl UserTableData {
    pub fn reload_count(&self) -> u8 {
        self.reload_count
    }
    pub fn reset_reload_count(&mut self) {
        self.reload_count = 0;
    }
    /// Add a user to the table
    pub fn add_user(
        &mut self,
        sender: Option<Chat>,
        date: NaiveDate,
        datetime: NaiveDateTime,
        seen_by: String,
        blacklisted: bool,
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

        if !blacklisted {
            let user_row = UserRowData::new(
                &full_name, &username, user_id, false, chat, datetime, seen_by,
            );

            entry_insert_user(&mut self.user_data, user_row, user_id, date);
        }

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
        self.reload_count += 1;
        // If a user sends multiple messages in a day, that specific day data needs to be updated
        let target_data = self.user_data.get_mut(&date).unwrap();
        let user_row_data = target_data.get_mut(&user_id).unwrap();

        let message_text = message.text();

        // Update last and first seen in this date for this user
        if user_row_data.first_seen > datetime {
            user_row_data.set_first_seen(datetime);
        }

        if user_row_data.last_seen < datetime {
            user_row_data.set_last_seen(datetime);
        }

        self.date_nav.handler().update_dates(date);

        let total_char = message_text.len() as u32;
        let total_word = message_text.split_whitespace().count() as u32;

        user_row_data.increment_total_message();
        user_row_data.increment_total_word(total_word);
        user_row_data.increment_total_char(total_char);
    }

    pub fn get_total_user(&self) -> usize {
        self.table.total_displayed_rows()
    }

    /// Recreate the rows that will be shown in the UI. Used only when date picker date is updated
    pub fn create_rows(&mut self) {
        let mut id_map = HashMap::new();
        self.table.clear_all_rows();
        let mut total_message = 0;
        let mut whitelisted_user = HashSet::new();
        let mut whitelisted_message = 0;

        // Go by all the data that are within the range and join them together
        for (date, data) in &self.user_data {
            if !self.date_nav.handler().within_range(*date) {
                continue;
            }

            for (id, row) in data {
                total_message += row.total_message;
                if row.whitelisted {
                    whitelisted_user.insert(row.id);
                    whitelisted_message += row.total_message;
                }

                if let Some(row_id) = id_map.get(id) {
                    self.table.add_modify_row(|rows| {
                        let target_row = rows.get_mut(row_id).unwrap();
                        let user_row_data = &mut target_row.row_data;
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
                        None
                    });
                } else {
                    let new_id = self.table.add_modify_row(|_| Some(row.clone()));
                    id_map.insert(row.id, new_id.unwrap());
                }
            }
        }
        self.total_whitelisted_message = whitelisted_message;
        self.total_message = total_message;
        self.total_whitelisted_user = whitelisted_user.len() as u32;
        self.table.recreate_rows();
    }

    /// Mark a row as whitelisted if exists
    pub fn set_as_whitelisted(&mut self, user_id: &[i64]) {
        for (_d, row_data) in self.user_data.iter_mut() {
            for (id, row) in row_data.iter_mut() {
                for u_id in user_id {
                    if id == u_id {
                        row.whitelisted = true;
                    }
                }
            }
        }
        self.create_rows();
    }

    pub fn remove_blacklisted_rows(&mut self, user_id: &[i64]) {
        for (_d, row_data) in self.user_data.iter_mut() {
            for id in user_id {
                row_data.remove(id);
            }
        }
        self.create_rows();
    }

    /// Remove whitelist status from a row if exists
    pub fn remove_whitelist(&mut self, user_id: &[i64]) {
        for (_d, row_data) in self.user_data.iter_mut() {
            for (id, row) in row_data.iter_mut() {
                for u_id in user_id {
                    if id == u_id {
                        row.whitelisted = false;
                    }
                }
            }
        }
        self.create_rows();
    }

    fn export_data(&mut self, chat_name: &str) {
        info!("Starting exporting table data");
        let rows = self.table.get_displayed_rows();
        export_table_data(rows, chat_name);
    }
}

impl MainWindow {
    pub fn show_user_table_ui(&mut self, ui: &mut Ui) {
        let date_enabled = !self.is_processing && !self.table_i().user_data.is_empty();

        let (values, len) = {
            let names = self.counter.get_chat_list();

            if names.is_empty() {
                (vec!["No chat available".to_string()], 0)
            } else {
                let total_val = names.len();
                (names, total_val)
            }
        };
        ui.horizontal(|ui| {
            ui.label("Selected chat:");
            ComboBox::from_id_salt("Table Box").show_index(
                ui,
                &mut self.table_chat_index,
                len,
                |i| &values[i],
            );
            ui.separator();
            let button = Button::new("Export Table Data");
            if ui
                .add_enabled(date_enabled, button)
                .on_hover_text("Export Table data in CSV format")
                .clicked()
            {
                let chat_name = self.counter.selected_chat_name(self.table_chat_index);
                self.table().export_data(&chat_name);
                self.process_state =
                    ProcessState::DataExported(current_dir().unwrap().to_string_lossy().into());
            };
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(format!("Total User: {}", self.table_i().get_total_user()));
            ui.separator();
            ui.label(format!("Total Message: {}", self.table_i().total_message));
            ui.separator();
            ui.label(format!(
                "Whitelisted User: {}",
                self.table_i().total_whitelisted_user
            ));
            ui.separator();
            ui.label(format!(
                "Whitelisted Message: {}",
                self.table_i().total_whitelisted_message
            ));
        });
        ui.separator();

        // Date section remains disabled while data processing is ongoing or the table is empty
        ui.add_enabled_ui(date_enabled, |ui| {
            ui.horizontal(|ui| {
                let table = self.table();

                ui.label("From:");
                ui.add(
                    DatePickerButton::new(table.date_nav.handler().from()).id_salt("1"),
                )
                .on_hover_text("Show data only after this date, including the date itself");

                ui.label("To:");
                ui.add(DatePickerButton::new(table.date_nav.handler().to()).id_salt("2"))
                    .on_hover_text("Show data only before this date, including the date itself");

                let reset_button = ui.button("Reset Date Selection").on_hover_text("Reset selected date to the oldest and the newest date with at least 1 data point");
                if reset_button.clicked() {
                    table.date_nav.handler().reset_dates();
                    table.create_rows();
                }

                ui.separator();

                let hover_position = ui.make_persistent_id("nav_hovered_1");
                let selected_position = ui.make_persistent_id("nav_selected_1");
                for nav in NavigationType::iter() {
                    let selected = table.date_nav.nav_type_i() == nav;
                    let resp = ui.add(AnimatedLabel::new(
                        selected,
                        nav.to_string(),
                        selected_position,
                        hover_position,
                        50.0,
                        20.0,
                        None,
                        (false, false),
                    ));


                    if resp.clicked() {
                        *table.date_nav.nav_type() = nav;
                    }
                }

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
                self.table().date_nav.go_previous();
            } else {
                let key_l_pressed = ui.ctx().input(|i| i.key_pressed(Key::L));
                if key_l_pressed && is_ctrl_pressed {
                    self.table().date_nav.go_next();
                }
            }
        }

        // recreate the rows if either of dates have changed
        if date_enabled && self.table().date_nav.handler().check_date_change() {
            self.table().create_rows();
        }

        ui.add_space(5.0);

        let mut clip_added = 0;

        let to_whitelist_selected = self.table().table.config.whitelist_rows;
        let to_blacklist_selected = self.table().table.config.blacklisted_rows;
        let to_copy = self.table().table.config.copy_selected;

        if to_whitelist_selected {
            self.table().table.config.whitelist_rows = false;
            self.whitelist_selected_rows();
        }

        if to_blacklist_selected {
            self.table().table.config.blacklisted_rows = false;
            self.blacklist_selected_rows();
        }

        if to_copy {
            self.table().table.config.copy_selected = false;
            self.copy_selected_cells(ui);
        }

        self.table().table.show_ui(ui, |builder| {
            let mut table = builder
                .striped(true)
                .resizable(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .drag_to_scroll(false)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0);

            for _ in ColumnName::iter() {
                let mut column = Column::initial(100.0);
                if clip_added < 2 {
                    column = column.clip(true);
                    clip_added += 1;
                }
                table = table.column(column);
            }
            table
        });
    }

    fn copy_selected_cells(&mut self, ui: &mut Ui) {
        self.table().table.copy_selected_cells(ui);
        self.process_state = ProcessState::DataCopied;
    }

    /// Marks all the rows with at least 1 column selected as whitelisted
    fn whitelist_selected_rows(&mut self) {
        let table_selected_rows = self.table().table.get_selected_rows();
        let mut selected_rows = Vec::new();

        for selected in &table_selected_rows {
            let row_data = &selected.row_data;
            if row_data.name != "Anonymous/Unknown" {
                selected_rows.push(row_data);
            }
        }

        let total_to_whitelist = selected_rows.len();
        let mut packed_chats = Vec::new();

        let mut all_ids = Vec::new();
        for row in selected_rows {
            let cloned_row = row.clone();
            all_ids.push(row.id);
            self.whitelist.add_to_whitelist(
                row.name.clone(),
                row.username.clone(),
                row.id,
                row.belongs_to.clone().unwrap(),
                row.seen_by.clone(),
            );
            let hex_value = cloned_row.belongs_to.unwrap().pack().to_hex();
            packed_chats.push(PackedWhitelistedUser::new(hex_value, cloned_row.seen_by));
        }
        self.table().set_as_whitelisted(&all_ids);
        self.chart().reset_saved_bars();

        self.whitelist.save_whitelisted_users(false);
        self.process_state = ProcessState::UsersWhitelisted(total_to_whitelist);
    }

    /// Marks all the rows with at least 1 column selected as blacklisted
    fn blacklist_selected_rows(&mut self) {
        let table_selected_rows = self.table().table.get_selected_rows();
        let mut selected_rows = Vec::new();

        for selected in &table_selected_rows {
            let row_data = &selected.row_data;
            if row_data.name != "Anonymous/Unknown" {
                selected_rows.push(row_data);
            }
        }

        let total_to_blacklist = selected_rows.len();
        let mut packed_chats = Vec::new();

        let mut all_ids = Vec::new();
        let mut names = Vec::new();
        for row in selected_rows {
            let chart_name = to_chart_name(row.username.clone(), &row.name, row.id);
            names.push(chart_name);

            let cloned_row = row.clone();
            all_ids.push(row.id);

            self.blacklist.add_to_blacklist(
                row.name.clone(),
                row.username.clone(),
                row.id,
                row.belongs_to.clone().unwrap(),
                row.seen_by.clone(),
            );
            let hex_value = cloned_row.belongs_to.unwrap().pack().to_hex();
            packed_chats.push(PackedBlacklistedUser::new(hex_value, cloned_row.seen_by));
        }

        for chart in self.chart_all() {
            chart.clear_blacklisted(&names);
        }

        for table in self.table_all() {
            table.remove_blacklisted_rows(&all_ids);
        }

        self.blacklist.save_blacklisted_users(false);
        self.process_state = ProcessState::UsersBlacklisted(total_to_blacklist);
    }
}
