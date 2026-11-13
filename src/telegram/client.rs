use grammers_client::{Client, peer::Peer};
use log::{error, info};

use crate::telegram::{AccountEvent, Event, EventSink, ProcessError};

/// A connected Telegram account. Created and kept by the [`Backend`]
#[derive(Clone)]
pub struct TGClient {
    client: Client,
    name: String,
    sink: EventSink,
    is_temporary: bool,
}

impl TGClient {
    pub fn new(client: Client, name: String, sink: EventSink, is_temporary: bool) -> Self {
        TGClient {
            client,
            name,
            sink,
            is_temporary,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Tell the GUI something happened
    pub fn send(&self, event: impl Into<Event>) {
        self.sink.event(event);
    }

    pub fn sink(&self) -> &EventSink {
        &self.sink
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn is_temporary(&self) -> bool {
        self.is_temporary
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
            self.send(AccountEvent::Unauthorized(self.name()));
            return Ok(false);
        }

        Ok(true)
    }

    /// Tries to resolve a username to get a Telegram chat account
    pub async fn check_username(&self, chat_name: &str) -> Result<Peer, ProcessError> {
        let tg_chat = self.client().resolve_username(chat_name).await;

        let Ok(tg_chat) = tg_chat else {
            error!("Failed to resolve username");
            return Err(ProcessError::InvalidChat(chat_name.to_owned()));
        };

        let Some(tg_chat) = tg_chat else {
            error!("Found None value for target chat. Stopping processing");
            return Err(ProcessError::InvalidChat(chat_name.to_owned()));
        };

        let name = tg_chat.name().unwrap_or("No name");

        info!("Target chat {} exist", name);

        Ok(tg_chat)
    }

    /// Logs out of the client
    pub async fn logout(&self) -> Result<(), ProcessError> {
        let _ = self.client().sign_out().await;
        info!("Logged out of client {}", self.name());
        Ok(())
    }
}
