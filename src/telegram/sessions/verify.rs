use log::{error, info};

use crate::telegram::events::StateUpdate;
use crate::telegram::{AccountEvent, EventSink, ProcessError, TGClient, connect_to_session};

/// Returns the sessions that are still logged in.
///
/// Sessions that were logged out since they connected are removed from the backend and
/// reported to the GUI. Sessions that could not be reached are only left out of the result
/// and stay connected.
pub async fn verify_sessions(clients: Vec<TGClient>) -> Vec<TGClient> {
    let Some(sink) = clients.first().map(|client| client.sink().clone()) else {
        return Vec::new();
    };

    let mut usable = Vec::new();
    let mut dropped = Vec::new();

    for client in clients {
        match client.client().is_authorized().await {
            Ok(true) => usable.push(client),
            Ok(false) => {
                info!(
                    "Session {} is no longer logged in. Dropping it",
                    client.name()
                );
                dropped.push(client.name());
            }
            Err(e) => {
                error!(
                    "Could not check session {}, skipping it for now: {e}",
                    client.name()
                );
            }
        }
    }

    if !dropped.is_empty() {
        sink.state(StateUpdate::SessionsDropped(dropped.clone()));
        sink.event(AccountEvent::SessionsDropped(dropped));
    }

    usable
}

/// Drops sessions that got logged out and connects to saved sessions that are not connected yet
pub async fn refresh_sessions(
    sink: EventSink,
    clients: Vec<TGClient>,
    new_files: Vec<String>,
) -> Result<(), ProcessError> {
    let working = verify_sessions(clients).await.len();

    if new_files.is_empty() {
        sink.event(AccountEvent::SessionsRefreshed { working });
        return Ok(());
    }

    connect_to_session(sink, new_files).await
}
