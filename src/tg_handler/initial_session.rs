use eframe::egui::Context;
use grammers_client::{Client, Config};
use grammers_session::Session;
use log::{error, info};
use std::sync::mpsc::Sender;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::utils::get_api_keys;

/// Tries to establish a connection to the local Telegram session files
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
    let api_hash = api_data.api_hash;

    let mut successful_session = Vec::new();
    let mut failed_session = Vec::new();

    let mut all_clients = Vec::new();

    for name in names {
        let name_without_session = name.replace(".session", "");

        let client = Client::connect(Config {
            session: Session::load_file_or_create(&name)
                .map_err(|_| ProcessError::FileCreationError)?,
            api_id,
            api_hash: api_hash.clone(),
            params: Default::default(),
        })
        .await;

        let Ok(client) = client else {
            info!("Failed to connect to session {name_without_session}");
            failed_session.push(name_without_session);
            continue;
        };

        info!("Connected to Session {name_without_session} successfully");

        let authorized = client.is_authorized().await;

        let Ok(authorized) = authorized else {
            info!("Failed to determine session authorization status {name_without_session}");
            failed_session.push(name_without_session);
            continue;
        };

        info!("Session {name_without_session} authorization status: {authorized}");

        if !authorized {
            failed_session.push(name_without_session);
            continue;
        }

        let new_client = TGClient::new(
            client,
            name_without_session.clone(),
            sender.clone(),
            context.clone(),
            false,
        );
        all_clients.push(new_client);
        successful_session.push(name_without_session);
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
