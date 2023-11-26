use grammers_client::types::Message;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};

pub struct TGCountData {
    message: Message,
    start_at: i32,
    end_at: i32,
    last_number: i32,
}

impl TGCountData {
    pub fn new(message: Message, start_at: i32, end_at: i32, last_number: i32) -> Self {
        TGCountData {
            message,
            start_at,
            end_at,
            last_number,
        }
    }

    pub fn message(&self) -> &Message {
        &self.message
    }

    pub fn start_at(&self) -> i32 {
        self.start_at
    }

    pub fn end_at(&self) -> i32 {
        self.end_at
    }

    pub fn last_number(&self) -> i32 {
        self.last_number
    }
}

impl TGClient {
    /// Iters through a Telegram chat from a given point to the first message or until the end point is reached
    pub async fn start_count(
        &self,
        start_chat: String,
        start_num: Option<i32>,
        end_num: Option<i32>,
    ) -> Result<(), ProcessError> {
        if !self.check_authorization().await? {
            return Ok(());
        }

        let tg_chat = self.check_username(&start_chat).await;

        let tg_chat = match tg_chat {
            Ok(chat) => chat,
            Err(e) => {
                self.send(e);
                return Ok(());
            }
        };

        info!("{} exists. Starting iterating messages", tg_chat.name());

        let end_at = if let Some(num) = end_num { num } else { 1 };
        let mut start_at = if let Some(num) = start_num { num } else { -1 };

        info!(
            "Staring message num {}, ending message num {}",
            start_at, end_at
        );

        let last_sent = Arc::new(Mutex::new(Some(Instant::now())));

        let last_sent_clone = last_sent.clone();
        let sender = self.sender();
        let context = self.context();

        // Every 500 ms, check when the last communication was made with the GUI.
        // If over 500, let the GUI side know that a flood wait was triggered
        thread::spawn(move || loop {
            sleep(Duration::from_millis(500));

            let last_sent = last_sent_clone.lock().unwrap();

            if let Some(last_sent) = *last_sent {
                let time_passed = last_sent.elapsed().as_millis();
                if time_passed > 500 && time_passed < 1050 {
                    sender.send(ProcessResult::FloodWait).unwrap();
                    context.request_repaint();
                };

                // stop this thread if no activity for over 60 seconds
                if time_passed > 60000 {
                    break;
                }
            } else {
                break;
            }
        });

        let mut last_number = -1;
        let mut iter_message = self.client().iter_messages(tg_chat);

        // Add 1 to offset because the latest message would start from the offset point - 1 message
        // Add 1 to last_number if the starting message is 100 but does not exist and starts from 99, we want to count that missing message
        // If last_number is 100, it would be counted as normal message progression
        if start_at != -1 {
            iter_message = iter_message.offset_id(start_at + 1);
            last_number = start_at + 1;
        }

        while let Some(message) = iter_message
            .next()
            .await
            .map_err(ProcessError::UnknownError)?
        {
            let message_num = message.id();
            debug!("Got message number: {}", message_num);
            if start_at == -1 {
                info!("Setting starting point as {message_num}");
                start_at = message_num
            }

            if message_num < end_at {
                debug!("Went beyond the ending point");
                break;
            }

            if message_num <= start_at {
                let count_data = TGCountData::new(message, start_at, end_at, last_number);
                self.send(ProcessResult::CountingMessage(count_data));
                last_number = message_num;
            }

            // Sleep to prevent flood time being too noticeable/getting triggered
            if start_at - end_at > 3000 {
                sleep(Duration::from_millis(5))
            } else {
                sleep(Duration::from_millis(2))
            }

            {
                let mut last_sent_lock = last_sent.lock().unwrap();
                *last_sent_lock = Some(Instant::now());
            }
        }

        let mut last_sent_lock = last_sent.lock().unwrap();
        *last_sent_lock = None;

        self.send(ProcessResult::CountingEnd((end_at, last_number)));
        Ok(())
    }
}
