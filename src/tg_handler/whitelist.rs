use grammers_client::types::PackedChat;
use log::error;
use std::collections::BTreeMap;

use crate::tg_handler::{ProcessError, ProcessResult, TGClient};
use crate::ui_components::processor::UnpackedWhitelistedUser;
use crate::utils::{get_whitelisted_users, save_whitelisted_users};

impl TGClient {
    /// Unpacks existing `PackedChat` hex string and sends it to the GUI
    pub async fn load_whitelisted_users(
        &self,
        client_list: BTreeMap<String, TGClient>,
    ) -> Result<(), ProcessError> {
        let packed_user_list = get_whitelisted_users();

        let mut chat_list = Vec::new();
        let mut failed_chat_num = 0;

        if let Some(packed_list) = packed_user_list {
            for user in packed_list {
                let packed_chat_result = PackedChat::from_hex(&user.hex_value);

                if let Ok(packed_chat) = packed_chat_result {
                    let seen_client = client_list.get(&user.seen_by);

                    let Some(target_client) = seen_client else {
                        failed_chat_num += 1;
                        error!("The target client is not available to unpack");
                        continue;
                    };

                    let chat = target_client.client().unpack_chat(packed_chat).await;

                    if let Ok(chat) = chat {
                        chat_list.push(UnpackedWhitelistedUser::new(chat, user.seen_by));
                    } else {
                        error!("Failed to unpack a chat");
                        failed_chat_num += 1;
                    }
                } else {
                    error!("Invalid chat hex found");
                    failed_chat_num += 1;
                }
            }
        } else {
            error!("Failed to deserialize a whitelist users json file. Deleting saved json data");
            save_whitelisted_users(Vec::new(), true);
            failed_chat_num = -1;
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
