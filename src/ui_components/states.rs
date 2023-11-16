use std::fmt::{self, Display};

#[derive(PartialEq)]
pub enum TabState {
    Counter,
    UserTable,
    Charts,
    Whitelist,
    Session,
}

#[derive(Clone)]
pub enum ProcessState {
    Idle,
    InitialClientConnectionSuccessful(String),
    Counting(u8),
    InvalidStartChat,
    UnmatchedChat,
    SmallerStartNumber,
    DataCopied(i32),
    InitialConnectionFailed(String),
    FileCreationFailed,
    UnauthorizedClient(String),
    NonExistingChat(String),
}

impl ProcessState {
    pub fn next_dot(&self) -> Self {
        match self {
            ProcessState::Counting(num) => {
                let new_num = if num == &3 { 0 } else { num + 1 };
                ProcessState::Counting(new_num)
            }
            _ => self.clone(),
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
            ProcessState::InitialConnectionFailed(name) => write!(
                f,
                "Status: Could not connect to {name} session. Restart the app to try again"
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
