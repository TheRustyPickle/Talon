use log::{debug, error, info};

use crate::tg_handler::{ProcessError, ProcessResult};
use crate::ui_components::{MainWindow, ProcessState};

impl MainWindow {
    pub fn check_receiver(&mut self) {
        while let Ok(data) = self.tg_receiver.try_recv() {
            match data {
                ProcessResult::InitialSessionSuccess(client) => {
                    info!("Initial connection to session {} successful", client.name());
                    self.process_state =
                        ProcessState::InitialClientConnectionSuccessful(client.name());
                    self.tg_clients.insert(client.name(), client);
                    self.update_counter_session()
                }
                ProcessResult::InvalidChat(chat_name) => {
                    info!("Invalid chat name found: {}", chat_name);
                    self.process_state = ProcessState::NonExistingChat(chat_name);
                    self.stop_process()
                }
                ProcessResult::UnauthorizedClient(client_name) => {
                    info!("{} is not authorized.", client_name);
                    self.process_state = ProcessState::UnauthorizedClient(client_name);
                    self.stop_process()
                }
                ProcessResult::CountingEnd => {
                    info!("Counting ended");
                    self.process_state = ProcessState::Idle;
                    self.stop_process()
                }
                ProcessResult::CountingMessage(message, start_from, end_at) => {
                    self.process_state = self.process_state.next_dot();
                    let sender_option = message.sender();
                    let mut sender_id = None;

                    if let Some(sender) = sender_option {
                        sender_id = Some(sender.id());
                        self.user_table.add_user(sender);
                    } else {
                        self.user_table.add_unknown_user();
                    }

                    self.user_table.count_user_message(sender_id, &message);

                    let total_user = self.user_table.get_total_user();
                    self.counter_data.set_total_user(total_user);

                    let total_to_iter = start_from - end_at;
                    let message_value = 100.0 / total_to_iter as f32;
                    let current_message_number = message.id();

                    let total_processed = start_from - current_message_number;
                    let processed_percentage = if total_processed != 0 {
                        total_processed as f32 * message_value
                    } else {
                        message_value
                    };
                    self.counter_data.add_one_total_message();
                    debug!(
                        "Bar percentage: {}. Current message: {current_message_number} Total message: {}, Started from: {}",
                        processed_percentage, total_to_iter, start_from
                    );
                    self.counter_data
                        .set_bar_percentage(processed_percentage / 100.0);
                }
                ProcessResult::ProcessFailed(err) => {
                    self.stop_process();
                    match err {
                        ProcessError::InitialClientConnectionError(name) => {
                            error!("Error acquired while trying to connect to the client");
                            self.process_state = ProcessState::InitialConnectionFailed(name)
                        }
                        ProcessError::FileCreationError => {
                            error!("Error acquired while trying to create/delete the session file");
                            self.stop_process();
                            self.process_state = ProcessState::FileCreationFailed;
                        }
                        ProcessError::InvalidTGCode => {
                            error!("Invalid TG Code given for the session");
                            self.stop_process();
                            self.process_state = ProcessState::InvalidTGCode
                        }
                        ProcessError::NotSignedUp => {
                            error!("The phone number is not signed up");
                            self.stop_process();
                            self.process_state = ProcessState::NotSignedUp
                        }
                        ProcessError::UnknownError(e) => {
                            error!("Unknown error encountered while trying to login. {e}");
                            self.stop_process();
                            self.process_state = ProcessState::UnknownError
                        }
                        ProcessError::InvalidPassword => {
                            error!("Invalid TG Password given for the session");
                            self.stop_process();
                            self.process_state = ProcessState::InvalidPassword
                        }
                        ProcessError::InvalidPhonePossibly => {
                            error!("Possibly invalid phone number given for the session");
                            self.stop_process();
                            self.process_state = ProcessState::InvalidPhonePossibly
                        }
                    }
                }
                ProcessResult::LoginCodeSent(token, client) => {
                    info!("Login code sent to the client");
                    self.stop_process();
                    self.tg_clients.insert(client.name(), client);
                    self.session_data.set_login_token(token);
                    self.process_state = ProcessState::TGCodeSent;
                }
                ProcessResult::PasswordRequired(token) => {
                    info!("Client requires a password authentication");
                    self.stop_process();
                    self.session_data.set_password_token(*token);
                    self.process_state = ProcessState::PasswordRequired;
                }
                ProcessResult::LoggedIn(name) => {
                    info!("Logged in to the client {name}");
                    self.stop_process();
                    self.session_data.reset_data();
                    self.process_state = ProcessState::LoggedIn(name);
                    self.update_counter_session()
                }
                ProcessResult::FloodWait => {
                    info!("Flood wait triggered");
                    self.process_state = ProcessState::FloodWait;
                }
            }
        }
    }

    fn stop_process(&mut self) {
        self.is_processing = false;
        self.counter_data.counting_ended();
    }
}
