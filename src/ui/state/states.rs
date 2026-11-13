use eframe::egui::{Color32, Vec2, Visuals, vec2};
use grammers_client::peer::Peer;
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display as fDisplay};
use strum_macros::{Display, EnumIter};

#[derive(Default)]
pub enum AppState {
    #[default]
    LoadingAPI,
    InputAPIKeys,
    InitializedUI,
}

#[derive(PartialEq, EnumIter, Display)]
pub enum TabState {
    #[strum(to_string = "Scan")]
    Counter,
    #[strum(to_string = "Users")]
    UserTable,
    Charts,
    Whitelist,
    Blacklist,
    #[strum(to_string = "Accounts")]
    Session,
}

impl TabState {
    pub fn window_size(&self) -> Vec2 {
        match self {
            TabState::Counter => vec2(550.0, 440.0),
            TabState::UserTable => vec2(1250.0, 700.0),
            TabState::Charts => vec2(1000.0, 700.0),
            TabState::Whitelist | TabState::Blacklist => vec2(550.0, 600.0),
            TabState::Session => vec2(550.0, 430.0),
        }
    }
}

pub enum ProcessState {
    Idle,
    ConnectingSessions,
    /// Connection summary and whether at least one session failed to connect
    InitialClientConnectionSuccessful(String, bool),
    Counting(u8),
    InvalidStartChat,
    DataCopied,
    FileCreationFailed,
    UnauthorizedClient(String),
    NonExistingChat(String),
    SendingTGCode,
    TGCodeSent,
    LogInWithCode,
    LogInWithPassword,
    InvalidTGCode,
    InvalidPassword,
    NotSignedUp,
    UnknownError,
    LoggedIn(String),
    EmptySelectedSession,
    /// Names of the sessions that were logged out and removed
    SessionsDropped(String),
    NoUsableSession,
    /// Sessions that are logged in after a refresh
    SessionsRefreshed(usize),
    InvalidPhoneOrAPI,
    InvalidAPIKeys,
    PasswordRequired,
    FloodWait,
    /// Users from the users table were added to a list
    UsersListed(ListKind, usize),
    /// Saved users were loaded: (list, loaded, failed)
    ListLoaded(ListKind, usize, i32),
    ListLoadFailed(ListKind),
    ListUsersRemoved(ListKind, usize),
    ListCleared(ListKind),
    ListUserAdded(ListKind),
    LatestMessageLoadingFailed,
    DataExported(String),
}

/// How the status bar should present a `ProcessState`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Idle,
    Busy,
    Success,
    Warning,
    Error,
}

impl Severity {
    pub fn dot_color(self, visuals: &Visuals) -> Color32 {
        match self {
            Severity::Idle => visuals.weak_text_color(),
            Severity::Busy => visuals.selection.bg_fill,
            Severity::Success => Color32::from_rgb(76, 175, 110),
            Severity::Warning => Color32::from_rgb(230, 160, 40),
            Severity::Error => visuals.error_fg_color,
        }
    }
}

impl ProcessState {
    pub fn severity(&self) -> Severity {
        match self {
            ProcessState::Idle => Severity::Idle,
            ProcessState::ConnectingSessions
            | ProcessState::Counting(_)
            | ProcessState::SendingTGCode
            | ProcessState::LogInWithCode
            | ProcessState::LogInWithPassword => Severity::Busy,
            ProcessState::DataCopied
            | ProcessState::TGCodeSent
            | ProcessState::LoggedIn(_)
            | ProcessState::UsersListed(..)
            | ProcessState::ListUsersRemoved(..)
            | ProcessState::ListCleared(_)
            | ProcessState::ListUserAdded(_)
            | ProcessState::DataExported(_) => Severity::Success,
            ProcessState::SessionsRefreshed(_) => Severity::Success,
            ProcessState::InitialClientConnectionSuccessful(_, failed) => {
                if *failed {
                    Severity::Warning
                } else {
                    Severity::Success
                }
            }
            ProcessState::ListLoaded(_, _, failed) => {
                if *failed > 0 {
                    Severity::Warning
                } else {
                    Severity::Success
                }
            }
            ProcessState::FloodWait
            | ProcessState::PasswordRequired
            | ProcessState::EmptySelectedSession
            | ProcessState::SessionsDropped(_) => Severity::Warning,
            ProcessState::InvalidStartChat
            | ProcessState::FileCreationFailed
            | ProcessState::UnauthorizedClient(_)
            | ProcessState::NoUsableSession
            | ProcessState::NonExistingChat(_)
            | ProcessState::InvalidTGCode
            | ProcessState::InvalidPassword
            | ProcessState::NotSignedUp
            | ProcessState::UnknownError
            | ProcessState::InvalidPhoneOrAPI
            | ProcessState::InvalidAPIKeys
            | ProcessState::ListLoadFailed(_)
            | ProcessState::LatestMessageLoadingFailed => Severity::Error,
        }
    }

    pub fn next_dot(&self) -> Self {
        match self {
            ProcessState::Counting(num) => {
                let new_num = if num == &3 { 0 } else { num + 1 };
                ProcessState::Counting(new_num)
            }
            _ => ProcessState::Counting(0),
        }
    }
}

impl fDisplay for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessState::Idle => write!(f, "Idle"),
            ProcessState::ConnectingSessions => write!(f, "Connecting to saved sessions..."),
            ProcessState::InitialClientConnectionSuccessful(text, _) => {
                write!(f, "{text}",)
            }
            ProcessState::Counting(count) => {
                write!(f, "Checking messages")?;
                for _ in 0..*count {
                    write!(f, ".")?;
                }
                Ok(())
            }
            ProcessState::InvalidStartChat => {
                write!(f, "Could not detect any valid chat details")
            }
            ProcessState::DataCopied => {
                write!(f, "Selected table data copied.")
            }
            ProcessState::FileCreationFailed => {
                write!(f, "Could not create the session file. Try again")
            }
            ProcessState::UnauthorizedClient(name) => write!(
                f,
                "The session {name} is not authorized. Delete the session and create a new one"
            ),
            ProcessState::NonExistingChat(name) => {
                write!(f, "The target chat {name} does not exist")
            }
            ProcessState::SendingTGCode => write!(f, "Trying to send Telegram login code"),
            ProcessState::TGCodeSent => write!(f, "Telegram code was sent"),
            ProcessState::LogInWithCode => {
                write!(f, "Trying to login to the session with the code")
            }
            ProcessState::LogInWithPassword => {
                write!(f, "Trying to login to the session with the password")
            }
            ProcessState::LoggedIn(name) => write!(f, "Logged in session {name}"),
            ProcessState::InvalidTGCode => write!(f, "Invalid TG Code given"),
            ProcessState::InvalidPassword => write!(f, "Invalid password given"),
            ProcessState::NotSignedUp => {
                write!(f, "Account not signed up with this phone number")
            }
            ProcessState::UnknownError => write!(f, "Unknown error acquired"),
            ProcessState::EmptySelectedSession => write!(
                f,
                "No session is selected. Log in to an account from the Accounts tab"
            ),
            ProcessState::SessionsDropped(names) => write!(
                f,
                "Skipped logged out sessions: {names}. Log in again from the Accounts tab"
            ),
            ProcessState::SessionsRefreshed(total) => {
                write!(f, "Sessions refreshed. {total} working")
            }
            ProcessState::NoUsableSession => write!(
                f,
                "No working session is available. Log in again from the Accounts tab"
            ),
            ProcessState::InvalidPhoneOrAPI => write!(
                f,
                "Unknown error acquired. Possibly invalid phone number given or API keys are invalid"
            ),
            ProcessState::InvalidAPIKeys => {
                write!(f, "Failed to parse saved API keys. Are the API keys valid?")
            }
            ProcessState::PasswordRequired => {
                write!(f, "Account requires a password authentication")
            }
            ProcessState::FloodWait => {
                write!(f, "Flood wait triggered. Will resume again soon")
            }
            ProcessState::UsersListed(kind, num) => {
                write!(f, "{} {num} users", kind.past_tense())
            }
            ProcessState::ListLoaded(kind, success, failed) => write!(
                f,
                "Loaded {success} {} users. Failed to load {failed} users",
                kind.adjective()
            ),
            ProcessState::ListLoadFailed(kind) => write!(
                f,
                "Failed to load {} users due to invalid saved data. Old data has been removed",
                kind.adjective()
            ),
            ProcessState::ListUsersRemoved(kind, num) => {
                write!(f, "{num} {} users removed", kind.adjective())
            }
            ProcessState::ListCleared(kind) => {
                write!(f, "All {} users removed", kind.adjective())
            }
            ProcessState::ListUserAdded(kind) => write!(f, "User added to {}", kind.name()),
            ProcessState::LatestMessageLoadingFailed => {
                write!(f, "Failed to get the latest message")
            }
            ProcessState::DataExported(location) => {
                write!(f, "Data exported to {location}")
            }
        }
    }
}

#[derive(EnumIter, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Copy, Display)]
pub enum ColumnName {
    #[default]
    Name,
    Username,
    #[strum(to_string = "User ID")]
    UserID,
    #[strum(to_string = "Total Message")]
    TotalMessage,
    #[strum(to_string = "Total Word")]
    TotalWord,
    #[strum(to_string = "Total Char")]
    TotalChar,
    #[strum(to_string = "Average Word")]
    AverageWord,
    #[strum(to_string = "Average Char")]
    AverageChar,
    #[strum(to_string = "First Message Seen")]
    FirstMessageSeen,
    #[strum(to_string = "Last Message Seen")]
    LastMessageSeen,
    Whitelisted,
}

#[derive(Default, PartialEq, Clone, Copy, EnumIter, Display)]
pub enum ChartType {
    #[default]
    #[strum(to_string = "Messages")]
    Message,
    #[strum(to_string = "Active Users")]
    ActiveUser,
    #[strum(to_string = "Messages by Weekday")]
    MessageWeekDay,
    #[strum(to_string = "Users by Weekday")]
    ActiveUserWeekDay,
}

impl ChartType {
    pub fn hover_text(&self) -> &'static str {
        match self {
            ChartType::Message => {
                "Total count of messages in the selected time frame (e.g., hourly, daily, weekly)."
            }
            ChartType::ActiveUser => {
                "Total count of active users in the selected time frame (e.g., hourly, daily, weekly)."
            }
            ChartType::MessageWeekDay => "Total count of messages for each day of the week.",
            ChartType::ActiveUserWeekDay => "Total count of active users for each day of the week.",
        }
    }
}

#[derive(Default, PartialEq, Copy, Clone, EnumIter, Display)]
pub enum ChartTiming {
    #[default]
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// The two user lists that share the same behavior
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListKind {
    Whitelist,
    Blacklist,
}

impl ListKind {
    pub fn name(self) -> &'static str {
        match self {
            ListKind::Whitelist => "whitelist",
            ListKind::Blacklist => "blacklist",
        }
    }

    pub fn adjective(self) -> &'static str {
        match self {
            ListKind::Whitelist => "whitelisted",
            ListKind::Blacklist => "blacklisted",
        }
    }

    pub fn past_tense(self) -> &'static str {
        match self {
            ListKind::Whitelist => "Whitelisted",
            ListKind::Blacklist => "Blacklisted",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            ListKind::Whitelist => "whitelist_v2.json",
            ListKind::Blacklist => "blacklist_v2.json",
        }
    }
}

/// Saved form of a `PeerRef`.
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct PackedUser {
    pub peer_id: i64,
    pub access_hash: i64,
    pub seen_by: String,
}

impl PackedUser {
    /// Returns `None` if the peer has no id that can be saved
    pub fn new(peer: PeerRef, seen_by: String) -> Option<Self> {
        Some(Self {
            peer_id: peer.id.bot_api_dialog_id()?,
            access_hash: peer.auth.hash(),
            seen_by,
        })
    }

    pub fn peer_ref(&self) -> Option<PeerRef> {
        unpack_peer_ref(self.peer_id, self.access_hash)
    }
}

pub struct UnpackedUser {
    pub name: String,
    pub username: String,
    pub id: i64,
    pub peer_ref: PeerRef,
    pub seen_by: String,
}

impl UnpackedUser {
    pub fn new(peer: &Peer, peer_ref: PeerRef, seen_by: String) -> Self {
        let (name, username, id) = peer_details(peer);
        Self {
            name,
            username,
            id,
            peer_ref,
            seen_by,
        }
    }
}

/// Rebuilds a `PeerRef` from the saved integers
fn unpack_peer_ref(peer_id: i64, access_hash: i64) -> Option<PeerRef> {
    Some(PeerRef {
        id: PeerId::from_bot_api_dialog_id(peer_id)?,
        auth: PeerAuth::from_hash(access_hash),
    })
}

/// Returns the name, username and bare id of a peer
pub fn peer_details(peer: &Peer) -> (String, String, i64) {
    // As per grammers lib doc, empty name can be given if it's a deleted account
    let name = match peer {
        Peer::User(user) => user.full_name(),
        _ => peer.name().unwrap_or_default().to_string(),
    };
    let name = if name.is_empty() {
        "Deleted Account".to_string()
    } else {
        name
    };

    let username = peer.username().unwrap_or("Empty").to_string();

    (name, username, peer.id().bare_id().unwrap_or(0))
}

#[derive(EnumIter, Default, PartialEq, Copy, Clone, Display)]
pub enum NavigationType {
    #[default]
    Day,
    Week,
    Month,
    Year,
}
