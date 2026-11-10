use grammers_session::types::PeerRef;
use log::{error, info};

use crate::telegram::{ProcessError, ProcessResult, TGClient};
use crate::ui::state::UnpackedBlacklistedUser;

impl TGClient {
    /// Resolves saved `PeerRef` values and sends them to the GUI
    pub async fn load_blacklisted_users(
        &self,
        peer_refs: Vec<PeerRef>,
    ) -> Result<(), ProcessError> {
        info!("Starting resolving peers by {}", self.name());
        let mut peer_list = Vec::new();
        let mut failed_peer_num = 0;

        for peer_ref in peer_refs {
            match self.client().resolve_peer(peer_ref).await {
                Ok(peer) => {
                    peer_list.push(UnpackedBlacklistedUser::new(&peer, peer_ref, self.name()))
                }
                Err(e) => {
                    error!("Failed to resolve a peer. Error: {e}");
                    failed_peer_num += 1;
                }
            }
        }

        self.send(ProcessResult::UnpackedBlacklist(peer_list, failed_peer_num));
        Ok(())
    }

    /// Tries to get a Telegram chat with the given chat name
    pub async fn new_blacklist(&self, chat_name: String) -> Result<(), ProcessError> {
        if !self.check_authorization().await? {
            return Ok(());
        }

        let tg_chat = self.check_username(&chat_name).await;

        let tg_chat = match tg_chat {
            Ok(chat) => chat,
            Err(e) => {
                self.send(e);
                return Ok(());
            }
        };

        let peer_ref = match tg_chat.to_ref().await {
            Ok(Some(peer_ref)) => peer_ref,
            _ => {
                error!("Failed to get a reference of the peer");
                self.send(ProcessResult::InvalidChat(chat_name));
                return Ok(());
            }
        };

        self.send(ProcessResult::BlackListUser(Box::new(
            UnpackedBlacklistedUser::new(&tg_chat, peer_ref, self.name()),
        )));

        Ok(())
    }
}
