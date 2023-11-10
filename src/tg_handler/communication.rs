use crate::tg_handler::TGClient;

pub enum ProcessResult {
    NewClient(TGClient),
    InvalidChat,
    UnauthorizedClient,
}

pub enum ProcessStart {
    StartCount(String, Option<i32>, Option<i32>),
}
