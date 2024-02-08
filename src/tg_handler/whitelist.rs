use grammers_client::types::PackedChat;
use log::{error, info};

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::ui_components::processor::UnpackedWhitelistedUser;

impl TGClient {
    /// Unpacks existing `PackedChat` hex string and sends it to the GUI
    pub async fn load_whitelisted_users(&self, hex_data: Vec<String>) -> Result<(), ProcessError> {
        info!("Starting unpacking chat by {}", self.name());
        let mut chat_list = Vec::new();
        let mut failed_chat_num = 0;

        for hex in hex_data {
            let packed_chat_result = PackedChat::from_hex(&hex);

            if let Ok(packed_chat) = packed_chat_result {
                let chat = self.client().unpack_chat(packed_chat).await;
                match chat {
                    Ok(chat) => chat_list.push(UnpackedWhitelistedUser::new(chat, self.name())),
                    Err(e) => {
                        error!("Failed to unpack a chat. Error: {e}");
                        failed_chat_num += 1;
                    }
                }
            } else {
                error!("Invalid chat hex found");
                failed_chat_num += 1;
            }
        }

        self.send(ProcessResult::UnpackedChats(chat_list, failed_chat_num));
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
        self.send(ProcessResult::WhiteListUser(UnpackedWhitelistedUser::new(
            tg_chat,
            self.name(),
        )));
        Ok(())
    }
}
