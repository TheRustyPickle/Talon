use eframe::egui::Context;
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

    pub async fn check_authorization(&self) -> Result<bool, ProcessError> {
        let authorized: bool = self
            .client()
            .is_authorized()
            .await
            .map_err(|_| ProcessError::InitialClientConnectionError(self.name()))?;

        info!("Client authorization status: {}", authorized);

        if !authorized {
            self.send(ProcessResult::UnauthorizedClient(self.name()));
            return Ok(false);
        }
        Ok(true)
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
