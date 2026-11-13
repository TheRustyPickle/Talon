use log::info;

use crate::telegram::UsersEvent;
use crate::ui::MainWindow;
use crate::ui::state::{ListKind, ProcessState, UnpackedUser};
use crate::utils::to_chart_name;

impl MainWindow {
    pub(super) fn on_users_event(&mut self, event: UsersEvent) {
        match event {
            UsersEvent::ListLoaded {
                kind,
                users,
                failed,
            } => self.on_list_loaded(kind, users, failed),
            UsersEvent::ListAdded { kind, user } => self.on_list_added(kind, *user),
        }
    }

    fn on_list_loaded(&mut self, kind: ListKind, users: Vec<UnpackedUser>, failed: i32) {
        let mut names = Vec::new();
        let mut user_ids = Vec::new();

        for user in users {
            names.push(to_chart_name(user.username.clone(), &user.name, user.id));
            user_ids.push(user.id);

            self.list_data(kind).add_user(
                user.name,
                user.username,
                user.id,
                user.peer_ref,
                user.seen_by,
            );
        }
        self.is_processing = false;

        let list = self.list_data(kind);
        list.increase_failed_by(failed);
        let (total, failed_num) = (list.row_len(), list.failed_num());

        if kind == ListKind::Blacklist {
            self.hide_blacklisted(&names, &user_ids);
        }

        self.process_state = ProcessState::ListLoaded(kind, total, failed_num);
    }

    fn on_list_added(&mut self, kind: ListKind, user: UnpackedUser) {
        self.stop_process();
        let user_id = user.id;

        info!("Adding {user_id} to {}", kind.name());

        match kind {
            ListKind::Whitelist => {
                self.table().set_as_whitelisted(&[user_id]);
                self.chart().reset_saved_bars();
            }
            ListKind::Blacklist => {
                let chart_name = to_chart_name(user.username.clone(), &user.name, user_id);
                self.hide_blacklisted(&[chart_name], &[user_id]);
            }
        }

        let list = self.list_data(kind);
        list.add_user(
            user.name,
            user.username,
            user_id,
            user.peer_ref,
            user.seen_by,
        );

        list.clear_text_box();
        list.save(false);
        self.process_state = ProcessState::ListUserAdded(kind);
    }

    /// Removes the data of blacklisted users from every chart and table
    fn hide_blacklisted(&mut self, chart_names: &[String], user_ids: &[i64]) {
        for chart in self.chart_all() {
            chart.clear_blacklisted(chart_names);
        }

        for table in self.table_all() {
            table.remove_blacklisted_rows(user_ids);
        }
    }
}
