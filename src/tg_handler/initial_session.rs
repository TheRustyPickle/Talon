use eframe::egui::Context;
use grammers_client::{Client, Config};
use grammers_session::Session;
use log::info;
use std::sync::mpsc::Sender;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::utils::get_api_keys;

/// Tries to establish a connection to the local Telegram session files
pub async fn connect_to_session(
    sender: Sender<ProcessResult>,
    names: Vec<String>,
    context: &Context,
) -> Result<(), ProcessError> {
    let api_data = get_api_keys().unwrap();

    let api_id = api_data.api_id.parse().unwrap();
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
            api_hash: api_hash.to_owned(),
            params: Default::default(),
        })
        .await;

        let client = if let Ok(client) = client {
            client
        } else {
            info!("Failed to connect to session {}", name_without_session);
            failed_session.push(name_without_session);
            continue;
        };

        info!("Connected to Session {name_without_session} successfully");

        let authorized = client.is_authorized().await;

        let authorized = if let Ok(authorized) = authorized {
            authorized
        } else {
            info!(
                "Failed to determine session authorization status {}",
                name_without_session
            );
            failed_session.push(name_without_session);
            continue;
        };

        info!(
            "Session {} authorization status: {}",
            name_without_session, authorized
        );

        if !authorized {
            failed_session.push(name_without_session);
            continue;
        }

        let new_client = TGClient::new(
            client,
            name_without_session.to_owned(),
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
