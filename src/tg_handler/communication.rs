use grammers_client::types::Message;

use crate::tg_handler::TGClient;

pub enum ProcessResult {
    NewClient(TGClient),
    InvalidChat(String),
    UnauthorizedClient(String),
    /// Message + Started from + End at
    CountingMessage(Message, i32, i32),
    CountingEnd,
    ProcessFailed(ProcessError),
}

#[derive(Debug)]
pub enum ProcessError {
    InitialClientConnectionError(String),
    FileCreationError,
}

pub enum ProcessStart {
    StartCount(String, Option<i32>, Option<i32>),
}
