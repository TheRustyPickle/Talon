use eframe::egui::{Vec2, vec2};
use grammers_client::peer::Peer;
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use strum_macros::{Display as sDisplay, EnumIter};

#[derive(Default)]
pub enum AppState {
    #[default]
    LoadingFontsAPI,
    InputAPIKeys,
    InitializedUI,
}

#[derive(PartialEq, EnumIter, sDisplay)]
pub enum TabState {
    Counter,
    #[strum(to_string = "User Table")]
    UserTable,
    Charts,
    Whitelist,
    Blacklist,
    Session,
}

impl TabState {
    pub fn first_value() -> Self {
        TabState::Counter
    }

    pub fn window_size(&self) -> Vec2 {
        match self {
            TabState::Counter => vec2(550.0, 400.0),
            TabState::UserTable => vec2(1250.0, 700.0),
            TabState::Charts => vec2(1000.0, 700.0),
            TabState::Whitelist | TabState::Blacklist => vec2(550.0, 600.0),
            TabState::Session => vec2(550.0, 320.0),
        }
    }
}

pub enum ProcessState {
    Idle,
    InitialClientConnectionSuccessful(String),
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
    InvalidPhoneOrAPI,
    InvalidAPIKeys,
    PasswordRequired,
    FloodWait,
    UsersWhitelisted(usize),
    UsersBlacklisted(usize),
    LoadedWhitelistedUsers(usize, i32),
    LoadedBlacklistedUsers(usize, i32),
    FailedLoadWhitelistedUsers,
    FailedLoadBlacklistedUsers,
    WhitelistedUserRemoved(usize),
    BlacklistedUserRemoved(usize),
    AllWhitelistRemoved,
    AllBlacklistRemoved,
    AddedToWhitelist,
    AddedToBlacklist,
    LatestMessageLoadingFailed,
    DataExported(String),
}

impl ProcessState {
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

impl Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessState::Idle => write!(f, "Status: Idle"),
            ProcessState::InitialClientConnectionSuccessful(text) => {
                write!(f, "Status: {text}",)
            }
            ProcessState::Counting(count) => {
                write!(f, "Status: Checking messages")?;
                for _ in 0..*count {
                    write!(f, ".")?;
                }
                Ok(())
            }
            ProcessState::InvalidStartChat => {
                write!(f, "Status: Could not detect any valid chat details")
            }
            ProcessState::DataCopied => {
                write!(f, "Status: Selected table data copied.")
            }
            ProcessState::FileCreationFailed => {
                write!(f, "Status: Could not create the session file. Try again")
            }
            ProcessState::UnauthorizedClient(name) => write!(
                f,
                "Status: The session {name} is not authorized. Delete the session and create a new one"
            ),
            ProcessState::NonExistingChat(name) => {
                write!(f, "Status: The target chat {name} does not exist")
            }
            ProcessState::SendingTGCode => write!(f, "Status: Trying to send Telegram login code"),
            ProcessState::TGCodeSent => write!(f, "Status: Telegram code was sent"),
            ProcessState::LogInWithCode => {
                write!(f, "Status: Trying to login to the session with the code")
            }
            ProcessState::LogInWithPassword => {
                write!(f, "Trying to login to the session with the password")
            }
            ProcessState::LoggedIn(name) => write!(f, "Status: Logged in session {name}"),
            ProcessState::InvalidTGCode => write!(f, "Status: Invalid TG Code given"),
            ProcessState::InvalidPassword => write!(f, "Status: Invalid password given"),
            ProcessState::NotSignedUp => {
                write!(f, "Status: Account not signed up with this phone number")
            }
            ProcessState::UnknownError => write!(f, "Status: Unknown error acquired"),
            ProcessState::EmptySelectedSession => write!(
                f,
                "Status: No session is selected. Create a new session from the Session tab"
            ),
            ProcessState::InvalidPhoneOrAPI => write!(
                f,
                "Status: Unknown error acquired. Possibly invalid phone number given or API keys are invalid"
            ),
            ProcessState::InvalidAPIKeys => write!(
                f,
                "Status: Failed to parse saved API keys. Are the API keys valid?"
            ),
            ProcessState::PasswordRequired => {
                write!(f, "Status: Account requires a password authentication")
            }
            ProcessState::FloodWait => {
                write!(f, "Status: Flood wait triggered. Will resume again soon")
            }
            ProcessState::UsersWhitelisted(num) => write!(f, "Status: Whitelisted {num} users"),
            ProcessState::UsersBlacklisted(num) => write!(f, "Status: Blacklisted {num} users"),
            ProcessState::LoadedWhitelistedUsers(success, failed) => write!(
                f,
                "Status: Loaded {success} whitelisted users. Failed to load {failed} users"
            ),
            ProcessState::LoadedBlacklistedUsers(success, failed) => write!(
                f,
                "Status: Loaded {success} blacklisted users. Failed to load {failed} users"
            ),
            ProcessState::FailedLoadWhitelistedUsers => write!(
                f,
                "Status: Failed to load whitelisted users due to invalid saved data. Old data has been removed"
            ),
            ProcessState::FailedLoadBlacklistedUsers => write!(
                f,
                "Status: Failed to load blacklisted users due to invalid saved data. Old data has been removed"
            ),
            ProcessState::WhitelistedUserRemoved(num) => {
                write!(f, "Status: {num} whitelisted users removed")
            }
            ProcessState::BlacklistedUserRemoved(num) => {
                write!(f, "Status: {num} blacklisted users removed")
            }
            ProcessState::AllWhitelistRemoved => write!(f, "Status: All whitelisted users removed"),
            ProcessState::AllBlacklistRemoved => write!(f, "Status: All blacklisted users removed"),
            ProcessState::AddedToWhitelist => write!(f, "Status: User added to whitelist"),
            ProcessState::AddedToBlacklist => write!(f, "Status: User added to blacklist"),
            ProcessState::LatestMessageLoadingFailed => {
                write!(f, "Status: Failed to get the latest message")
            }
            ProcessState::DataExported(location) => {
                write!(f, "Status: Data exported to {location}")
            }
        }
    }
}

#[derive(EnumIter, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Copy)]
pub enum ColumnName {
    #[default]
    Name,
    Username,
    UserID,
    TotalMessage,
    TotalWord,
    TotalChar,
    AverageWord,
    AverageChar,
    FirstMessageSeen,
    LastMessageSeen,
    Whitelisted,
}

impl fmt::Display for ColumnName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ColumnName::Name => "Name",
            ColumnName::Username => "Username",
            ColumnName::UserID => "User ID",
            ColumnName::TotalMessage => "Total Message",
            ColumnName::TotalWord => "Total Word",
            ColumnName::TotalChar => "Total Char",
            ColumnName::AverageWord => "Average Word",
            ColumnName::AverageChar => "Average Char",
            ColumnName::FirstMessageSeen => "First Message Seen",
            ColumnName::LastMessageSeen => "Last Message Seen",
            ColumnName::Whitelisted => "Whitelisted",
        };
        write!(f, "{name}")
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ChartType {
    #[default]
    Message,
    ActiveUser,
    MessageWeekDay,
    ActiveUserWeekDay,
}

impl Display for ChartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartType::Message => write!(f, "Message"),
            ChartType::ActiveUser => write!(f, "Active User"),
            ChartType::MessageWeekDay => write!(f, "Message Weekday"),
            ChartType::ActiveUserWeekDay => write!(f, "Active User Weekday"),
        }
    }
}

#[derive(Default, PartialEq, Copy, Clone)]
pub enum ChartTiming {
    #[default]
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl Display for ChartTiming {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartTiming::Daily => write!(f, "Daily"),
            ChartTiming::Hourly => write!(f, "Hourly"),
            ChartTiming::Weekly => write!(f, "Weekly"),
            ChartTiming::Monthly => write!(f, "Monthly"),
        }
    }
}

/// Saved form of a `PeerRef`.
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct PackedWhitelistedUser {
    pub peer_id: i64,
    pub access_hash: i64,
    pub seen_by: String,
}

impl PackedWhitelistedUser {
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

pub struct UnpackedWhitelistedUser {
    pub name: String,
    pub username: String,
    pub id: i64,
    pub peer_ref: PeerRef,
    pub seen_by: String,
}

impl UnpackedWhitelistedUser {
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

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct PackedBlacklistedUser {
    pub peer_id: i64,
    pub access_hash: i64,
    pub seen_by: String,
}

impl PackedBlacklistedUser {
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

pub struct UnpackedBlacklistedUser {
    pub name: String,
    pub username: String,
    pub id: i64,
    pub peer_ref: PeerRef,
    pub seen_by: String,
}

impl UnpackedBlacklistedUser {
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

#[derive(EnumIter, Default, PartialEq, Copy, Clone)]
pub enum NavigationType {
    #[default]
    Day,
    Week,
    Month,
    Year,
}

impl Display for NavigationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavigationType::Day => write!(f, "Day"),
            NavigationType::Week => write!(f, "Week"),
            NavigationType::Month => write!(f, "Month"),
            NavigationType::Year => write!(f, "Year"),
        }
    }
}
