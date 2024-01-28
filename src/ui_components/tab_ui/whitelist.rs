use eframe::egui::{
    Align, Button, Grid, Key, Label, Layout, RichText, ScrollArea, SelectableLabel, TextEdit, Ui,
};
use egui_extras::{Column, TableBuilder};
use grammers_client::types::Chat;
use log::info;
use std::collections::{HashMap, HashSet};
use std::thread;

use crate::tg_handler::ProcessStart;
use crate::ui_components::processor::{ColumnName, ProcessState};
use crate::ui_components::MainWindow;
use crate::utils::{get_whitelisted_users, save_whitelisted_users};

#[derive(Clone)]
struct WhiteListRowData {
    name: String,
    username: String,
    id: i64,
    is_selected: bool,
    belongs_to: Chat,
}

impl WhiteListRowData {
    fn new(name: String, username: String, id: i64, belongs_to: Chat) -> Self {
        WhiteListRowData {
            name,
            username,
            id,
            is_selected: false,
            belongs_to,
        }
    }
}

#[derive(Default)]
pub struct WhitelistData {
    target_username: String,
    rows: HashMap<i64, WhiteListRowData>,
    active_rows: HashSet<i64>,
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
    pub fn add_to_whitelist(&mut self, name: String, username: String, id: i64, belongs_to: Chat) {
        let name = if name.is_empty() {
            String::from("Deleted Account")
        } else {
            name
        };

        info!("Adding {} to whitelist", name);
        let to_add = WhiteListRowData::new(name, username, id, belongs_to);
        self.rows.insert(id, to_add);
    }

    /// Check if user is whitelisted/in the whitelist UI
    pub fn is_user_whitelisted(&self, id: &i64) -> bool {
        self.rows.contains_key(id)
    }

    /// Overwrite the current row data in the whitelist json
    fn save_whitelisted_users(&self) {
        let mut packed_chats = Vec::new();

        for row in self.rows.values() {
            packed_chats.push(row.belongs_to.pack().to_hex());
        }

        save_whitelisted_users(packed_chats, true);
    }

    /// Removes selected row from whitelist and saves the result
    fn remove_selected(&mut self) -> HashSet<i64> {
        let active_rows = self.active_rows.clone();

        for i in &active_rows {
            info!("Removing user {} from whitelist", i);
            self.rows.remove(i);
        }
        self.save_whitelisted_users();
        active_rows
    }

    /// Removes all row from whitelist and saves the result
    fn remove_all(&mut self) -> Vec<i64> {
        info!("Removing all users from whitelist");
        let row_keys = self.rows.keys().map(ToOwned::to_owned).collect();
        self.rows.clear();
        self.save_whitelisted_users();

        row_keys
    }

    pub fn clear_text_box(&mut self) {
        self.target_username.clear();
    }
}

impl MainWindow {
    pub fn show_whitelist_ui(&mut self, ui: &mut Ui) {
        let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
        let key_a_pressed = ui.ctx().input(|i| i.key_pressed(Key::A));

        if is_ctrl_pressed && key_a_pressed {
            self.whitelist_data.select_all();
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
                    TextEdit::singleline(&mut self.whitelist_data.target_username)
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
            let button = if self.is_processing || self.whitelist_data.target_username.is_empty() {
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
                self.whitelist_data.select_all();
            };
            if ui
                .button("Delete Selected")
                .on_hover_text("Delete selected users from whitelist")
                .clicked()
            {
                let deleted = self.whitelist_data.remove_selected();

                for i in deleted {
                    self.user_table.remove_whitelist(&i);
                }
            };
            if ui
                .button("Delete All")
                .on_hover_text("Delete all whitelisted users")
                .clicked()
            {
                let deleted = self.whitelist_data.remove_all();

                for i in deleted {
                    self.user_table.remove_whitelist(&i);
                }
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
                        let table_rows = self.whitelist_data.rows();
                        body.rows(25.0, table_rows.len(), |mut row| {
                            let row_data = &table_rows[row.index()];
                            row.col(|ui| self.create_whitelist_row(ColumnName::Name, row_data, ui));
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
        let (text, hover_text) = match column {
            ColumnName::Name => ("Name".to_string(), "Telegram name of the user".to_string()),
            ColumnName::Username => (
                "Username".to_string(),
                "Telegram username of the user".to_string(),
            ),
            ColumnName::UserID => (
                "User ID".to_string(),
                "Telegram User ID of the user".to_string(),
            ),
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
        if ui
            .add_sized(
                ui.available_size(),
                SelectableLabel::new(row_data.is_selected, row_text),
            )
            .context_menu(|ui| {
                if ui.button("Delete Selected").clicked() {
                    let deleted = self.whitelist_data.remove_selected();

                    for i in deleted {
                        self.user_table.remove_whitelist(&i);
                    }
                    ui.close_menu();
                }
            })
            .clicked()
        {
            if !ui.ctx().input(|i| i.modifiers.ctrl) {
                self.whitelist_data.unselected_all();
            }
            let target_row = self.whitelist_data.rows.get_mut(&row_data.id).unwrap();
            target_row.is_selected = true;
            self.whitelist_data.active_rows.insert(row_data.id);
        }
    }

    pub fn load_whitelisted_users(&mut self) {
        let whitelisted = get_whitelisted_users();

        if whitelisted.is_empty() {
            self.is_processing = false;
            return;
        }

        let selected_session = self.get_selected_session();

        if selected_session.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        let client = self.tg_clients.get(&selected_session).unwrap().clone();
        thread::spawn(move || {
            client.start_process(ProcessStart::LoadWhitelistedUsers);
        });
    }

    pub fn whitelist_new_user(&mut self) {
        let selected_session = self.get_selected_session();

        if selected_session.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        let client = self.tg_clients.get(&selected_session).unwrap().clone();
        let target_username = self.whitelist_data.target_username.clone().replace('@', "");
        self.is_processing = true;

        thread::spawn(move || {
            client.start_process(ProcessStart::NewWhitelistUser(target_username));
        });
    }
}
