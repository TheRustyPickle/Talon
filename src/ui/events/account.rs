use log::info;
use std::fmt::Write as _;

use crate::telegram::AccountEvent;
use crate::ui::MainWindow;
use crate::ui::state::ProcessState;

impl MainWindow {
    pub(super) fn on_account_event(&mut self, event: AccountEvent) {
        match event {
            AccountEvent::SessionsRestored { connected, failed } => {
                self.on_sessions_restored(connected, failed)
            }
            AccountEvent::LoginCodeSent => self.on_login_code_sent(),
            AccountEvent::PasswordRequired => self.on_password_required(),
            AccountEvent::LoggedIn(name) => self.on_logged_in(name),
            AccountEvent::Unauthorized(client_name) => self.on_unauthorized(client_name),
            AccountEvent::SessionsDropped(names) => self.on_sessions_dropped(names),
            AccountEvent::SessionsRefreshed { working } => self.on_sessions_refreshed(working),
        }
    }

    fn on_sessions_restored(&mut self, connected: Vec<String>, failed: Vec<String>) {
        let mut status_text = if connected.is_empty() {
            String::new()
        } else {
            format!("Successfully connected to: {}.", connected.join(", "))
        };

        let any_failed = !failed.is_empty();
        if any_failed {
            let _ = write!(status_text, " Failed connection to: {}", failed.join(", "));
        }

        self.process_state =
            ProcessState::InitialClientConnectionSuccessful(status_text, any_failed);

        // Reloading the lists would add every saved user a second time
        if self.saved_lists_loaded {
            self.stop_process();
        } else {
            self.saved_lists_loaded = true;
            self.load_whitelisted_users();
            self.load_blacklisted_users();
        }
    }

    fn on_login_code_sent(&mut self) {
        info!("Login code sent to the client");
        self.stop_process();
        self.session.code_sent = true;
        self.process_state = ProcessState::TGCodeSent;
    }

    fn on_password_required(&mut self) {
        info!("Client requires a password authentication");
        self.stop_process();
        self.session.password_mode = true;
        self.process_state = ProcessState::PasswordRequired;
    }

    fn on_logged_in(&mut self, name: String) {
        info!("Logged in to the client {name}");
        self.stop_process();
        self.session.reset_data();
        self.process_state = ProcessState::LoggedIn(name);
    }

    fn on_unauthorized(&mut self, client_name: String) {
        info!("{client_name} is not authorized.");
        self.process_state = ProcessState::UnauthorizedClient(client_name);
        self.stop_process();
    }

    fn on_sessions_dropped(&mut self, names: Vec<String>) {
        info!("Sessions removed because they are logged out: {names:?}");
        // The list got shorter, keep the selected session in range
        self.counter.session_index = 0;
        self.process_state = ProcessState::SessionsDropped(names.join(", "));
    }

    fn on_sessions_refreshed(&mut self, working: usize) {
        self.stop_process();
        self.process_state = ProcessState::SessionsRefreshed(working);
    }
}
