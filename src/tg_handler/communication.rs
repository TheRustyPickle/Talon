use grammers_client::types::Message;

use crate::tg_handler::TGClient;

pub enum ProcessResult {
    NewClient(TGClient),
    InvalidChat,
    UnauthorizedClient,
    /// Message + Started from + End at
    CountingMessage(Message, i32, i32),
    CountingEnd,
}

pub enum ProcessStart {
    StartCount(String, Option<i32>, Option<i32>),
}
