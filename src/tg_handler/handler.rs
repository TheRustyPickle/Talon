use eframe::egui::Context;
use grammers_client::types::Chat;
use grammers_client::Client;
use log::{error, info};
use std::sync::mpsc::Sender;

use crate::tg_handler::{
    connect_to_session, send_login_code, NewProcess, ProcessError, ProcessResult, ProcessStart,
};
use crate::utils::get_runtime;

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

    pub fn start_process(self, process_type: ProcessStart) {
        let runtime = get_runtime();

        let result = match process_type {
            ProcessStart::StartCount(start_chat, start_num, end_num) => {
                runtime.block_on(self.start_count(start_chat, start_num, end_num))
            }
            ProcessStart::SignInCode(token, code) => {
                runtime.block_on(self.sign_in_code(token, code))
            }
            ProcessStart::SignInPasswords(token, password) => {
                runtime.block_on(self.sign_in_password(token, password))
            }
            ProcessStart::SessionLogout => runtime.block_on(self.logout()),
            ProcessStart::LoadWhitelistedUsers => runtime.block_on(self.load_whitelisted_users()),
            ProcessStart::NewWhitelistUser(name) => runtime.block_on(self.new_whitelist(name)),
        };

        if let Err(err) = result {
            error!("Error acquired while handing a process: {err:?}");
            self.send(ProcessResult::ProcessFailed(err));
        }
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

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

    pub async fn check_authorization(&self) -> Result<bool, ProcessError> {
        let authorized: bool = self
            .client()
            .is_authorized()
            .await
            .map_err(ProcessError::UnknownError)?;

        info!("Client authorization status: {}", authorized);

        if !authorized {
            self.send(ProcessResult::UnauthorizedClient(self.name()));
            return Ok(false);
        }
        Ok(true)
    }

    pub async fn check_username(&self, chat_name: &str) -> Result<Chat, ProcessResult> {
        let tg_chat = self.client().resolve_username(chat_name).await;

        let tg_chat = if let Ok(chat) = tg_chat {
            chat
        } else {
            error!("Failed to resolve username");
            return Err(ProcessResult::InvalidChat(chat_name.to_owned()));
        };

        let tg_chat = if let Some(chat) = tg_chat {
            chat
        } else {
            error!("Found None value for target chat. Stopping processing");
            return Err(ProcessResult::InvalidChat(chat_name.to_owned()));
        };

        info!("Target chat {} exist", tg_chat.name());

        Ok(tg_chat)
    }

    pub async fn logout(&self) -> Result<(), ProcessError> {
        let _ = self.client().sign_out().await;
        Ok(())
    }
}

pub fn start_process(process: NewProcess, sender: Sender<ProcessResult>, context: Context) {
    let runtime = get_runtime();
    let result = match process {
        NewProcess::InitialSessionConnect(name) => {
            runtime.block_on(connect_to_session(sender.clone(), name, &context))
        }
        NewProcess::SendLoginCode(session_name, phone_number, is_temporary) => {
            runtime.block_on(send_login_code(
                sender.clone(),
                &context,
                session_name,
                phone_number,
                is_temporary,
            ))
        }
    };

    if let Err(err) = result {
        error!("Error acquired while handing a process: {err:?}");
        sender.send(ProcessResult::ProcessFailed(err)).unwrap();
        context.request_repaint()
    }
}
