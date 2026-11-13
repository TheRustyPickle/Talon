use jiff::tz::TimeZone;
use log::info;

use crate::telegram::{Command, ScanEvent, TGCountData};
use crate::ui::MainWindow;
use crate::ui::state::{ProcessState, peer_details};
use crate::utils::to_chart_name;

impl MainWindow {
    pub(super) fn on_scan_event(&mut self, event: ScanEvent) {
        match event {
            ScanEvent::ChatChecked {
                chat,
                start,
                end,
                sessions,
            } => self.on_chat_checked(chat, start, end, sessions),
            ScanEvent::Message(count_data) => self.on_counted_message(*count_data),
            ScanEvent::Finished {
                end_at,
                last_number,
            } => self.on_count_finished(end_at, last_number),
            ScanEvent::FloodWait => self.on_flood_wait(),
        }
    }

    fn on_chat_checked(
        &mut self,
        chat_name: String,
        start_at: i32,
        end_at: i32,
        sessions: Vec<String>,
    ) {
        // Because we count both the start and ending message ID
        let total_to_count = start_at - end_at + 1;
        let total_session = sessions.len();
        let per_session_value = total_to_count / total_session as i32;

        info!("Each session to process {per_session_value}~ messages");

        self.counter.set_session_count(total_session);

        let mut ongoing_start_at = start_at;
        let mut ongoing_end_at = start_at - per_session_value;

        let mut negative_added = false;
        self.backend.reset_cancel();

        for (index, session) in sessions.into_iter().enumerate() {
            self.counter.add_session(session.clone());

            if index == total_session - 1 {
                ongoing_end_at = end_at;
            }

            info!(
                "{} start point {} end point {}",
                session, ongoing_start_at, ongoing_end_at
            );

            self.backend.send(Command::StartCount {
                session,
                chat: chat_name.clone(),
                start: Some(ongoing_start_at),
                end: Some(ongoing_end_at),
                multi_session: true,
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

    fn on_counted_message(&mut self, count_data: TGCountData) {
        self.process_state = self.process_state.next_dot();

        let message = count_data.message();
        let start_from = count_data.start_at();
        let end_at = count_data.end_at();
        let last_number = count_data.last_number();
        let multi_session = count_data.multi_session();

        // Group messages by the day and hour they were sent in the local time zone
        let sent_at = message.date().to_zoned(TimeZone::system());
        let local_time_date = sent_at.date();
        let local_time_datetime = sent_at.datetime();

        let sender = message.sender();

        let user_id = sender.map_or(0, |peer| peer_details(peer).2);

        let blacklisted = self.blacklist.contains(user_id);

        let (user_id, full_name, user_name) = self.t_table().add_user(
            sender,
            count_data.sender_ref(),
            local_time_date,
            local_time_datetime,
            count_data.name(),
            blacklisted,
        );

        let chart_user = to_chart_name(user_name, &full_name, user_id);

        if blacklisted {
            self.t_table().count_blacklisted_message(local_time_date);
        } else {
            self.t_chart().add_user(chart_user.clone(), user_id);
            self.t_table().count_user_message(
                user_id,
                message,
                local_time_date,
                local_time_datetime,
            );
        }

        let whitelisted = self.whitelist.contains(user_id);

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
        let total_deleted = if last_number == -1 {
            0
        } else {
            (last_number - current_message_number) - 1
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
            self.counter
                .set_session_percentage(&count_data.name(), processed_percentage / 100.0);
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

    fn on_count_finished(&mut self, end_at: i32, last_number: i32) {
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

    fn on_flood_wait(&mut self) {
        info!("Flood wait triggered");
        self.process_state = ProcessState::FloodWait;
    }
}
