use grammers_client::types::iter_buffer::InvocationError;
use grammers_client::types::{LoginToken, PasswordToken};
use grammers_mtsender::AuthorizationError;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

use crate::tg_handler::{TGClient, TGCountData};
use crate::ui_components::processor::{UnpackedBlacklistedUser, UnpackedWhitelistedUser};

#[allow(clippy::large_enum_variant)]
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
    UnpackedWhitelist(Vec<UnpackedWhitelistedUser>, i32),
    UnpackedBlacklist(Vec<UnpackedBlacklistedUser>, i32),
    FloodWait,
    WhiteListUser(UnpackedWhitelistedUser),
    BlackListUser(UnpackedBlacklistedUser),
    ChatExists(String, i32, i32),
}

#[derive(Debug)]
pub enum ProcessError {
    AuthorizationError,
    FileCreationError,
    InvalidTGCode,
    InvalidPassword,
    NotSignedUp,
    InvalidPhoneOrAPI(AuthorizationError),
    InvalidAPIKeys,
    FailedLatestMessage,
    UnknownError(InvocationError),
}

/// Used by `TGClient` struct to handle operations
pub enum ProcessStart {
    /// Start chat, start num, end num, multi session, whether to cancel
    StartCount(String, Option<i32>, Option<i32>, bool, Arc<AtomicBool>),
    SignInCode(Arc<Mutex<LoginToken>>, String),
    SignInPasswords(Arc<Mutex<PasswordToken>>, String),
    SessionLogout,
    LoadWhitelistedUsers(Vec<String>),
    LoadBlacklistedUsers(Vec<String>),
    NewWhitelistUser(String),
    NewBlacklistUser(String),
    /// Start chat, start num, end num
    CheckChatExistence(String, Option<i32>, Option<i32>),
}

/// Used when trying to create a new `TGClient` by processing some operations
pub enum NewProcess {
    SendLoginCode(String, String, bool),
    InitialSessionConnect(Vec<String>),
}
