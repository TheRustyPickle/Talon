use eframe::egui::Context;
use grammers_client::client::{LoginToken, PasswordToken};
use log::error;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use tokio::runtime::Runtime;

use crate::telegram::events::{Packet, StateUpdate};
use crate::telegram::{
    Command, Event, EventSink, ProcessError, TGClient, check_scan_sessions, connect_to_session,
    refresh_sessions, send_login_code,
};
use crate::utils::get_runtime;

/// A login that is waiting for the user to finish it
struct PendingLogin {
    client: TGClient,
    code_token: Arc<LoginToken>,
    password_token: Option<PasswordToken>,
}

/// Everything on the Telegram side that the GUI talks to.
pub struct Backend {
    runtime: Runtime,
    sink: EventSink,
    receiver: Receiver<Packet>,
    clients: BTreeMap<String, TGClient>,
    pending: Option<PendingLogin>,
    cancel_scan: Arc<AtomicBool>,
}

impl Backend {
    pub fn new(context: Context) -> Self {
        let (sender, receiver) = channel();

        Self {
            runtime: get_runtime(),
            sink: EventSink::new(sender, context),
            receiver,
            clients: BTreeMap::new(),
            pending: None,
            cancel_scan: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Next event for the GUI, if any. State changes that come before it are applied first
    pub fn poll(&mut self) -> Option<Event> {
        loop {
            match self.receiver.try_recv().ok()? {
                Packet::Event(event) => return Some(event),
                Packet::State(update) => self.apply(update),
            }
        }
    }

    fn apply(&mut self, update: StateUpdate) {
        match update {
            StateUpdate::ClientsRestored(clients) => {
                for client in clients {
                    self.clients.insert(client.name(), client);
                }
            }
            StateUpdate::LoginCodeSent { client, token } => {
                self.pending = Some(PendingLogin {
                    client,
                    code_token: Arc::new(token),
                    password_token: None,
                });
            }
            StateUpdate::PasswordRequired(token) | StateUpdate::PasswordRejected(token) => {
                if let Some(pending) = &mut self.pending {
                    pending.password_token = Some(token);
                }
            }
            StateUpdate::SessionsDropped(names) => {
                for name in names {
                    self.clients.remove(&name);
                }
            }
            StateUpdate::LoggedIn(client) => {
                self.pending = None;
                self.clients.insert(client.name(), client);
            }
        }
    }

    /// Names of the connected sessions in alphabetical order
    pub fn session_names(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    pub fn session_count(&self) -> usize {
        self.clients.len()
    }

    pub fn has_session(&self, name: &str) -> bool {
        self.clients.contains_key(name)
    }

    /// Forget a login that was started but not finished
    pub fn cancel_login(&mut self) {
        self.pending = None;
    }

    /// Ask the running count to stop at the next message
    pub fn cancel_scan(&self) {
        self.cancel_scan.store(true, Ordering::Release);
    }

    /// Must be called before a fresh scan, otherwise it would stop right away after a cancel
    pub fn reset_cancel(&self) {
        self.cancel_scan.store(false, Ordering::Relaxed);
    }

    /// Run any async job that does not report back through events
    pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        self.runtime.spawn(task);
    }

    /// Runs a job and reports its error to the GUI
    fn run(&self, task: impl Future<Output = Result<(), ProcessError>> + Send + 'static) {
        let sink = self.sink.clone();

        self.runtime.spawn(async move {
            if let Err(err) = task.await {
                error!("Error acquired while handling a command: {err:?}");
                sink.event(err);
            }
        });
    }

    /// Log out of the sessions that were created as temporary and wait for it to finish
    pub fn logout_temporary_sessions(&self) {
        let handles: Vec<_> = self
            .clients
            .values()
            .filter(|client| client.is_temporary())
            .cloned()
            .map(|client| self.runtime.spawn(async move { client.logout().await }))
            .collect();

        for handle in handles {
            let _ = self.runtime.block_on(handle);
        }
    }

    pub fn send(&mut self, command: Command) {
        match command {
            Command::ConnectSavedSessions(names) => {
                self.run(connect_to_session(self.sink.clone(), names));
            }
            Command::RefreshSessions { new_files } => {
                let clients: Vec<TGClient> = self.clients.values().cloned().collect();
                self.run(refresh_sessions(self.sink.clone(), clients, new_files));
            }
            Command::RequestLoginCode {
                session,
                phone,
                temporary,
            } => {
                self.run(send_login_code(
                    self.sink.clone(),
                    session,
                    phone,
                    temporary,
                ));
            }
            Command::SubmitLoginCode { code } => {
                let Some(pending) = &self.pending else {
                    return self.reject("no login is waiting for a code");
                };

                let client = pending.client.clone();
                let token = pending.code_token.clone();
                self.run(async move { client.sign_in_code(token, code).await });
            }
            Command::SubmitPassword { password } => {
                let Some(pending) = &mut self.pending else {
                    return self.reject("no login is waiting for a password");
                };

                let Some(token) = pending.password_token.take() else {
                    return self.reject("the password token is already in use");
                };

                let client = pending.client.clone();
                self.run(async move { client.sign_in_password(token, password).await });
            }
            Command::CheckChat {
                session,
                chat,
                start,
                end,
            } => {
                let clients: Vec<TGClient> = self.clients.values().cloned().collect();
                self.run(check_scan_sessions(clients, session, chat, start, end));
            }
            Command::StartCount {
                session,
                chat,
                start,
                end,
                multi_session,
            } => {
                let cancel = self.cancel_scan.clone();
                self.with_client(&session, |client| async move {
                    client
                        .start_count(chat, start, end, multi_session, cancel)
                        .await
                });
            }
            Command::LoadList {
                kind,
                session,
                peers,
            } => {
                self.with_client(&session, |client| async move {
                    client.load_list_users(kind, peers).await
                });
            }
            Command::AddListUser {
                kind,
                session,
                username,
            } => {
                self.with_client(&session, |client| async move {
                    client.add_list_user(kind, username).await
                });
            }
        }
    }

    fn with_client<F, Fut>(&self, session: &str, task: F)
    where
        F: FnOnce(TGClient) -> Fut,
        Fut: Future<Output = Result<(), ProcessError>> + Send + 'static,
    {
        match self.clients.get(session) {
            Some(client) => self.run(task(client.clone())),
            None => self.reject(&format!("session {session} is not connected")),
        }
    }

    fn reject(&self, reason: &str) {
        error!("Command rejected: {reason}");
        self.sink.event(ProcessError::UnknownSession);
    }
}
