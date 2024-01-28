use grammers_client::types::PackedChat;
use log::error;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::utils::get_whitelisted_users;

impl TGClient {
    /// Unpacks existing `PackedChat` hex string and sends it to the GUI
    pub async fn load_whitelisted_users(&self) -> Result<(), ProcessError> {
        let packed_user_list = get_whitelisted_users();

        let mut chat_list = Vec::new();

        for user in packed_user_list {
            let packed_chat_result = PackedChat::from_hex(&user);

            if let Ok(packed_chat) = packed_chat_result {
                let chat = self.client().unpack_chat(packed_chat).await;

                if let Ok(chat) = chat {
                    chat_list.push(chat);
                } else {
                    error!("Failed to unpack a chat");
                }
            } else {
                error!("Invalid chat hex found");
            }
        }
        self.send(ProcessResult::UnpackedChats(chat_list));

        Ok(())
    }

    /// Tries to get a Telegram chat with the given chat name
    pub async fn new_whitelist(&self, chat_name: String) -> Result<(), ProcessError> {
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
        self.send(ProcessResult::WhiteListUser(tg_chat));
        Ok(())
    }
}
