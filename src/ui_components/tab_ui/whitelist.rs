use eframe::egui::{
    Align, Button, Grid, Key, Label, Layout, RichText, ScrollArea, SelectableLabel, TextEdit, Ui,
};
use egui_extras::{Column, TableBuilder};
use grammers_client::types::Chat;
use log::{error, info};
use std::collections::{HashMap, HashSet};

use crate::tg_handler::ProcessStart;
use crate::ui_components::processor::{ColumnName, PackedWhitelistedUser, ProcessState};
use crate::ui_components::MainWindow;
use crate::utils::{get_whitelisted, save_whitelisted_users, separate_whitelist_by_seen};

#[derive(Clone)]
struct WhiteListRowData {
    name: String,
    username: String,
    id: i64,
    is_selected: bool,
    belongs_to: Chat,
    seen_by: String,
}

impl WhiteListRowData {
    fn new(name: String, username: String, id: i64, belongs_to: Chat, seen_by: String) -> Self {
        WhiteListRowData {
            name,
            username,
            id,
            is_selected: false,
            belongs_to,
            seen_by,
        }
    }
}

#[derive(Default)]
pub struct WhitelistData {
    target_username: String,
    rows: HashMap<i64, WhiteListRowData>,
    active_rows: HashSet<i64>,
    /// Only used when initially loading the saved whitelist data
    /// and for creating the `ProcessState`.
    /// Will never be changed after all whitelists are processed
    failed_whitelist: i32,
}

impl WhitelistData {
    /// Get all rows in a vector
    fn rows(&self) -> Vec<WhiteListRowData> {
        self.rows.values().cloned().collect()
    }

    /// Remove selection from all rows
    fn unselected_all(&mut self) {
        for (_, row) in self.rows.iter_mut() {
            row.is_selected = false;
        }
        self.active_rows.clear();
    }

    /// Select all rows
    fn select_all(&mut self) {
        let mut rows = HashSet::new();
        for (_, row) in self.rows.iter_mut() {
            row.is_selected = true;
            rows.insert(row.id);
        }
        self.active_rows = rows;
    }

    /// Add a new row to the UI
    pub fn add_to_whitelist(
        &mut self,
        name: String,
        username: String,
        id: i64,
        belongs_to: Chat,
        seen_by: String,
    ) {
        let name = if name.is_empty() {
            String::from("Deleted Account")
        } else {
            name
        };

        info!("Adding {name} to whitelist, seen by {seen_by}");
        let to_add = WhiteListRowData::new(name, username, id, belongs_to, seen_by);
        self.rows.insert(id, to_add);
    }

    /// Check if user is whitelisted/in the whitelist UI
    pub fn is_user_whitelisted(&self, id: i64) -> bool {
        self.rows.contains_key(&id)
    }

    /// Save the current row data in the whitelist json
    pub fn save_whitelisted_users(&self, overwrite: bool) {
        let mut packed_chats = Vec::new();

        for row in self.rows.values() {
            let hex_value = row.belongs_to.pack().to_hex();
            packed_chats.push(PackedWhitelistedUser::new(
                hex_value,
                row.seen_by.to_string(),
            ));
        }

        save_whitelisted_users(packed_chats, overwrite);
    }

    /// Removes selected row from whitelist and saves the result
    fn remove_selected(&mut self) -> HashSet<i64> {
        let active_rows = self.active_rows.clone();

        for i in &active_rows {
            info!("Removing user {} from whitelist", i);
            self.rows.remove(i);
        }
        self.save_whitelisted_users(true);
        active_rows
    }

    /// Removes all row from whitelist and saves the result
    fn remove_all(&mut self) -> Vec<i64> {
        info!("Removing all users from whitelist");
        let row_keys = self.rows.keys().map(ToOwned::to_owned).collect();
        self.rows.clear();
        self.save_whitelisted_users(true);

        row_keys
    }

    pub fn clear_text_box(&mut self) {
        self.target_username.clear();
    }

    pub fn increase_failed_by(&mut self, count: i32) {
        self.failed_whitelist += count;
    }

    pub fn row_len(&self) -> usize {
        self.rows.len()
    }

    pub fn failed_whitelist_num(&self) -> i32 {
        self.failed_whitelist
    }
}

impl MainWindow {
    pub fn show_whitelist_ui(&mut self, ui: &mut Ui) {
        let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
        let key_a_pressed = ui.ctx().input(|i| i.key_pressed(Key::A));

        if is_ctrl_pressed && key_a_pressed {
            self.whitelist.select_all();
        }

        Grid::new("Whitelist Grid")
            .num_columns(2)
            .spacing([5.0, 10.0])
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Label::new("Target Username:"));
                });
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut self.whitelist.target_username)
                        .hint_text("Telegram username: @username"),
                )
                .on_hover_text(
                    "Don't have a username? 

Use the Counter to count at least 1 message by that user
then right click on User Table to whitelist",
                );
            });

        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            let button = if self.is_processing || self.whitelist.target_username.is_empty() {
                ui.add_enabled(false, Button::new("Add to whitelist"))
            } else {
                ui.button("Add to whitelist")
            }
            .on_hover_text("Add the username to whitelist");

            if button.clicked() {
                self.whitelist_new_user();
            }
        });

        ui.add_space(40.0);

        ui.horizontal(|ui| {
            if ui
                .button("Select All")
                .on_hover_text("Select all users. Also usable with CTRL + A. Use CTRL + mouse click for manual selection")
                .clicked()
            {
                self.whitelist.select_all();
            };
            if ui
                .button("Delete Selected")
                .on_hover_text("Delete selected users from whitelist")
                .clicked()
            {
                let deleted: Vec<i64> = self.whitelist.remove_selected().into_iter().collect();
                let total_to_remove = deleted.len();

                self.table().remove_whitelist(&deleted);
                self.process_state = ProcessState::WhitelistedUserRemoved(total_to_remove);
            };
            if ui
                .button("Delete All")
                .on_hover_text("Delete all whitelisted users")
                .clicked()
            {
                let deleted = self.whitelist.remove_all();

                self.table().remove_whitelist(&deleted);
                self.chart().reset_saved_bars();
                self.process_state = ProcessState::AllWhitelistRemoved;
            };
        });

        ScrollArea::horizontal()
            .drag_to_scroll(false)
            .show(ui, |ui| {
                let column_size = (ui.available_width() - 20.0) / 3.0;
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::exact(column_size).clip(true))
                    .column(Column::exact(column_size))
                    .column(Column::exact(column_size))
                    .drag_to_scroll(false)
                    .auto_shrink([false; 2])
                    .min_scrolled_height(0.0);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            self.create_whitelist_header(ColumnName::Name, ui);
                        });
                        header.col(|ui| {
                            self.create_whitelist_header(ColumnName::Username, ui);
                        });
                        header.col(|ui| {
                            self.create_whitelist_header(ColumnName::UserID, ui);
                        });
                    })
                    .body(|body| {
                        let table_rows = self.whitelist.rows();
                        body.rows(25.0, table_rows.len(), |mut row| {
                            let row_data = &table_rows[row.index()];
                            row.col(|ui| {
                                self.create_whitelist_row(ColumnName::Name, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_whitelist_row(ColumnName::Username, row_data, ui);
                            });
                            row.col(|ui| {
                                self.create_whitelist_row(ColumnName::UserID, row_data, ui);
                            });
                        });
                    });
            });
    }

    fn create_whitelist_header(&self, column: ColumnName, ui: &mut Ui) {
        let text = column.to_string();
        let hover_text = match column {
            ColumnName::Name => "Telegram name of the user".to_string(),
            ColumnName::Username => "Telegram username of the user".to_string(),

            ColumnName::UserID => "Telegram User ID of the user".to_string(),

            _ => unreachable!(),
        };

        let text = RichText::new(text).strong();
        ui.add_sized(ui.available_size(), Label::new(text))
            .on_hover_text(hover_text);
    }

    fn create_whitelist_row(
        &mut self,
        column: ColumnName,
        row_data: &WhiteListRowData,
        ui: &mut Ui,
    ) {
        let row_text = match column {
            ColumnName::Name => row_data.name.clone(),
            ColumnName::Username => row_data.username.clone(),
            ColumnName::UserID => row_data.id.to_string(),
            _ => unreachable!(),
        };

        let row = ui.add_sized(
            ui.available_size(),
            SelectableLabel::new(row_data.is_selected, row_text),
        );
        row.context_menu(|ui| {
            if ui.button("Delete Selected").clicked() {
                let deleted: Vec<i64> = self.whitelist.remove_selected().into_iter().collect();

                self.table().remove_whitelist(&deleted);
                ui.close_menu();
            }
        });

        if row.clicked() {
            if !ui.ctx().input(|i| i.modifiers.ctrl) {
                self.whitelist.unselected_all();
            }
            let target_row = self.whitelist.rows.get_mut(&row_data.id).unwrap();
            target_row.is_selected = true;
            self.whitelist.active_rows.insert(row_data.id);
        };
    }

    pub fn load_whitelisted_users(&mut self) {
        // This function will never be called if there are no sessions detected.
        // Unnecessary to handle in case `self.tg_clients` is empty

        let all_whitelisted_users = get_whitelisted();

        if all_whitelisted_users.is_err() {
            // This case means it failed to deserialize the json or is using the old whitelist json format
            // All previous data will be removed
            error!("Failed to deserialize a whitelist users json file. Deleting saved json data");
            self.whitelist.save_whitelisted_users(true);
            self.process_state = ProcessState::FailedLoadWhitelistedUsers;
            self.is_processing = false;
            return;
        }

        // separate whitelist data by seen_by as the key and hex as the value
        let separated_data = separate_whitelist_by_seen(all_whitelisted_users.unwrap());

        if separated_data.is_empty() {
            self.is_processing = false;
            return;
        }

        // Open a thread for each unique session found in the whitelist json and pass the relevant hex data to that thread
        // `hex_data` cannot be empty as the key will only exist if there is at least one hex found
        for (seen_by, hex_data) in separated_data {
            let client = self.tg_clients.get(&seen_by);

            let Some(tg_client) = client.cloned() else {
                let total_whitelist = hex_data.len();
                error!(
                    "{seen_by} client does not exist! Ignoring {total_whitelist} whitelisted users"
                );
                self.whitelist.increase_failed_by(total_whitelist as i32);

                let success_whitelist = self.whitelist.row_len();
                let failed_whitelist = self.whitelist.failed_whitelist_num();
                self.process_state =
                    ProcessState::LoadedWhitelistedUsers(success_whitelist, failed_whitelist);
                continue;
            };
            self.runtime.spawn(async move {
                tg_client
                    .start_process(ProcessStart::LoadWhitelistedUsers(hex_data))
                    .await;
            });
        }
    }

    pub fn whitelist_new_user(&mut self) {
        let selected_session = self.get_selected_session();

        if selected_session.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        let client = self.tg_clients.get(&selected_session).unwrap().clone();
        let target_username = self.whitelist.target_username.clone().replace('@', "");
        self.is_processing = true;

        self.runtime.spawn(async move {
            client
                .start_process(ProcessStart::NewWhitelistUser(target_username))
                .await;
        });
    }
}
