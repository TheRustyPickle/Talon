use chrono::{NaiveDate, NaiveDateTime};
use log::info;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::runtime::{self, Runtime};

use crate::ui_components::processor::ChartTiming;
use crate::ui_components::TGKeys;

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

pub fn get_api_keys() -> Option<TGKeys> {
    let mut to_return = None;
    let mut api_key_path = PathBuf::from(".");
    api_key_path.push("api_keys.json");

    let file = File::open(api_key_path);

    if let Ok(mut file) = file {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");

        let result = serde_json::from_str::<TGKeys>(&contents);
        if let Ok(result) = result {
            if !result.api_id.is_empty() || !result.api_hash.is_empty() {
                to_return = Some(result)
            }
        }
    }

    to_return
}

pub fn save_api_keys(api_keys: &TGKeys) {
    let data = serde_json::to_string(api_keys);

    if let Ok(data) = data {
        let mut api_key_path = PathBuf::from(".");
        api_key_path.push("api_keys.json");
        let mut file = File::create(api_key_path).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    };
}

pub fn get_whitelisted_users() -> Vec<String> {
    let mut to_return = Vec::new();

    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push("whitelist.json");

    let file = File::open(whitelist_path);

    if let Ok(mut file) = file {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");
        to_return = serde_json::from_str(&contents).unwrap();
    }

    to_return
}

pub fn save_whitelisted_users(packed_chats: Vec<String>, overwrite: bool) {
    let mut existing_data: HashSet<String> = HashSet::new();

    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push("whitelist.json");

    let file = File::open(&whitelist_path);

    if !overwrite {
        if let Ok(mut file) = file {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Failed to read file");

            existing_data = serde_json::from_str(&contents).unwrap();
        }
    }

    existing_data.extend(packed_chats);

    let data = serde_json::to_string(&existing_data).unwrap();

    let mut file = File::create(whitelist_path).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

pub fn get_font_data() -> Option<(Vec<u8>, Vec<u8>)> {
    let mut gentium_font = PathBuf::from(".");
    let mut cjk_font = PathBuf::from(".");

    gentium_font.push("fonts");
    cjk_font.push("fonts");
    fs::create_dir_all(&gentium_font).unwrap();

    gentium_font.push("GentiumBookPlus-Regular.ttf");
    cjk_font.push("NotoSansCJK-Regular.ttc");

    let mut gentium_font_data = Vec::new();
    let mut cjk_font_data = Vec::new();

    if let Ok(mut file) = File::open(gentium_font) {
        file.read_to_end(&mut gentium_font_data)
            .expect("Failed to read file");
    } else {
        return None;
    }

    if let Ok(mut file) = File::open(cjk_font) {
        file.read_to_end(&mut cjk_font_data)
            .expect("Failed to read file");
    } else {
        return None;
    }

    Some((cjk_font_data, gentium_font_data))
}

pub fn days_in_month(month: u32, year: i32) -> i64 {
    let date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };

    date.signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days()
}

pub fn time_to_string(time: &NaiveDateTime, timing: &ChartTiming) -> String {
    match timing {
        ChartTiming::Hourly => time.to_string(),
        _ => time.format("%Y-%m-%d").to_string(),
    }
}
