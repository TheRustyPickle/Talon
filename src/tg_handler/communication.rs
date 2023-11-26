use grammers_client::types::{iter_buffer::InvocationError, Chat, LoginToken, PasswordToken};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::tg_handler::{TGClient, TGCountData};

pub enum ProcessResult {
    InitialSessionSuccess((Vec<TGClient>, Vec<String>, Vec<String>)),
    InvalidChat(String),
    UnauthorizedClient(String),
    CountingMessage(TGCountData),
    CountingEnd((i32, i32)),
    ProcessFailed(ProcessError),
    LoginCodeSent(LoginToken, TGClient),
    PasswordRequired(Box<PasswordToken>),
    LoggedIn(String),
    UnpackedChats(Vec<Chat>),
    FloodWait,
    WhiteListUser(Chat),
}

#[derive(Debug)]
pub enum ProcessError {
    AuthorizationError,
    FileCreationError,
    InvalidTGCode,
    InvalidPassword,
    NotSignedUp,
    InvalidPhoneOrAPI,
    UnknownError(InvocationError),
}

/// Use by TGClient struct to handle operations
pub enum ProcessStart {
    StartCount(String, Option<i32>, Option<i32>),
    SignInCode(Arc<Mutex<LoginToken>>, String),
    SignInPasswords(Arc<Mutex<PasswordToken>>, String),
    SessionLogout,
    LoadWhitelistedUsers,
    NewWhitelistUser(String),
}

/// Used when trying to create a new TGClient by processing some operations
pub enum NewProcess {
    SendLoginCode(String, String, bool),
    InitialSessionConnect(Vec<String>),
}
