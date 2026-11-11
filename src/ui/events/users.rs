use log::info;

use crate::telegram::UsersEvent;
use crate::ui::MainWindow;
use crate::ui::state::{ProcessState, UnpackedBlacklistedUser, UnpackedWhitelistedUser};
use crate::utils::to_chart_name;

impl MainWindow {
    pub(super) fn on_users_event(&mut self, event: UsersEvent) {
        match event {
            UsersEvent::WhitelistLoaded { users, failed } => {
                self.on_whitelist_loaded(users, failed)
            }
            UsersEvent::BlacklistLoaded { users, failed } => {
                self.on_blacklist_loaded(users, failed)
            }
            UsersEvent::WhitelistAdded(chat) => self.on_whitelist_added(*chat),
            UsersEvent::BlacklistAdded(chat) => self.on_blacklist_added(*chat),
        }
    }

    fn on_whitelist_loaded(&mut self, chats: Vec<UnpackedWhitelistedUser>, failed_chats: i32) {
        for chat in chats {
            self.whitelist.add_to_whitelist(
                chat.name,
                chat.username,
                chat.id,
                chat.peer_ref,
                chat.seen_by,
            );
        }
        self.is_processing = false;

        self.whitelist.increase_failed_by(failed_chats);
        let total_chat = self.whitelist.row_len();
        let failed_chat_num = self.whitelist.failed_whitelist_num();

        self.process_state = ProcessState::LoadedWhitelistedUsers(total_chat, failed_chat_num);
    }

    fn on_blacklist_loaded(&mut self, chats: Vec<UnpackedBlacklistedUser>, failed_chats: i32) {
        let mut names = Vec::new();
        let mut user_ids = Vec::new();

        for chat in chats {
            names.push(to_chart_name(chat.username.clone(), &chat.name, chat.id));
            user_ids.push(chat.id);

            self.blacklist.add_to_blacklist(
                chat.name,
                chat.username,
                chat.id,
                chat.peer_ref,
                chat.seen_by,
            );
        }
        self.is_processing = false;

        self.blacklist.increase_failed_by(failed_chats);
        let total_chat = self.blacklist.row_len();
        let failed_chat_num = self.blacklist.failed_blacklist_num();

        for chart in self.chart_all() {
            chart.clear_blacklisted(&names);
        }

        for table in self.table_all() {
            table.remove_blacklisted_rows(&user_ids);
        }

        self.process_state = ProcessState::LoadedBlacklistedUsers(total_chat, failed_chat_num);
    }

    fn on_whitelist_added(&mut self, chat: UnpackedWhitelistedUser) {
        self.stop_process();
        let user_id = chat.id;

        info!("Adding {user_id} to whitelist");

        self.whitelist.add_to_whitelist(
            chat.name,
            chat.username,
            user_id,
            chat.peer_ref,
            chat.seen_by,
        );
        self.whitelist.clear_text_box();
        self.table().set_as_whitelisted(&[user_id]);
        self.chart().reset_saved_bars();
        self.whitelist.save_whitelisted_users(false);
        self.process_state = ProcessState::AddedToWhitelist;
    }

    fn on_blacklist_added(&mut self, chat: UnpackedBlacklistedUser) {
        self.stop_process();
        let user_id = chat.id;

        info!("Adding {user_id} to blacklist");

        let chart_name = to_chart_name(chat.username.clone(), &chat.name, user_id);

        for chart in self.chart_all() {
            chart.clear_blacklisted(std::slice::from_ref(&chart_name));
        }

        for table in self.table_all() {
            table.remove_blacklisted_rows(&[user_id]);
        }

        self.blacklist.add_to_blacklist(
            chat.name,
            chat.username,
            user_id,
            chat.peer_ref,
            chat.seen_by,
        );
        self.blacklist.clear_text_box();
        self.blacklist.save_blacklisted_users(false);
        self.process_state = ProcessState::AddedToBlacklist;
    }
}
