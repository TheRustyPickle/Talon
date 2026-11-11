use eframe::egui::Context;
use grammers_client::InvocationError;
use grammers_client::client::{LoginToken, PasswordToken};
use std::sync::mpsc::Sender;

use crate::telegram::{TGClient, TGCountData};
use crate::ui::state::{UnpackedBlacklistedUser, UnpackedWhitelistedUser};

/// Everything the Telegram side can tell the GUI, grouped by topic
pub enum Event {
    Account(AccountEvent),
    Scan(ScanEvent),
    Users(UsersEvent),
    Failed(ProcessError),
}

pub enum AccountEvent {
    SessionsRestored {
        connected: Vec<String>,
        failed: Vec<String>,
    },
    LoginCodeSent,
    PasswordRequired,
    LoggedIn(String),
    Unauthorized(String),
    /// Sessions that are no longer logged in and were removed
    SessionsDropped(Vec<String>),
    /// A refresh finished without any new session to connect
    SessionsRefreshed {
        working: usize,
    },
}

pub enum ScanEvent {
    /// The chat exists, this is the message range to count
    ChatChecked {
        chat: String,
        start: i32,
        end: i32,
        /// The sessions that passed the check and can share the work
        sessions: Vec<String>,
    },
    Message(Box<TGCountData>),
    Finished {
        end_at: i32,
        last_number: i32,
    },
    FloodWait,
}

pub enum UsersEvent {
    WhitelistLoaded {
        users: Vec<UnpackedWhitelistedUser>,
        failed: i32,
    },
    BlacklistLoaded {
        users: Vec<UnpackedBlacklistedUser>,
        failed: i32,
    },
    WhitelistAdded(Box<UnpackedWhitelistedUser>),
    BlacklistAdded(Box<UnpackedBlacklistedUser>),
}

#[derive(Debug)]
pub enum ProcessError {
    FileCreationError,
    InvalidTGCode,
    InvalidPassword,
    NotSignedUp,
    InvalidPhoneOrAPI(InvocationError),
    InvalidAPIKeys,
    InvalidChat(String),
    /// Every session was logged out or unreachable
    NoUsableSession,
    FailedLatestMessage,
    UnknownSession,
    UnknownError(InvocationError),
}

impl From<AccountEvent> for Event {
    fn from(event: AccountEvent) -> Self {
        Event::Account(event)
    }
}

impl From<ScanEvent> for Event {
    fn from(event: ScanEvent) -> Self {
        Event::Scan(event)
    }
}

impl From<UsersEvent> for Event {
    fn from(event: UsersEvent) -> Self {
        Event::Users(event)
    }
}

impl From<ProcessError> for Event {
    fn from(error: ProcessError) -> Self {
        Event::Failed(error)
    }
}

pub(super) enum StateUpdate {
    ClientsRestored(Vec<TGClient>),
    LoginCodeSent { client: TGClient, token: LoginToken },
    PasswordRequired(PasswordToken),
    PasswordRejected(PasswordToken),
    LoggedIn(TGClient),
    SessionsDropped(Vec<String>),
}

pub(super) enum Packet {
    Event(Event),
    State(StateUpdate),
}

/// The only way for async code to reach the GUI. Wakes the GUI up after every message
#[derive(Clone)]
pub struct EventSink {
    sender: Sender<Packet>,
    context: Context,
}

impl EventSink {
    pub(super) fn new(sender: Sender<Packet>, context: Context) -> Self {
        Self { sender, context }
    }

    pub fn event(&self, event: impl Into<Event>) {
        self.push(Packet::Event(event.into()));
    }

    pub(super) fn state(&self, update: StateUpdate) {
        self.push(Packet::State(update));
    }

    fn push(&self, packet: Packet) {
        let _ = self.sender.send(packet);
        self.context.request_repaint();
    }
}
