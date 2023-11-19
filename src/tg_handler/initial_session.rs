use eframe::egui::Context;
use grammers_client::{Client, Config};
use grammers_session::Session;
use log::info;
use std::sync::mpsc::Sender;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::utils::get_api_keys;

pub async fn connect_to_session(
    sender: Sender<ProcessResult>,
    name: String,
    context: &Context,
) -> Result<(), ProcessError> {
    let api_data = get_api_keys().unwrap();

    let api_id = api_data.api_id.parse().unwrap();
    let api_hash = api_data.api_hash;

    let name_without_session = name.replace(".session", "");

    let client = Client::connect(Config {
        session: Session::load_file_or_create(&name)
            .map_err(|_| ProcessError::FileCreationError)?,
        api_id,
        api_hash,
        params: Default::default(),
    })
    .await
    .map_err(|_| ProcessError::AuthorizationError)?;

    info!("Connected to client {name_without_session} successfully");

    let authorized = client
        .is_authorized()
        .await
        .map_err(ProcessError::UnknownError)?;

    info!("Client authorization status: {}", authorized);

    if !authorized {
        sender
            .send(ProcessResult::UnauthorizedClient(name_without_session))
            .unwrap();
        context.request_repaint();
        return Ok(());
    }

    let new_client = TGClient::new(
        client,
        name_without_session,
        sender.clone(),
        context.clone(),
        false,
    );
    new_client.send(ProcessResult::InitialSessionSuccess(new_client.clone()));
    Ok(())
}
