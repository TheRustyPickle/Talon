use grammers_client::types::iter_buffer::InvocationError;
use grammers_client::types::{LoginToken, PasswordToken};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::tg_handler::{TGClient, TGCountData};
use crate::ui_components::processor::UnpackedWhitelistedUser;

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
    UnpackedChats(Vec<UnpackedWhitelistedUser>, i32),
    FloodWait,
    WhiteListUser(UnpackedWhitelistedUser),
    ChatExists(String, i32, i32),
}

#[derive(Debug)]
pub enum ProcessError {
    AuthorizationError,
    FileCreationError,
    InvalidTGCode,
    InvalidPassword,
    NotSignedUp,
    InvalidPhoneOrAPI,
    InvalidAPIKeys,
    FailedLatestMessage,
    UnknownError(InvocationError),
}

/// Used by `TGClient` struct to handle operations
pub enum ProcessStart {
    StartCount(String, Option<i32>, Option<i32>, bool),
    SignInCode(Arc<Mutex<LoginToken>>, String),
    SignInPasswords(Arc<Mutex<PasswordToken>>, String),
    SessionLogout,
    LoadWhitelistedUsers(Vec<String>),
    NewWhitelistUser(String),
    CheckChatExistence(String, Option<i32>, Option<i32>),
}

/// Used when trying to create a new `TGClient` by processing some operations
pub enum NewProcess {
    SendLoginCode(String, String, bool),
    InitialSessionConnect(Vec<String>),
}
