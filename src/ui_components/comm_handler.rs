use log::{debug, info};

use crate::tg_handler::{ProcessError, ProcessResult};
use crate::ui_components::{MainWindow, ProcessState};

impl MainWindow {
    pub fn check_receiver(&mut self) {
        while let Ok(data) = self.tg_receiver.try_recv() {
            match data {
                ProcessResult::NewClient(client) => {
                    self.process_state =
                        ProcessState::InitialClientConnectionSuccessful(client.name());
                    self.tg_clients.push(client);
                    if self.tg_clients.len() == 1 {
                        self.update_counter_session()
                    }
                }
                ProcessResult::InvalidChat(chat_name) => {
                    self.process_state = ProcessState::NonExistingChat(chat_name)
                }
                ProcessResult::UnauthorizedClient(client_name) => {
                    self.process_state = ProcessState::UnauthorizedClient(client_name)
                }
                ProcessResult::CountingEnd => {
                    info!("Counting ended");
                    self.process_state = ProcessState::Idle;
                    self.counter_data.counting_ended();
                    self.is_processing = false;
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
                ProcessResult::ProcessFailed(err) => match err {
                    ProcessError::InitialClientConnectionError(name) => {
                        self.process_state = ProcessState::InitialConnectionFailed(name)
                    }
                    ProcessError::FileCreationError => {}
                },
            }
        }
    }
}
