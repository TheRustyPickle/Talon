mod account;
mod failure;
mod scan;
mod users;

use crate::telegram::Event;
use crate::ui::MainWindow;

impl MainWindow {
    /// Handles one pending event from the Telegram side. Returns false when there was none
    pub fn check_receiver(&mut self) -> bool {
        let Some(event) = self.backend.poll() else {
            return false;
        };

        match event {
            Event::Account(event) => self.on_account_event(event),
            Event::Scan(event) => self.on_scan_event(event),
            Event::Users(event) => self.on_users_event(event),
            Event::Failed(error) => self.on_failure(error),
        }
        true
    }

    pub fn stop_process(&mut self) {
        self.is_processing = false;
        self.counter.counting_ended();
    }

    fn go_next_or_stop(&mut self) {
        if self.counter.counting() {
            self.counter.increment_ongoing();
            self.process_next_count();
        } else {
            self.stop_process();
        }
    }
}
