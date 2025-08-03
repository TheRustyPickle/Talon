use eframe::egui::Context;
use grammers_client::Client;
use grammers_client::types::Chat;
use log::{error, info};
use std::sync::mpsc::Sender;

use crate::tg_handler::{
    NewProcess, ProcessError, ProcessResult, ProcessStart, connect_to_session, send_login_code,
};

#[derive(Clone)]
pub struct TGClient {
    client: Client,
    name: String,
    sender: Sender<ProcessResult>,
    context: Context,
    is_temporary: bool,
}

impl TGClient {
    pub fn new(
        client: Client,
        name: String,
        sender: Sender<ProcessResult>,
        context: Context,
        is_temporary: bool,
    ) -> Self {
        TGClient {
            client,
            name,
            sender,
            context,
            is_temporary,
        }
    }

    /// Start an operation with a telegram client
    pub async fn start_process(self, process_type: ProcessStart) {
        let result = match process_type {
            ProcessStart::StartCount(start_chat, start_num, end_num, multi_session, cancel) => {
                self.start_count(start_chat, start_num, end_num, multi_session, cancel)
                    .await
            }
            ProcessStart::SignInCode(token, code) => self.sign_in_code(token, code).await,
            ProcessStart::SignInPasswords(token, password) => {
                self.sign_in_password(token, password).await
            }
            ProcessStart::SessionLogout => self.logout().await,
            ProcessStart::LoadWhitelistedUsers(hex_data) => {
                self.load_whitelisted_users(hex_data).await
            }
            ProcessStart::NewWhitelistUser(name) => self.new_whitelist(name).await,
            ProcessStart::LoadBlacklistedUsers(hex_data) => {
                self.load_blacklisted_users(hex_data).await
            }
            ProcessStart::NewBlacklistUser(name) => self.new_blacklist(name).await,
            ProcessStart::CheckChatExistence(name, start, end) => {
                self.check_chat_status(name, start, end).await
            }
        };

        if let Err(err) = result {
            error!("Error acquired while handing a process: {err:?}");
            self.send(ProcessResult::ProcessFailed(err));
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Sends a process result to the GUI side, the only way to communicate with the GUI from async
    pub fn send(&self, data: ProcessResult) {
        self.sender.send(data).unwrap();
        self.context.request_repaint();
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn is_temporary(&self) -> bool {
        self.is_temporary
    }

    pub fn sender(&self) -> Sender<ProcessResult> {
        self.sender.clone()
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    /// Verifies if the current client is authorized for usage
    pub async fn check_authorization(&self) -> Result<bool, ProcessError> {
        let authorized: bool = self
            .client()
            .is_authorized()
            .await
            .map_err(ProcessError::UnknownError)?;

        info!("Client authorization status: {authorized}");

        if !authorized {
            self.send(ProcessResult::UnauthorizedClient(self.name()));
            return Ok(false);
        }
        Ok(true)
    }

    /// Tries to resolve a username to get a Telegram chat account
    pub async fn check_username(&self, chat_name: &str) -> Result<Chat, ProcessResult> {
        let tg_chat = self.client().resolve_username(chat_name).await;

        let Ok(tg_chat) = tg_chat else {
            error!("Failed to resolve username");
            return Err(ProcessResult::InvalidChat(chat_name.to_owned()));
        };

        let Some(tg_chat) = tg_chat else {
            error!("Found None value for target chat. Stopping processing");
            return Err(ProcessResult::InvalidChat(chat_name.to_owned()));
        };

        info!("Target chat {} exist", tg_chat.name());

        Ok(tg_chat)
    }

    /// Logs out of the client
    pub async fn logout(&self) -> Result<(), ProcessError> {
        let _ = self.client().sign_out().await;
        info!("Logged out of client {}", self.name());
        Ok(())
    }
}

/// Start a process that will result in the creation of a `TGClient` if successful
pub async fn start_process(process: NewProcess, sender: Sender<ProcessResult>, context: Context) {
    let result = match process {
        NewProcess::InitialSessionConnect(name) => {
            connect_to_session(sender.clone(), name, &context).await
        }
        NewProcess::SendLoginCode(session_name, phone_number, is_temporary) => {
            send_login_code(
                sender.clone(),
                &context,
                session_name,
                phone_number,
                is_temporary,
            )
            .await
        }
    };

    if let Err(err) = result {
        error!("Error acquired while handing a process: {err:?}");
        sender.send(ProcessResult::ProcessFailed(err)).unwrap();
        context.request_repaint();
    }
}
