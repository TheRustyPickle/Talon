use eframe::egui::Context;
use grammers_client::types::{LoginToken, PasswordToken};
use grammers_client::{Client, Config, FixedReconnect, InitParams, SignInError};
use grammers_session::Session;
use log::{error, info};
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::utils::get_api_keys;

/// Tries to send Telegram login code to the given phone number
pub async fn send_login_code(
    sender: Sender<ProcessResult>,
    context: &Context,
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

    let session = if is_temporary {
        Session::new()
    } else {
        let target_path = format!("./{session_name}.session");

        if fs::metadata(&target_path).is_ok() {
            info!("{target_path} exists. Removing old session file");
            fs::remove_file(&target_path).unwrap();
        }

        Session::load_file_or_create(target_path).unwrap()
    };

    let reconnection = Box::leak(Box::new(FixedReconnect {
        attempts: 10,
        delay: std::time::Duration::from_secs(1),
    }));

    let client = Client::connect(Config {
        session,
        api_id,
        api_hash,
        params: InitParams {
            reconnection_policy: reconnection,
            update_queue_limit: Some(1),
            ..Default::default()
        },
    })
    .await
    .map_err(|_| ProcessError::AuthorizationError)?;

    let code_token = client
        .request_login_code(&phone_number)
        .await
        .map_err(ProcessError::InvalidPhoneOrAPI)?;

    let new_client = TGClient::new(client, session_name, sender, context.clone(), is_temporary);

    new_client.send(ProcessResult::LoginCodeSent(code_token, new_client.clone()));
    Ok(())
}

impl TGClient {
    /// Tries to sign in to a Telegram account with the given Telegram login code
    pub async fn sign_in_code(
        &self,
        token: Arc<Mutex<LoginToken>>,
        code: String,
    ) -> Result<(), ProcessError> {
        let token = token.lock().await;

        let result = self.client().sign_in(&token, &code).await;

        match result {
            Ok(_) => {
                if !self.is_temporary() {
                    info!("Saving session data to a file");
                    let target_path = format!("./{}.session", self.name());
                    self.client().session().save_to_file(target_path).unwrap();
                }
                self.send(ProcessResult::LoggedIn(self.name()));
            }
            Err(err) => match err {
                SignInError::InvalidCode => {
                    self.send(ProcessResult::ProcessFailed(ProcessError::InvalidTGCode));
                }
                SignInError::PasswordRequired(token) => {
                    self.send(ProcessResult::PasswordRequired(Box::new(token)));
                }
                SignInError::SignUpRequired {
                    terms_of_service: _,
                } => self.send(ProcessResult::ProcessFailed(ProcessError::NotSignedUp)),
                SignInError::InvalidPassword => unreachable!(),
                SignInError::Other(e) => {
                    self.send(ProcessResult::ProcessFailed(ProcessError::UnknownError(e)));
                }
            },
        }
        Ok(())
    }

    /// Tries to sign in to a Telegram account with the given login password.
    /// May continue to fail if the first time password was not correct
    pub async fn sign_in_password(
        &self,
        token: Arc<Mutex<PasswordToken>>,
        password: String,
    ) -> Result<(), ProcessError> {
        let token = token.lock().await.clone();

        let result = self.client().check_password(token, password).await;

        match result {
            Ok(_) => {
                if !self.is_temporary() {
                    info!("Saving session data to a file");
                    let target_path = format!("./{}.session", self.name());
                    self.client().session().save_to_file(target_path).unwrap();
                }
                self.send(ProcessResult::LoggedIn(self.name()));
            }
            Err(err) => match err {
                SignInError::InvalidCode
                | SignInError::PasswordRequired(_)
                | SignInError::SignUpRequired {
                    terms_of_service: _,
                } => unreachable!(),
                SignInError::InvalidPassword => {
                    self.send(ProcessResult::ProcessFailed(ProcessError::InvalidPassword));
                }
                SignInError::Other(e) => {
                    self.send(ProcessResult::ProcessFailed(ProcessError::UnknownError(e)));
                }
            },
        }

        Ok(())
    }
}
