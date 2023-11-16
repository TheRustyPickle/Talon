use eframe::egui::Context;
use grammers_client::{Client, Config};
use grammers_session::Session;
use log::{debug, info};
use std::env;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::{self, Runtime};

use crate::tg_handler::{ProcessError, ProcessResult, ProcessStart};

#[derive(Clone)]
pub struct TGClient {
    client: Client,
    pub name: String,
    authorized: bool,
    sender: Sender<ProcessResult>,
    context: Context,
}

impl TGClient {
    fn new(
        client: Client,
        name: String,
        authorized: bool,
        sender: Sender<ProcessResult>,
        context: Context,
    ) -> Self {
        info!(
            "Creating a new TG Client with {name}. Currently authorized: {}",
            authorized
        );
        TGClient {
            client,
            name,
            sender,
            authorized,
            context,
        }
    }

    pub fn start_process(self, process_type: ProcessStart) {
        let runtime = get_runtime();

        let result = match process_type {
            ProcessStart::StartCount(start_chat, start_num, end_num) => {
                runtime.block_on(self.start_count(start_chat, start_num, end_num))
            }
        };

        if let Err(err) = result {
            match err {
                _ => self.send(ProcessResult::ProcessFailed(err)),
            }
        }
    }

    pub async fn start_count(
        &self,
        start_chat: String,
        start_num: Option<i32>,
        end_num: Option<i32>,
    ) -> Result<(), ProcessError> {
        if !self.is_authorized() {
            self.send(ProcessResult::UnauthorizedClient(self.name()));
            return Ok(());
        }
        let tg_chat = self.client.resolve_username(&start_chat).await;

        if tg_chat.is_err() {
            self.send(ProcessResult::InvalidChat(start_chat));
            return Ok(());
        }

        let tg_chat = tg_chat.unwrap().unwrap();

        let end_at = if let Some(num) = end_num { num } else { 0 };
        let mut start_at = if let Some(num) = start_num { num } else { -1 };

        let mut iter_message = self.client.iter_messages(tg_chat);

        while let Some(message) = iter_message.next().await.unwrap() {
            let message_num = message.id();
            debug!("Got message number: {}", message_num);
            if start_at == -1 {
                start_at = message_num
            }

            if message_num < end_at {
                break;
            }
            if message_num >= end_at {
                self.send(ProcessResult::CountingMessage(message, start_at, end_at));
            }

            // Sleep to prevent flood time being too noticeable/getting triggered
            if start_at - end_at > 3000 {
                sleep(Duration::from_millis(5))
            } else {
                sleep(Duration::from_millis(2))
            }
        }
        self.send(ProcessResult::CountingEnd);
        Ok(())
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    fn is_authorized(&self) -> bool {
        self.authorized
    }

    fn send(&self, data: ProcessResult) {
        self.sender.send(data).unwrap();
        self.context.request_repaint();
    }
}

pub fn start_tg_client(name: String, sender: Sender<ProcessResult>, context: Context) {
    let result = get_runtime().block_on(start_session(sender.clone(), name, context));

    if let Err(err) = result {
        sender.send(ProcessResult::ProcessFailed(err)).unwrap();
    };
}

async fn start_session(
    sender: Sender<ProcessResult>,
    name: String,
    context: Context,
) -> Result<(), ProcessError> {
    let api_id = env::var("API_ID").unwrap().parse().unwrap();
    let api_hash = env::var("API_HASH").unwrap();

    let client = Client::connect(Config {
        session: Session::load_file_or_create(&name)
            .map_err(|_| ProcessError::FileCreationError)?,
        api_id,
        api_hash,
        params: Default::default(),
    })
    .await
    .map_err(|_| ProcessError::InitialClientConnectionError(name.replace(".session", "to")))?;

    let authorized = client
        .is_authorized()
        .await
        .map_err(|_| ProcessError::InitialClientConnectionError(name.replace(".session", "to")))?;

    let new_client = TGClient::new(
        client,
        name.replace(".session", ""),
        authorized,
        sender.clone(),
        context.clone(),
    );
    new_client.send(ProcessResult::NewClient(new_client.clone()));
    Ok(())
}

fn get_runtime() -> Runtime {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
