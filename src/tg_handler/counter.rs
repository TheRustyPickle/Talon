use log::{debug, error, info};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};

impl TGClient {
    pub async fn start_count(
        &self,
        start_chat: String,
        start_num: Option<i32>,
        end_num: Option<i32>,
    ) -> Result<(), ProcessError> {
        let authorized = self.check_authorization().await?;

        if !authorized {
            return Ok(());
        }

        let tg_chat = self.client().resolve_username(&start_chat).await;

        let tg_chat = if let Ok(chat) = tg_chat {
            chat
        } else {
            error!("Failed to resolve username");
            self.send(ProcessResult::InvalidChat(start_chat));
            return Ok(());
        };

        let tg_chat = if let Some(chat) = tg_chat {
            chat
        } else {
            error!("Found None value for target chat. Stopping processing");
            self.send(ProcessResult::InvalidChat(start_chat));
            return Ok(());
        };

        info!("{} exists. Starting iterating messages", tg_chat.name());

        let end_at = if let Some(num) = end_num { num } else { 0 };
        let mut start_at = if let Some(num) = start_num { num } else { -1 };

        info!(
            "Staring message num {}, ending message num {}",
            start_at, end_at
        );

        let last_sent = Arc::new(Mutex::new(Some(Instant::now())));

        let last_sent_clone = last_sent.clone();
        let sender = self.sender();
        let context = self.context();

        thread::spawn(move || loop {
            sleep(Duration::from_millis(500));

            let last_sent = last_sent_clone.lock().unwrap();

            if let Some(last_sent) = *last_sent {
                let time_passed = last_sent.elapsed().as_millis();
                if time_passed > 500 && time_passed < 1200 {
                    sender.send(ProcessResult::FloodWait).unwrap();
                    context.request_repaint();
                };
            } else {
                break;
            }
        });

        let mut iter_message = self.client().iter_messages(tg_chat);

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
                break;
            }
            if message_num >= end_at {
                self.send(ProcessResult::CountingMessage(message, start_at, end_at));
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

        self.send(ProcessResult::CountingEnd);
        Ok(())
    }
}