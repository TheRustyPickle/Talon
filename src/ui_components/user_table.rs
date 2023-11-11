use grammers_client::types::Chat;
use std::collections::HashSet;
use eframe::egui::Ui;

use crate::ui_components::MainWindow;

struct UserRowData {
    name: String,
    username: Option<String>,
    id: i64,
}

impl UserRowData {
    fn new(name: &str, username: Option<&str>, id: i64) -> Self {
        let username = username.map(|s| s.to_owned());
        UserRowData {
            name: name.to_string(),
            username: username,
            id,
        }
    }
}

#[derive(Default)]
pub struct UserTableData {
    user_ids: HashSet<i64>,
    rows: Vec<UserRowData>,
}

impl UserTableData {
    pub fn add_user(&mut self, user_data: Chat) {
        let user_id = user_data.id();
        if !self.user_ids.contains(&user_id) {
            let row_data = UserRowData::new(user_data.name(), user_data.username(), user_id);
            self.user_ids.insert(user_id);
            self.rows.push(row_data);
        }
    }

    pub fn add_unknown_user(&mut self) {
        if !self.user_ids.contains(&0) {
            let row_data = UserRowData::new("Unknown", None, 0);
            self.user_ids.insert(0);
            self.rows.push(row_data);
        }
    }

    pub fn get_total_user(&self) -> i32 {
        self.user_ids.len() as i32
    }
}

impl MainWindow {
    pub fn show_user_table_ui(&mut self, ui: &mut Ui) {
        ui.label("Work in progress");
    }
}