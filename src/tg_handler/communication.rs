use crate::tg_handler::TGClient;
use grammers_client::types::{iter_buffer::InvocationError, LoginToken, Message, PasswordToken};
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum ProcessResult {
    InitialSessionSuccess(TGClient),
    InvalidChat(String),
    UnauthorizedClient(String),
    /// Message + Started from + End at
    CountingMessage(Message, i32, i32),
    CountingEnd,
    ProcessFailed(ProcessError),
    LoginCodeSent(LoginToken, TGClient),
    PasswordRequired(Box<PasswordToken>),
    LoggedIn(String),
    FloodWait,
}

#[derive(Debug)]
pub enum ProcessError {
    InitialClientConnectionError(String),
    FileCreationError,
    InvalidTGCode,
    InvalidPassword,
    NotSignedUp,
    InvalidPhonePossibly,
    UnknownError(InvocationError),
}

/// Use by TGClient struct to handle operations
pub enum ProcessStart {
    StartCount(String, Option<i32>, Option<i32>),
    SignInCode(Arc<Mutex<LoginToken>>, String),
    SignInPasswords(Arc<Mutex<PasswordToken>>, String),
    SessionLogout,
}

/// Used when trying to create a new TGClient by processing some operations
pub enum NewProcess {
    SendLoginCode(String, String, bool),
    InitialSessionConnect(String),
}
