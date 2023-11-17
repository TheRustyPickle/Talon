use log::info;
use std::fs;
use tokio::runtime::{self, Runtime};

pub fn find_session_files() -> Vec<String> {
    let mut sessions = Vec::new();
    if let Ok(entires) = fs::read_dir(".") {
        for entry in entires {
            if let Ok(file_name) = entry.unwrap().file_name().into_string() {
                if file_name.ends_with("session") {
                    info!("Found existing session file {}", file_name);
                    sessions.push(file_name);
                }
            }
        }
    }
    sessions
}

pub fn parse_tg_chat(text: String) -> (Option<String>, Option<i32>) {
    if text.is_empty() {
        return (None, None);
    }
    let mut chat_name = None;
    let mut message_number = None;

    if text.contains("t.me") {
        let splitted_text = text.split_once("t.me/");
        if let Some((_first, second)) = splitted_text {
            // It will be either chat_name/number or chat_name
            if second.contains('/') {
                let group_data = second.split('/').collect::<Vec<&str>>();
                chat_name = Some(group_data[0].to_string());
                if let Ok(num) = group_data[1].parse() {
                    message_number = Some(num)
                }
            } else {
                chat_name = Some(second.to_string())
            }
        }
    } else if text.starts_with('@') {
        let splitted_text = text.split_once('@');
        if let Some((_first, second)) = splitted_text {
            chat_name = Some(second.to_string());
        }
    } else {
        chat_name = Some(text)
    }
    info!(
        "Parsed group name: {:?} message number: {:?}",
        chat_name, message_number
    );
    (chat_name, message_number)
}

pub fn get_theme_emoji(is_light_theme: bool) -> (String, String) {
    if is_light_theme {
        ("🌙".to_string(), "Switch to dark theme".to_string())
    } else {
        ("☀".to_string(), "Switch to light theme".to_string())
    }
}

pub fn get_runtime() -> Runtime {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
