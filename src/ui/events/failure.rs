use log::{error, info};

use crate::telegram::ProcessError;
use crate::ui::MainWindow;
use crate::ui::state::ProcessState;

impl MainWindow {
    pub(super) fn on_failure(&mut self, err: ProcessError) {
        match err {
            ProcessError::FileCreationError => {
                error!("Error acquired while trying to create/delete the session file");
                self.process_state = ProcessState::FileCreationFailed;
            }
            ProcessError::InvalidTGCode => {
                error!("Invalid TG Code given for the session");
                self.process_state = ProcessState::InvalidTGCode;
            }
            ProcessError::NotSignedUp => {
                error!("The phone number is not signed up");
                self.process_state = ProcessState::NotSignedUp;
            }
            ProcessError::UnknownError(e) => {
                error!("Unknown error encountered while trying to complete the process. {e}");
                self.process_state = ProcessState::UnknownError;
            }
            ProcessError::InvalidPassword => {
                error!("Invalid TG Password given for the session");
                self.process_state = ProcessState::InvalidPassword;
            }
            ProcessError::InvalidPhoneOrAPI(e) => {
                error!("Possibly invalid phone number given or API keys error. Error: {e}");
                self.process_state = ProcessState::InvalidPhoneOrAPI;
            }
            ProcessError::InvalidAPIKeys => {
                error!("Invalid API keys were given");
                self.process_state = ProcessState::InvalidAPIKeys;
            }
            ProcessError::InvalidChat(chat_name) => {
                info!("Invalid chat name found: {chat_name}");
                self.process_state = ProcessState::NonExistingChat(chat_name);
            }
            ProcessError::NoUsableSession => {
                error!("Every session is logged out or unreachable");
                self.process_state = ProcessState::NoUsableSession;
            }
            ProcessError::UnknownSession => {
                error!("A command was sent to a session that does not exist");
                self.process_state = ProcessState::EmptySelectedSession;
            }
            ProcessError::FailedLatestMessage => {
                error!("Failed to get the latest message ID");
                self.process_state = ProcessState::LatestMessageLoadingFailed;
            }
        }
        self.go_next_or_stop();
    }
}
