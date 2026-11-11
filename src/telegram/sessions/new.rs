use grammers_client::client::{LoginToken, PasswordToken};
use grammers_client::{Client, SignInError};
use grammers_mtsender::SenderPool;
use grammers_session::storages::{MemorySession, SqliteSession};
use log::{error, info};
use std::fs;
use std::sync::Arc;

use crate::telegram::events::StateUpdate;
use crate::telegram::{AccountEvent, EventSink, ProcessError, TGClient};
use crate::utils::get_api_keys;

/// Tries to send Telegram login code to the given phone number
pub async fn send_login_code(
    sink: EventSink,
    session_name: String,
    phone_number: String,
    is_temporary: bool,
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

    let pool = if is_temporary {
        let session = Arc::new(MemorySession::default());
        SenderPool::new(Arc::clone(&session), api_id)
    } else {
        let target_path = format!("{session_name}.sqlite");

        if fs::metadata(&target_path).is_ok() {
            info!("{target_path} exists. Removing old sqlite file");
            fs::remove_file(&target_path).expect("Failed to remove old sqlite file");
        }

        let session = SqliteSession::open(target_path).await;

        if let Err(e) = session {
            error!("Failed to create/open session file: {e}");
            return Err(ProcessError::FileCreationError);
        }

        let session = Arc::new(session.unwrap());

        SenderPool::new(Arc::clone(&session), api_id)
    };

    let SenderPool { runner, handle, .. } = pool;
    tokio::spawn(runner.run());

    let client = Client::new(handle);

    let code_token = client
        .request_login_code(&phone_number, &api_hash)
        .await
        .map_err(ProcessError::InvalidPhoneOrAPI)?;

    let new_client = TGClient::new(client, session_name, sink.clone(), is_temporary);

    sink.state(StateUpdate::LoginCodeSent {
        client: new_client,
        token: code_token,
    });

    sink.event(AccountEvent::LoginCodeSent);

    Ok(())
}

impl TGClient {
    /// Tries to sign in to a Telegram account with the given Telegram login code
    pub async fn sign_in_code(
        &self,
        token: Arc<LoginToken>,
        code: String,
    ) -> Result<(), ProcessError> {
        let result = self.client().sign_in(&token, &code).await;

        match result {
            Ok(_) => {
                self.finish_login();
            }
            Err(err) => match err {
                SignInError::InvalidCode => self.send(ProcessError::InvalidTGCode),
                SignInError::PasswordRequired(token) => {
                    self.sink().state(StateUpdate::PasswordRequired(token));
                    self.send(AccountEvent::PasswordRequired);
                }
                SignInError::SignUpRequired => self.send(ProcessError::NotSignedUp),
                SignInError::InvalidPassword(_) => unreachable!(),
                SignInError::Other(e) => self.send(ProcessError::UnknownError(e)),
            },
        }
        Ok(())
    }

    /// Tries to sign in to a Telegram account with the given login password.
    /// May continue to fail if the first time password was not correct
    pub async fn sign_in_password(
        &self,
        token: PasswordToken,
        password: String,
    ) -> Result<(), ProcessError> {
        let result = self.client().check_password(token, password).await;

        match result {
            Ok(_) => {
                self.finish_login();
            }
            Err(err) => match err {
                SignInError::InvalidCode
                | SignInError::PasswordRequired(_)
                | SignInError::SignUpRequired => unreachable!(),
                SignInError::InvalidPassword(new_token) => {
                    self.sink().state(StateUpdate::PasswordRejected(new_token));
                    self.send(ProcessError::InvalidPassword);
                }
                SignInError::Other(e) => self.send(ProcessError::UnknownError(e)),
            },
        }

        Ok(())
    }

    fn finish_login(&self) {
        self.sink().state(StateUpdate::LoggedIn(self.clone()));
        self.send(AccountEvent::LoggedIn(self.name()));
    }
}
