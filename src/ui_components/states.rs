use std::fmt::{self, Display};

#[derive(PartialEq)]
pub enum TabState {
    Counter,
    UserTable,
    Charts,
    Whitelist,
    Session,
}

pub enum ProcessState {
    Idle,
    InitialClientConnectionSuccessful(String),
    Counting(u8),
    InvalidStartChat,
    UnmatchedChat,
    SmallerStartNumber,
    DataCopied(i32),
    AuthorizationError,
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
    PasswordRequired,
    FloodWait,
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
            ProcessState::InitialClientConnectionSuccessful(name) => {
                write!(f, "Status: Connection successful with session: {}", name)
            }
            ProcessState::Counting(count) => {
                write!(f, "Status: Checking messages")?;
                for _ in 0..*count {
                    write!(f, ".")?;
                }
                Ok(())
            }
            ProcessState::InvalidStartChat => write!(f, "Status: Could not detect chat name"),
            ProcessState::UnmatchedChat => write!(f, "Status: Start and end chat names must match"),
            ProcessState::SmallerStartNumber => write!(
                f,
                "Status: Start message number must be greater than the ending message number"
            ),
            ProcessState::DataCopied(num) => {
                write!(f, "Status: Table data copied. Total cells: {}", num)
            }
            ProcessState::AuthorizationError => write!(
                f,
                "Status: Could not connect to the session. Are your API keys valid?"
            ),
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
            ProcessState::LogInWithCode => write!(f, "Status: Trying to login to the session with the code"),
            ProcessState::LogInWithPassword => write!(f, "Trying to login to the session with the password"),
            ProcessState::LoggedIn(name) => write!(f, "Status: Logged in session {name}"),
            ProcessState::InvalidTGCode => write!(f, "Status: Invalid TG Code given"),
            ProcessState::InvalidPassword => write!(f, "Status: Invalid password given"),
            ProcessState::NotSignedUp => write!(f, "Status: Account not signed up with this phone number"),
            ProcessState::UnknownError => write!(f, "Status: Unknown error acquired"),
            ProcessState::EmptySelectedSession => write!(f, "Status: No session is selected. Create a new session from the Session tab"),
            ProcessState::InvalidPhoneOrAPI => write!(f, "Status: Unknown error acquired. Possibly invalid phone number given or API keys are invalid"),
            ProcessState::PasswordRequired => write!(f, "Status: Account requires a password authentication"),
            ProcessState::FloodWait => write!(f, "Status: Flood wait triggered. Will resume again soon"),
        }
    }
}

#[derive(Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Debug)]
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
    Whitelisted,
}

impl ColumnName {
    pub fn get_next(&self) -> Self {
        match self {
            ColumnName::Name => ColumnName::Username,
            ColumnName::Username => ColumnName::UserID,
            ColumnName::UserID => ColumnName::TotalMessage,
            ColumnName::TotalMessage => ColumnName::TotalWord,
            ColumnName::TotalWord => ColumnName::TotalChar,
            ColumnName::TotalChar => ColumnName::AverageWord,
            ColumnName::AverageWord => ColumnName::AverageChar,
            ColumnName::AverageChar => ColumnName::Whitelisted,
            ColumnName::Whitelisted => ColumnName::Name,
        }
    }
}
