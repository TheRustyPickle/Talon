use grammers_session::types::PeerRef;

use crate::ui::state::ListKind;

/// Everything the GUI can ask the Telegram side to do. Sent through [`Backend::send`]
pub enum Command {
    /// Connect to the saved session files and keep the ones that are still logged in
    ConnectSavedSessions(Vec<String>),
    /// Drop sessions that got logged out and connect the given saved sessions
    RefreshSessions { new_files: Vec<String> },
    RequestLoginCode {
        session: String,
        phone: String,
        temporary: bool,
    },
    /// Finish the login that is waiting for a code
    SubmitLoginCode { code: String },
    /// Finish the login that is waiting for the two-step verification password
    SubmitPassword { password: String },
    /// Check that the chat exists and find the range to count
    /// Verify every session still works, drop the ones that do not, then check the chat
    /// with `session` (or another working one if it was dropped)
    CheckChat {
        session: String,
        chat: String,
        start: Option<i32>,
        end: Option<i32>,
    },
    StartCount {
        session: String,
        chat: String,
        start: Option<i32>,
        end: Option<i32>,
        multi_session: bool,
    },
    /// Resolve the saved users of a list
    LoadList {
        kind: ListKind,
        session: String,
        peers: Vec<PeerRef>,
    },
    /// Look up a username and add it to a list
    AddListUser {
        kind: ListKind,
        session: String,
        username: String,
    },
}
