use eframe::egui::Context;
use grammers_client::{Client, SenderPool};
use grammers_session::storages::SqliteSession;
use log::{error, info};
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::utils::get_api_keys;

/// Tries to establish a connection to the local Telegram sqlite files
pub async fn connect_to_session(
    sender: Sender<ProcessResult>,
    names: Vec<String>,
    context: &Context,
) -> Result<(), ProcessError> {
    let Some(api_data) = get_api_keys() else {
        error!("No API keys found");
        return Err(ProcessError::InvalidAPIKeys);
    };

    let Ok(api_id) = api_data.api_id.parse() else {
        error!("Failed to parse API ID. Given API ID: {}", api_data.api_id);
        return Err(ProcessError::InvalidAPIKeys);
    };

    let mut successful_session = Vec::new();
    let mut failed_session = Vec::new();

    let mut all_clients = Vec::new();

    for name in names {
        let name_without_sqlite = name.replace(".sqlite", "");

        let session = SqliteSession::open(name).await;

        if let Err(e) = session {
            error!("Failed to open session file: {e}");
            failed_session.push(name_without_sqlite);
            continue;
        }

        let session = Arc::new(session.unwrap());

        let pool = SenderPool::new(Arc::clone(&session), api_id);

        let client = Client::new(pool.handle);

        info!("Connected to Session {name_without_sqlite} successfully");

        let authorized = client.is_authorized().await;

        let Ok(authorized) = authorized else {
            info!("Failed to determine session authorization status {name_without_sqlite}");
            failed_session.push(name_without_sqlite);
            continue;
        };

        info!("Session {name_without_sqlite} authorization status: {authorized}");

        if !authorized {
            failed_session.push(name_without_sqlite);
            continue;
        }

        let new_client = TGClient::new(
            client,
            name_without_sqlite.clone(),
            sender.clone(),
            context.clone(),
            false,
        );
        all_clients.push(new_client);
        successful_session.push(name_without_sqlite);
    }

    sender
        .send(ProcessResult::InitialSessionSuccess((
            all_clients,
            successful_session,
            failed_session,
        )))
        .unwrap();

    context.request_repaint();

    Ok(())
}
