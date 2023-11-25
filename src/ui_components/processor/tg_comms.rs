use log::{error, info};

use crate::tg_handler::{ProcessError, ProcessResult};
use crate::ui_components::processor::ProcessState;
use crate::ui_components::MainWindow;

impl MainWindow {
    pub fn check_receiver(&mut self) {
        while let Ok(data) = self.tg_receiver.try_recv() {
            match data {
                ProcessResult::InitialSessionSuccess((clients, success, failed)) => {
                    let mut status_text =
                        format!("Successfully connected to: {}.", success.join(", "));

                    if !failed.is_empty() {
                        status_text
                            .push_str(&format!(" Failed connection to: {}", failed.join(", ")));
                    }

                    for client in clients {
                        self.tg_clients.insert(client.name(), client);
                    }

                    self.process_state =
                        ProcessState::InitialClientConnectionSuccessful(status_text);
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
                ProcessResult::CountingEnd((end_at, last_number)) => {
                    // Example case end point 100, last processed 102
                    // Current num 99 so end point reached
                    // 101 and 100 is missing so count them as deleted
                    if last_number - 1 != end_at {
                        let total_deleted = last_number - end_at;
                        self.counter_data.add_deleted_message(total_deleted);
                    }

                    info!("Counting ended");
                    self.process_state = ProcessState::Idle;
                    self.stop_process()
                }
                ProcessResult::CountingMessage(count_data) => {
                    self.process_state = self.process_state.next_dot();

                    let message = count_data.message();
                    let start_from = count_data.start_at();
                    let end_at = count_data.end_at();
                    let last_number = count_data.last_number();

                    let sender = message.sender();
                    let sender_id = self.user_table.add_user(sender);

                    self.user_table.count_user_message(sender_id, message);

                    let total_user = self.user_table.get_total_user();
                    self.counter_data.set_total_user(total_user);

                    let total_to_iter = start_from - end_at;
                    let message_value = 100.0 / total_to_iter as f32;

                    let current_message_number = message.id();

                    // If current num = 100 and last processed = 105
                    // messages with 101, 102, 103 and 104 are missing/deleted
                    // The current num is already getting processed so subtract 1
                    let total_deleted = if last_number != -1 {
                        (last_number - current_message_number) - 1
                    } else {
                        0
                    };

                    self.counter_data.add_deleted_message(total_deleted);

                    let total_processed = start_from - current_message_number;
                    let processed_percentage = if total_processed != 0 {
                        total_processed as f32 * message_value
                    } else {
                        message_value
                    };
                    self.counter_data.add_one_total_message();
                    self.counter_data
                        .set_bar_percentage(processed_percentage / 100.0);
                }
                ProcessResult::ProcessFailed(err) => {
                    self.stop_process();
                    match err {
                        ProcessError::AuthorizationError => {
                            error!("Error acquired while trying to connect to the client");
                            self.process_state = ProcessState::AuthorizationError
                        }
                        ProcessError::FileCreationError => {
                            error!("Error acquired while trying to create/delete the session file");
                            self.process_state = ProcessState::FileCreationFailed;
                        }
                        ProcessError::InvalidTGCode => {
                            error!("Invalid TG Code given for the session");
                            self.process_state = ProcessState::InvalidTGCode
                        }
                        ProcessError::NotSignedUp => {
                            error!("The phone number is not signed up");
                            self.process_state = ProcessState::NotSignedUp
                        }
                        ProcessError::UnknownError(e) => {
                            error!("Unknown error encountered while trying to login. {e}");
                            self.process_state = ProcessState::UnknownError
                        }
                        ProcessError::InvalidPassword => {
                            error!("Invalid TG Password given for the session");
                            self.process_state = ProcessState::InvalidPassword
                        }
                        ProcessError::InvalidPhoneOrAPI => {
                            error!("Possibly invalid phone number given or API keys error");
                            self.process_state = ProcessState::InvalidPhoneOrAPI
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
