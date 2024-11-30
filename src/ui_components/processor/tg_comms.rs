use chrono::{Local, TimeZone};
use log::{error, info};
use std::sync::atomic::Ordering;

use crate::tg_handler::{ProcessError, ProcessResult, ProcessStart};
use crate::ui_components::processor::ProcessState;
use crate::ui_components::MainWindow;
use crate::utils::to_chart_name;

impl MainWindow {
    /// Checks if there are any new message from the async side
    pub fn check_receiver(&mut self) -> bool {
        if let Ok(data) = self.tg_receiver.try_recv() {
            match data {
                ProcessResult::InitialSessionSuccess((clients, success, failed)) => {
                    let mut status_text = if success.is_empty() {
                        String::new()
                    } else {
                        format!("Successfully connected to: {}.", success.join(", "))
                    };

                    if !failed.is_empty() {
                        status_text
                            .push_str(&format!(" Failed connection to: {}", failed.join(", ")));
                    }

                    for client in clients {
                        self.tg_clients.insert(client.name(), client);
                    }

                    self.process_state =
                        ProcessState::InitialClientConnectionSuccessful(status_text);
                    self.load_whitelisted_users();
                    self.load_blacklisted_users();
                }
                ProcessResult::InvalidChat(chat_name) => {
                    info!("Invalid chat name found: {}", chat_name);
                    self.process_state = ProcessState::NonExistingChat(chat_name);
                    self.go_next_or_stop();
                }
                ProcessResult::UnauthorizedClient(client_name) => {
                    info!("{} is not authorized.", client_name);
                    self.process_state = ProcessState::UnauthorizedClient(client_name);
                    self.stop_process();
                }
                ProcessResult::CountingEnd((end_at, last_number)) => {
                    // Example case end point 100, last processed 102
                    // Current num 99 so end point reached
                    // 101 and 100 is missing so count them as deleted
                    if last_number != end_at {
                        let total_deleted = last_number - end_at;
                        self.t_count().add_deleted_message(total_deleted);
                    }

                    info!("Counting ended for a session");

                    // Stop process sets the progress bar to 100
                    // Progress only if 1 session is remaining to be completed or it was 0 (0 in normal counting)
                    // Also recreate table data after counting session ends for a chat
                    if self.counter.session_remaining() <= 1 {
                        self.t_table().create_rows();
                        let total_user = self.t_table().get_total_users_full();
                        self.t_count().set_total_user(total_user as i32);
                        self.go_next_or_stop();
                    } else {
                        self.counter.reduce_session();
                    }
                }
                ProcessResult::CountingMessage(count_data) => {
                    self.process_state = self.process_state.next_dot();

                    let message = count_data.message();
                    let start_from = count_data.start_at();
                    let end_at = count_data.end_at();
                    let last_number = count_data.last_number();
                    let multi_session = count_data.multi_session();

                    let message_sent_at = message.date().naive_utc();
                    let local_time_date = Local.from_utc_datetime(&message_sent_at).date_naive();
                    let local_time_datetime =
                        Local.from_utc_datetime(&message_sent_at).naive_local();

                    let sender = message.sender();

                    let user_id = if let Some(c) = &sender { c.id() } else { 0 };

                    let blacklisted = self.blacklist.is_user_blacklisted(user_id);

                    let (user_id, full_name, user_name) = self.t_table().add_user(
                        sender,
                        local_time_date,
                        local_time_datetime,
                        count_data.name(),
                        blacklisted,
                    );

                    let chart_user = to_chart_name(user_name, &full_name, user_id);

                    if !blacklisted {
                        self.t_chart().add_user(chart_user.clone(), user_id);
                        self.t_table().count_user_message(
                            user_id,
                            message,
                            local_time_date,
                            local_time_datetime,
                        );
                    }

                    let whitelisted = self.whitelist.is_user_whitelisted(user_id);

                    if user_id != 0 && whitelisted && !blacklisted {
                        self.t_table().set_as_whitelisted(&[user_id]);
                    }

                    let total_user = self.t_table().get_total_users_full();
                    self.t_count().set_total_user(total_user as i32);

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

                    self.t_count().add_deleted_message(total_deleted);

                    let total_processed = start_from - current_message_number;
                    let processed_percentage = if total_processed != 0 {
                        total_processed as f32 * message_value
                    } else {
                        message_value
                    };

                    self.t_count().add_one_total_message();
                    if whitelisted {
                        self.t_count().add_one_whitelisted_message();
                        self.t_count().add_whitelisted_user(user_id);
                    }

                    // In single session set the progress by explicitly by counting it on the go
                    // On multi session add whatever percentage there is + new value for this session
                    if multi_session {
                        self.counter.set_session_percentage(
                            &count_data.name(),
                            processed_percentage / 100.0,
                        );
                    } else {
                        self.counter
                            .set_bar_percentage(processed_percentage / 100.0);
                    }

                    if !blacklisted {
                        self.t_chart().add_message(
                            local_time_datetime,
                            local_time_date,
                            chart_user,
                            &count_data.name(),
                        );
                    }
                    if self.t_table().reload_count() > 100 {
                        self.t_table().create_rows();
                        self.t_table().reset_reload_count();
                    }
                }
                ProcessResult::ProcessFailed(err) => {
                    match err {
                        ProcessError::AuthorizationError => {
                            error!("Error acquired while trying to connect to the client");
                            self.process_state = ProcessState::AuthorizationError;
                        }
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
                            error!(
                                "Possibly invalid phone number given or API keys error. Error: {e}"
                            );
                            self.process_state = ProcessState::InvalidPhoneOrAPI;
                        }
                        ProcessError::InvalidAPIKeys => {
                            error!("Invalid API keys were given");
                            self.process_state = ProcessState::InvalidAPIKeys;
                        }
                        ProcessError::FailedLatestMessage => {
                            error!("Failed to get the latest message ID");
                            self.process_state = ProcessState::LatestMessageLoadingFailed;
                        }
                    }
                    self.go_next_or_stop();
                }
                ProcessResult::LoginCodeSent(token, client) => {
                    info!("Login code sent to the client");
                    self.stop_process();
                    self.incomplete_tg_client = Some(client);
                    self.session.set_login_token(token);
                    self.process_state = ProcessState::TGCodeSent;
                }
                ProcessResult::PasswordRequired(token) => {
                    info!("Client requires a password authentication");
                    self.stop_process();
                    self.session.set_password_token(*token);
                    self.process_state = ProcessState::PasswordRequired;
                }
                ProcessResult::LoggedIn(name) => {
                    info!("Logged in to the client {name}");
                    self.stop_process();
                    self.session.reset_data();
                    let incomplete_client = self.incomplete_tg_client.take().unwrap();
                    self.tg_clients
                        .insert(incomplete_client.name(), incomplete_client);
                    self.process_state = ProcessState::LoggedIn(name);
                }
                ProcessResult::FloodWait => {
                    info!("Flood wait triggered");
                    self.process_state = ProcessState::FloodWait;
                }
                ProcessResult::UnpackedWhitelist(chats, failed_chats) => {
                    for chat in chats {
                        let username = if let Some(name) = chat.user_chat.username() {
                            name.to_string()
                        } else {
                            String::from("Empty")
                        };
                        self.whitelist.add_to_whitelist(
                            chat.user_chat.name().to_string(),
                            username,
                            chat.user_chat.id(),
                            chat.user_chat,
                            chat.seen_by,
                        );
                    }
                    self.is_processing = false;

                    self.whitelist.increase_failed_by(failed_chats);
                    let total_chat = self.whitelist.row_len();
                    let failed_chat_num = self.whitelist.failed_whitelist_num();

                    self.process_state =
                        ProcessState::LoadedWhitelistedUsers(total_chat, failed_chat_num);
                }
                ProcessResult::UnpackedBlacklist(chats, failed_chats) => {
                    let mut names = Vec::new();
                    let mut user_ids = Vec::new();

                    for chat in chats {
                        let username = if let Some(name) = chat.user_chat.username() {
                            name.to_string()
                        } else {
                            String::from("Empty")
                        };
                        let full_name = chat.user_chat.name().to_string();
                        let user_id = chat.user_chat.id();

                        names.push(to_chart_name(username.clone(), &full_name, user_id));
                        user_ids.push(user_id);

                        self.blacklist.add_to_blacklist(
                            full_name,
                            username,
                            user_id,
                            chat.user_chat,
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

                    self.process_state =
                        ProcessState::LoadedBlacklistedUsers(total_chat, failed_chat_num);
                }
                ProcessResult::WhiteListUser(chat) => {
                    self.stop_process();
                    let user_id = chat.user_chat.id();

                    info!("Adding {user_id} to whitelist");

                    let username = if let Some(name) = chat.user_chat.username() {
                        name.to_string()
                    } else {
                        String::from("Empty")
                    };
                    self.whitelist.add_to_whitelist(
                        chat.user_chat.name().to_string(),
                        username,
                        user_id,
                        chat.user_chat,
                        chat.seen_by,
                    );
                    self.whitelist.clear_text_box();
                    self.table().set_as_whitelisted(&[user_id]);
                    self.chart().reset_saved_bars();
                    self.whitelist.save_whitelisted_users(false);
                    self.process_state = ProcessState::AddedToWhitelist;
                }
                ProcessResult::BlackListUser(chat) => {
                    self.stop_process();
                    let user_id = chat.user_chat.id();

                    info!("Adding {user_id} to blacklist");

                    let username = if let Some(name) = chat.user_chat.username() {
                        name.to_string()
                    } else {
                        String::from("Empty")
                    };
                    let full_name = chat.user_chat.name().to_string();
                    let chart_name = to_chart_name(username.clone(), &full_name, user_id);

                    for chart in self.chart_all() {
                        chart.clear_blacklisted(&[chart_name.clone()]);
                    }

                    for table in self.table_all() {
                        table.remove_blacklisted_rows(&[user_id]);
                    }

                    self.blacklist.add_to_blacklist(
                        full_name,
                        username,
                        user_id,
                        chat.user_chat,
                        chat.seen_by,
                    );
                    self.blacklist.clear_text_box();
                    self.blacklist.save_blacklisted_users(false);
                    self.process_state = ProcessState::AddedToBlacklist;
                }
                ProcessResult::ChatExists(chat_name, start_at, end_at) => {
                    // Because we count both the start and ending message ID
                    let total_to_count = start_at - end_at + 1;
                    let total_session = self.tg_clients.len();
                    let per_session_value = total_to_count / total_session as i32;

                    info!("Each session to process {}~ messages", per_session_value);

                    self.counter.set_session_count(total_session);

                    let mut ongoing_start_at = start_at;
                    let mut ongoing_end_at = start_at - per_session_value;

                    let mut negative_added = false;
                    self.cancel_count.store(false, Ordering::Relaxed);
                    for (index, client) in self.tg_clients.values().enumerate() {
                        let cancel = self.cancel_count.clone();
                        self.counter.add_session(client.name());

                        let client = client.clone();
                        let chat_name = chat_name.clone();
                        if index == total_session - 1 {
                            ongoing_end_at = end_at;
                        }
                        info!(
                            "{} start point {} end point {}",
                            client.name(),
                            ongoing_start_at,
                            ongoing_end_at
                        );
                        self.runtime.spawn(async move {
                            client
                                .start_process(ProcessStart::StartCount(
                                    chat_name,
                                    Some(ongoing_start_at),
                                    Some(ongoing_end_at),
                                    true,
                                    cancel,
                                ))
                                .await;
                        });
                        ongoing_start_at -= per_session_value;
                        ongoing_end_at -= per_session_value;

                        // because we count both starting and end point
                        // Example starting point 100, end point 1. Total session 4, per session to process 25 messages
                        // First session start at 100 end at 100 - 25 = 75
                        // Next session start at (last session start - per session process) end at (last session end - per session process)
                        //
                        // Here start at would be 75 which will overlap a message so reduce 1
                        //
                        // next session start at 49 end at 25 and so on
                        if !negative_added {
                            ongoing_start_at -= 1;
                            negative_added = true;
                        }
                    }
                }
            }
            true
        } else {
            false
        }
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
