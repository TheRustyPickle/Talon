use eframe::egui::{
    Align, Button, Grid, Label, Layout, Response, RichText, SelectableLabel, Sense, TextEdit, Ui,
};
use egui_extras::Column;
use egui_selectable_table::{
    ColumnOperations, ColumnOrdering, SelectableRow, SelectableTable, SortOrder,
};
use grammers_client::types::Chat;
use log::{error, info};
use std::collections::HashSet;

use crate::tg_handler::ProcessStart;
use crate::ui_components::processor::{ColumnName, PackedWhitelistedUser, ProcessState};
use crate::ui_components::MainWindow;
use crate::utils::{get_whitelisted, save_whitelisted_users, separate_whitelist_by_seen};

#[derive(Default)]
struct Config {
    deleted_selected: bool,
}

impl ColumnOperations<WhiteListRowData, ColumnName, Config> for ColumnName {
    fn column_text(&self, row: &WhiteListRowData) -> String {
        match self {
            ColumnName::Name => row.name.to_string(),
            ColumnName::Username => row.username.to_string(),
            ColumnName::UserID => row.id.to_string(),
            _ => unreachable!(),
        }
    }
    fn create_header(
        &self,
        ui: &mut eframe::egui::Ui,
        _sort_order: Option<SortOrder>,
        _table: &mut SelectableTable<WhiteListRowData, ColumnName, Config>,
    ) -> Option<Response> {
        let label_text = self.to_string();
        let hover_text = match self {
            ColumnName::Name => "Telegram name of the user".to_string(),
            ColumnName::Username => "Telegram username of the user".to_string(),

            ColumnName::UserID => "Telegram User ID of the user".to_string(),
            _ => unreachable!(),
        };

        let label_text = RichText::new(label_text).strong();

        let response = ui
            .add_sized(ui.available_size(), Label::new(label_text))
            .on_hover_text(hover_text);

        Some(response)
    }
    fn create_table_row(
        &self,
        ui: &mut Ui,
        row: &SelectableRow<WhiteListRowData, ColumnName>,
        column_selected: bool,
        table: &mut SelectableTable<WhiteListRowData, ColumnName, Config>,
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
            _ => unreachable!(),
        };
        let is_selected = column_selected;

        let mut label = ui
            .add_sized(
                ui.available_size(),
                SelectableLabel::new(is_selected, &row_text),
            )
            .interact(Sense::drag());

        if show_tooltip {
            label = label.on_hover_text(row_text);
        };
        label.context_menu(|ui| {
            if ui.button("Deleted Selected").clicked() {
                table.config.deleted_selected = true;
                ui.close_menu();
            };
        });
        label
    }
}

impl ColumnOrdering<WhiteListRowData> for ColumnName {
    fn order_by(&self, row_1: &WhiteListRowData, row_2: &WhiteListRowData) -> std::cmp::Ordering {
        row_1.name.cmp(&row_2.name)
    }
}

#[derive(Clone)]
struct WhiteListRowData {
    name: String,
    username: String,
    id: i64,
    belongs_to: Chat,
    seen_by: String,
}

impl WhiteListRowData {
    fn new(name: String, username: String, id: i64, belongs_to: Chat, seen_by: String) -> Self {
        WhiteListRowData {
            name,
            username,
            id,
            belongs_to,
            seen_by,
        }
    }
}

pub struct WhitelistData {
    table: SelectableTable<WhiteListRowData, ColumnName, Config>,
    target_username: String,
    failed_whitelist: i32,
    all_ids: HashSet<i64>,
}

impl Default for WhitelistData {
    fn default() -> Self {
        let table = SelectableTable::new(vec![
            ColumnName::Name,
            ColumnName::Username,
            ColumnName::UserID,
        ])
        .auto_scroll()
        .select_full_row()
        .serial_column()
        .auto_reload(1);
        Self {
            table,
            target_username: String::new(),
            failed_whitelist: 0,
            all_ids: HashSet::new(),
        }
    }
}

impl WhitelistData {
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
        self.all_ids.insert(id);
        self.table.add_modify_row(|_rows| {
            let to_add = WhiteListRowData::new(name, username, id, belongs_to, seen_by);
            Some(to_add)
        });
    }

    /// Check if user is whitelisted/in the whitelist UI
    pub fn is_user_whitelisted(&self, id: i64) -> bool {
        self.all_ids.contains(&id)
    }

    /// Save the current row data in the whitelist json
    pub fn save_whitelisted_users(&self, overwrite: bool) {
        let mut packed_chats = Vec::new();

        self.table.get_all_rows().iter().for_each(|(_id, row)| {
            let hex_value = row.row_data.belongs_to.pack().to_hex();
            packed_chats.push(PackedWhitelistedUser::new(
                hex_value,
                row.row_data.seen_by.to_string(),
            ));
        });

        save_whitelisted_users(packed_chats, overwrite);
    }

    /// Removes selected row from whitelist and saves the result
    fn remove_selected(&mut self) -> Vec<i64> {
        let active_rows = self.table.get_selected_rows();

        let mut row_ids = Vec::new();
        for i in &active_rows {
            info!(
                "Removing user {} | {} from whitelist",
                i.row_data.username, i.row_data.id
            );
            self.all_ids.remove(&i.row_data.id);
            self.table.add_modify_row(|rows| {
                rows.remove(&i.id);
                row_ids.push(i.row_data.id);
                None
            });
        }
        self.save_whitelisted_users(true);
        row_ids
    }

    /// Removes all row from whitelist and saves the result
    fn remove_all(&mut self) -> Vec<i64> {
        info!("Removing all users from whitelist");
        let row_keys = self.all_ids.iter().copied().collect();
        self.table.clear_all_rows();
        self.save_whitelisted_users(true);
        self.all_ids.clear();

        row_keys
    }

    pub fn clear_text_box(&mut self) {
        self.target_username.clear();
    }

    pub fn increase_failed_by(&mut self, count: i32) {
        self.failed_whitelist += count;
    }

    pub fn row_len(&self) -> usize {
        self.table.total_rows()
    }

    pub fn failed_whitelist_num(&self) -> i32 {
        self.failed_whitelist
    }
}

impl MainWindow {
    pub fn show_whitelist_ui(&mut self, ui: &mut Ui) {
        if self.whitelist.table.config.deleted_selected {
            self.whitelist.table.config.deleted_selected = false;
            let rows = self.whitelist.table.get_selected_rows();
            let total_to_remove = rows.len();
            let deleted_ids: Vec<i64> = rows.iter().map(|row| row.row_data.id).collect();

            self.table().remove_whitelist(&deleted_ids);
            self.whitelist.remove_selected();
            self.process_state = ProcessState::WhitelistedUserRemoved(total_to_remove);
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
                self.whitelist.table.select_all();
            };
            if ui
                .button("Delete Selected")
                .on_hover_text("Delete selected users from whitelist")
                .clicked()
            {
                let deleted: Vec<i64> = self.whitelist.remove_selected();
                let total_to_remove = deleted.len();

                self.table().remove_whitelist(&deleted);
                self.chart().reset_saved_bars();
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

        let column_size = (ui.available_width() - 20.0) / 3.0;
        self.whitelist.table.show_ui(ui, |table| {
            table
                .striped(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::exact(column_size).clip(true))
                .column(Column::exact(column_size))
                .column(Column::exact(column_size))
                .drag_to_scroll(false)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0)
        });
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
