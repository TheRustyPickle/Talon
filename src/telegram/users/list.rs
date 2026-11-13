use grammers_session::types::PeerRef;
use log::{error, info};

use crate::telegram::{ProcessError, TGClient, UsersEvent};
use crate::ui::state::{ListKind, UnpackedUser};

impl TGClient {
    /// Resolves saved `PeerRef` values and sends them to the GUI
    pub async fn load_list_users(
        &self,
        kind: ListKind,
        peer_refs: Vec<PeerRef>,
    ) -> Result<(), ProcessError> {
        info!(
            "Starting resolving {} peers by {}",
            kind.name(),
            self.name()
        );
        let mut peer_list = Vec::new();
        let mut failed_peer_num = 0;

        for peer_ref in peer_refs {
            match self.client().resolve_peer(peer_ref).await {
                Ok(peer) => peer_list.push(UnpackedUser::new(&peer, peer_ref, self.name())),
                Err(e) => {
                    error!("Failed to resolve a peer. Error: {e}");
                    failed_peer_num += 1;
                }
            }
        }

        self.send(UsersEvent::ListLoaded {
            kind,
            users: peer_list,
            failed: failed_peer_num,
        });

        Ok(())
    }

    /// Tries to get a Telegram user with the given username to add to a list
    pub async fn add_list_user(
        &self,
        kind: ListKind,
        chat_name: String,
    ) -> Result<(), ProcessError> {
        if !self.check_authorization().await? {
            return Ok(());
        }

        let tg_chat = self.check_username(&chat_name).await?;

        let peer_ref = match tg_chat.to_ref().await {
            Ok(Some(peer_ref)) => peer_ref,
            _ => {
                error!("Failed to get a reference of the peer");
                return Err(ProcessError::InvalidChat(chat_name));
            }
        };

        self.send(UsersEvent::ListAdded {
            kind,
            user: Box::new(UnpackedUser::new(&tg_chat, peer_ref, self.name())),
        });

        Ok(())
    }
}
