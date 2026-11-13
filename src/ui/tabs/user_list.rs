use eframe::egui::scroll_area::DragScroll;
use eframe::egui::{
    Align, Align2, Button, Color32, Label, Layout, Response, RichText, Sense, TextEdit, Ui, vec2,
};
use egui_extras::Column;
use egui_selectable_table::{
    ColumnOperations, ColumnOrdering, SelectableRow, SelectableTable, SortOrder,
};
use grammers_session::types::PeerRef;
use log::{error, info};
use std::collections::HashSet;

use crate::telegram::Command;
use crate::ui::MainWindow;
use crate::ui::state::{ColumnName, ListKind, PackedUser, ProcessState};
use crate::ui::widgets::{Card, PrimaryButton, RowLabel};
use crate::utils::{get_saved_users, save_users, separate_by_seen};

#[derive(Default)]
struct Config {
    deleted_selected: bool,
}

impl ColumnOperations<ListRowData, ColumnName, Config> for ColumnName {
    fn column_text(&self, row: &ListRowData) -> String {
        match self {
            ColumnName::Name => row.name.clone(),
            ColumnName::Username => row.username.clone(),
            ColumnName::UserID => row.id.to_string(),
            _ => unreachable!(),
        }
    }

    fn create_header(
        &self,
        ui: &mut Ui,
        _sort_order: Option<SortOrder>,
        _table: &mut SelectableTable<ListRowData, ColumnName, Config>,
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
        row: &SelectableRow<ListRowData, ColumnName>,
        column_selected: bool,
        table: &mut SelectableTable<ListRowData, ColumnName, Config>,
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
                RowLabel::new(is_selected, false, &row_text),
            )
            .interact(Sense::drag());

        if show_tooltip {
            label = label.on_hover_text(row_text);
        }

        label.context_menu(|ui| {
            if ui.button("Deleted Selected").clicked() {
                table.config.deleted_selected = true;
                ui.close();
            }
        });

        label
    }
}

impl ColumnOrdering<ListRowData> for ColumnName {
    fn order_by(&self, row_1: &ListRowData, row_2: &ListRowData) -> std::cmp::Ordering {
        row_1.name.cmp(&row_2.name)
    }
}

#[derive(Clone)]
struct ListRowData {
    name: String,
    username: String,
    id: i64,
    belongs_to: PeerRef,
    seen_by: String,
}

impl ListRowData {
    fn new(name: String, username: String, id: i64, belongs_to: PeerRef, seen_by: String) -> Self {
        ListRowData {
            name,
            username,
            id,
            belongs_to,
            seen_by,
        }
    }
}

pub struct UserListData {
    kind: ListKind,
    table: SelectableTable<ListRowData, ColumnName, Config>,
    target_username: String,
    failed: i32,
    all_ids: HashSet<i64>,
    search_query: String,
}

impl UserListData {
    pub fn new(kind: ListKind) -> Self {
        let table = SelectableTable::new(vec![
            ColumnName::Name,
            ColumnName::Username,
            ColumnName::UserID,
        ])
        .auto_scroll()
        .select_full_row()
        .serial_column()
        .no_ctrl_a_capture()
        .auto_reload(1);

        Self {
            kind,
            table,
            target_username: String::new(),
            failed: 0,
            all_ids: HashSet::new(),
            search_query: String::new(),
        }
    }
}

impl UserListData {
    /// Add a new row to the UI
    pub fn add_user(
        &mut self,
        name: String,
        username: String,
        id: i64,
        belongs_to: PeerRef,
        seen_by: String,
    ) {
        let name = if name.is_empty() {
            String::from("Deleted Account")
        } else {
            name
        };

        info!("Adding {name} to {}, seen by {seen_by}", self.kind.name());
        self.all_ids.insert(id);
        self.table.add_modify_row(|_rows| {
            let to_add = ListRowData::new(name, username, id, belongs_to, seen_by);
            Some(to_add)
        });
    }

    /// Check if the user is in the list
    pub fn contains(&self, id: i64) -> bool {
        self.all_ids.contains(&id)
    }

    /// Save the current row data in the list json
    pub fn save(&self, overwrite: bool) {
        let packed_chats = self
            .table
            .get_all_rows()
            .values()
            .filter_map(|row| {
                PackedUser::new(row.row_data.belongs_to, row.row_data.seen_by.clone())
            })
            .collect();

        save_users(self.kind, packed_chats, overwrite);
    }

    /// Removes selected rows from the list and saves the result
    fn remove_selected(&mut self) -> Vec<i64> {
        let active_rows = self.table.get_selected_rows();

        let mut row_ids = Vec::new();
        for i in &active_rows {
            info!(
                "Removing user {} | {} from {}",
                i.row_data.username,
                i.row_data.id,
                self.kind.name()
            );
            self.all_ids.remove(&i.row_data.id);
            self.table.add_modify_row(|rows| {
                rows.remove(&i.id);
                row_ids.push(i.row_data.id);
                None
            });
        }
        self.save(true);
        row_ids
    }

    /// Removes all rows from the list and saves the result
    fn remove_all(&mut self) -> Vec<i64> {
        info!("Removing all users from {}", self.kind.name());
        let row_keys = self.all_ids.iter().copied().collect();
        self.table.clear_all_rows();
        self.save(true);
        self.all_ids.clear();

        row_keys
    }

    pub fn clear_text_box(&mut self) {
        self.target_username.clear();
    }

    pub fn increase_failed_by(&mut self, count: i32) {
        self.failed += count;
    }

    pub fn row_len(&self) -> usize {
        self.table.total_rows()
    }

    pub fn failed_num(&self) -> i32 {
        self.failed
    }
}

impl MainWindow {
    pub fn list_data(&mut self, kind: ListKind) -> &mut UserListData {
        match kind {
            ListKind::Whitelist => &mut self.whitelist,
            ListKind::Blacklist => &mut self.blacklist,
        }
    }

    /// Updates everything that depends on the list after users were removed from it
    fn on_list_users_removed(&mut self, kind: ListKind, ids: &[i64]) {
        // Blacklisted users never had their data hidden, nothing to restore
        if kind == ListKind::Whitelist {
            self.table().remove_whitelist(ids);
            self.chart().reset_saved_bars();
        }
    }

    pub fn show_list_ui(&mut self, ui: &mut Ui, kind: ListKind) {
        let name = kind.name();
        let adjective = kind.adjective();

        if self.list_data(kind).table.config.deleted_selected {
            self.list_data(kind).table.config.deleted_selected = false;
            let deleted = self.list_data(kind).remove_selected();

            self.on_list_users_removed(kind, &deleted);
            self.process_state = ProcessState::ListUsersRemoved(kind, deleted.len());
        }

        Card::new()
            .title(format!("ADD TO {}", name.to_uppercase()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let button_size = vec2(80.0, 28.0);
                    let text_width =
                        ui.available_width() - button_size.x - ui.spacing().item_spacing.x;
                    let is_processing = self.is_processing;
                    let list = self.list_data(kind);

                    ui.add_sized(
                        [text_width, button_size.y],
                        TextEdit::singleline(&mut list.target_username)
                            .align(Align2::LEFT_CENTER)
                            .hint_text("Telegram username: @username"),
                    )
                    .on_hover_text(format!(
                        "Don't have a username?\n\nUse the Scan tab to count at least 1 message by that user\nthen right click on the Users tab to {name}"
                    ));

                    let can_add = !is_processing && !list.target_username.is_empty();
                    let add = ui
                        .add_enabled(can_add, PrimaryButton::new("Add").size(button_size))
                        .on_hover_text(format!("Add the username to {name}"));

                    if add.clicked() {
                        self.add_list_user(kind);
                    }
                });
            });

        ui.add_space(8.0);

        let list = self.list_data(kind);
        let failed = list.failed_num();
        let mut summary = format!("{} users", list.row_len());
        if failed > 0 {
            summary.push_str(&format!(" ({failed} failed to load)"));
        }
        ui.label(RichText::new(summary).weak());
        ui.add_space(4.0);

        let column_size = (ui.available_width() - 20.0) / 3.0;

        ui.horizontal(|ui| {
            let list = self.list_data(kind);
            let text_box = TextEdit::singleline(&mut list.search_query)
                .hint_text("Search by name, username or user ID");
            ui.add(text_box);

            if ui
                .button("Search")
                .on_hover_text("Filter rows that match the search query")
                .clicked()
            {
                let search_query = list.search_query.clone();

                if search_query.is_empty() {
                    list.table.recreate_rows();
                } else {
                    let column_list = vec![ColumnName::Name, ColumnName::Username, ColumnName::UserID];

                    list.table
                        .search_and_show(&column_list, &search_query, None, None);
                }
            }
            if ui
                .button("Clear")
                .on_hover_text("Clear search result")
                .clicked()
            {
                list.table.recreate_rows();
                list.search_query.clear();
            }

            ui.separator();

            if ui
                .button("Select All")
                .on_hover_text("Select all users. Also usable with CTRL + A. Use CTRL + mouse click for manual selection")
                .clicked()
            {
                list.table.select_all();
            }
            if ui
                .button("Delete Selected")
                .on_hover_text(format!("Delete selected users from {name}"))
                .clicked()
            {
                let deleted = list.remove_selected();

                self.on_list_users_removed(kind, &deleted);
                self.process_state = ProcessState::ListUsersRemoved(kind, deleted.len());
            }
            if ui
                .add(Button::new(
                    RichText::new("Delete All").color(Color32::from_rgb(220, 90, 90)),
                ))
                .on_hover_text(format!("Delete all {adjective} users"))
                .clicked()
            {
                let deleted = self.list_data(kind).remove_all();

                self.on_list_users_removed(kind, &deleted);
                self.process_state = ProcessState::ListCleared(kind);
            }
        });

        ui.separator();

        self.list_data(kind).table.show_ui(ui, |table| {
            table
                .striped(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::exact(column_size).clip(true))
                .column(Column::exact(column_size))
                .column(Column::exact(column_size))
                .drag_to_scroll(DragScroll::Never)
                .auto_shrink([false; 2])
                .min_scrolled_height(0.0)
        });
    }

    /// Loads the saved users of a list and asks the sessions that saw them to resolve them
    pub fn load_saved_list(&mut self, kind: ListKind) {
        // This function will never be called if there are no sessions detected.
        // Unnecessary to handle in case is empty

        let saved_users = match get_saved_users(kind) {
            Ok(users) => users,
            Err(_) => {
                // This case means it failed to deserialize the json or is using the old json format
                // All previous data will be removed
                error!(
                    "Failed to deserialize the {} json file. Deleting saved json data",
                    kind.name()
                );
                self.list_data(kind).save(true);
                self.process_state = ProcessState::ListLoadFailed(kind);
                self.is_processing = false;
                return;
            }
        };

        // separate the saved data by seen_by as the key and the peer refs as the value
        let separated_data = separate_by_seen(kind, saved_users);

        if separated_data.is_empty() {
            self.is_processing = false;
            return;
        }

        // Ask each unique session found in the json to resolve its own peer refs
        // `peer_refs` cannot be empty as the key will only exist if there is at least one peer ref found
        for (seen_by, peer_refs) in separated_data {
            if !self.backend.has_session(&seen_by) {
                let total = peer_refs.len();
                error!(
                    "{seen_by} client does not exist! Ignoring {total} {} users",
                    kind.adjective()
                );
                let list = self.list_data(kind);
                list.increase_failed_by(total as i32);

                let (loaded, failed) = (list.row_len(), list.failed_num());
                self.process_state = ProcessState::ListLoaded(kind, loaded, failed);
                continue;
            }

            self.backend.send(Command::LoadList {
                kind,
                session: seen_by,
                peers: peer_refs,
            });
        }
    }

    pub fn add_list_user(&mut self, kind: ListKind) {
        let selected_session = self.get_selected_session();

        if selected_session.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        let username = self
            .list_data(kind)
            .target_username
            .clone()
            .replace('@', "");
        self.is_processing = true;

        self.backend.send(Command::AddListUser {
            kind,
            session: selected_session,
            username,
        });
    }
}
