use std::fmt::{self, Display};

#[derive(PartialEq)]
pub enum TabState {
    Counter,
    UserTable,
    Charts,
    Whitelist,
    Sessions,
}

pub enum ProcessState {
    Idle,
    Counting(u8),
    InvalidStartChat,
    UnmatchedChat,
    SmallerStartNumber,
}

impl ProcessState {
    pub fn next_dot(&self) -> Self {
        match self {
            ProcessState::Idle => ProcessState::Idle,
            ProcessState::Counting(num) => {
                let new_num = if num == &3 { 0 } else { num + 1 };
                ProcessState::Counting(new_num)
            }
            ProcessState::InvalidStartChat => ProcessState::InvalidStartChat,
            ProcessState::UnmatchedChat => ProcessState::UnmatchedChat,
            ProcessState::SmallerStartNumber => ProcessState::SmallerStartNumber,
        }
    }
}

impl Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessState::Idle => write!(f, "Status: Idle"),
            ProcessState::Counting(count) => {
                write!(f, "Status: Checking Messages")?;
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
        }
    }
}
