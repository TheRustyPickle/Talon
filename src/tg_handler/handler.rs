use eframe::egui::Context;
use grammers_client::{Client, Config};
use grammers_session::Session;
use std::env;
use std::sync::mpsc::Sender;
use tokio::runtime::{self, Runtime};
use tracing::info;

use crate::tg_handler::{ProcessResult, ProcessStart};

#[derive(Clone)]
pub struct TGClient {
    client: Client,
    name: String,
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

        match process_type {
            ProcessStart::StartCount(start_chat, start_num, end_num) => {
                runtime.block_on(self.start_count(start_chat, start_num, end_num))
            }
        }
    }

    pub async fn start_count(
        &self,
        start_chat: String,
        start_num: Option<i32>,
        end_num: Option<i32>,
    ) {
        if !self.is_authorized() {
            self.send(ProcessResult::UnauthorizedClient);
            return;
        }
        let tg_chat = self.client.resolve_username(&start_chat).await;

        if tg_chat.is_err() {
            self.send(ProcessResult::InvalidChat);
            return;
        }

        let tg_chat = tg_chat.unwrap().unwrap();

        let end_at = if let Some(num) = end_num { num } else { 0 };

        let mut total_to_iter = if let Some(num) = start_num {
            num - end_at
        } else {
            0
        };

        let mut iter_message = self.client.iter_messages(tg_chat);

        if total_to_iter != 0 {
            iter_message = iter_message.limit(total_to_iter as usize)
        }

        while let Some(message) = iter_message.next().await.unwrap() {
            let message_num = message.id();
            if total_to_iter == 0 {
                total_to_iter = end_at as i32 - message_num
            }
            if message_num < end_at {
                break;
            }
            if message_num >= end_at {
                // TODO send data to GUI
            }
        }
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
    get_runtime().block_on(tg_handler(sender, name, context));
}

async fn tg_handler(sender: Sender<ProcessResult>, name: String, context: Context) {
    let api_id = env::var("API_ID").unwrap().parse().unwrap();
    let api_hash = env::var("API_HASH").unwrap();

    let client = Client::connect(Config {
        session: Session::load_file_or_create(&name).unwrap(),
        api_id,
        api_hash,
        params: Default::default(),
    })
    .await
    .unwrap();

    let authorized = client.is_authorized().await.unwrap();

    let new_client = TGClient::new(
        client,
        name.replace(".session", ""),
        authorized,
        sender.clone(),
        context.clone(),
    );
    new_client.send(ProcessResult::NewClient(new_client.clone()));
}

fn get_runtime() -> Runtime {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
