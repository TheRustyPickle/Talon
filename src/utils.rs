use chrono::{Local, NaiveDate, NaiveDateTime};
use log::{error, info};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::runtime::{self, Runtime};

use crate::ui_components::processor::{
    ChartTiming, PackedBlacklistedUser, PackedWhitelistedUser, ParsedChat,
};
use crate::ui_components::tab_ui::UserRowData;
use crate::ui_components::TGKeys;

/// Finds all the saved session files
pub fn find_session_files() -> Vec<String> {
    let mut sessions = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries {
            if let Ok(file_name) = entry.unwrap().file_name().into_string() {
                if file_name.ends_with(".session") {
                    info!("Found existing session file {}", file_name);
                    sessions.push(file_name);
                }
            }
        }
    }
    sessions
}

/// Convert inserted chat points into textual representation
pub fn chat_to_text(start: &str, end: &str) -> String {
    let chat_data: BTreeMap<String, ParsedChat> =
        parse_chat_details(start, end).into_iter().collect();

    let mut text_data = "Detected Chats:".to_string();

    for (name, parsed) in chat_data {
        let mut chat_text = format!(" {name}");

        if let Some(end_point) = parsed.end_point() {
            chat_text += &format!(": {end_point} - ");

            if let Some(start_point) = parsed.start_point() {
                chat_text += &format!("{start_point}");
            } else {
                chat_text += "∞";
            }
        } else if let Some(start_point) = parsed.start_point() {
            chat_text += ": 1 - ";
            chat_text += &format!("{start_point}");
        } else {
            chat_text += ": ∞";
        }
        text_data += &format!(" {chat_text}");
    }

    text_data
}

/// Parse start and end point strings as parsed chat points
pub fn parse_chat_details(start: &str, end: &str) -> HashMap<String, ParsedChat> {
    let start_chat_list: Vec<&str> = start.split_whitespace().collect();
    let end_chat_list: Vec<&str> = end.split_whitespace().collect();

    let mut parsed_chat_list = HashMap::new();

    for chat in start_chat_list {
        let (name, num) = parse_tg_chat(chat);
        if name.is_none() {
            error!("{chat} is getting ignored as no chat name was found");
            continue;
        }

        let name = name.unwrap();
        let parsed = ParsedChat::new(name.clone(), num, None);
        parsed_chat_list.insert(name, parsed);
    }

    for chat in end_chat_list {
        let (name, num) = parse_tg_chat(chat);
        if name.is_none() {
            error!("{chat} is getting ignored as no chat name was found");
            continue;
        }

        let name = name.unwrap();
        if let Some(parsed) = parsed_chat_list.get_mut(&name) {
            if let Some(end_num) = num {
                let completed = parsed.set_end_point(end_num);
                if !completed {
                    error!("End point cannot be equal or bigger than start point. Ignoring the end point for {chat}");
                }
            }
        } else {
            error!("{chat} was not found in the start point, this will be ignored");
        }
    }

    parsed_chat_list
}

/// Tries to parse a link to get the chat name and the message ID
pub fn parse_tg_chat(text: &str) -> (Option<String>, Option<i32>) {
    if text.is_empty() {
        return (None, None);
    }

    let mut chat_name = None;
    let mut message_number = None;

    // Example t.me/chat_name/1234
    if text.contains("t.me") {
        // split expected result t.me and chat_name/1234
        let split_text = text.split_once("t.me/");
        if let Some((_first, second)) = split_text {
            // It will be either chat_name/number or chat_name
            // if chat_name/number split it again and get the number
            // otherwise set whatever value is remaining as the parsed chat name
            if second.contains('/') {
                (chat_name, message_number) = split_tg_link(second);
            } else {
                chat_name = Some(second.to_string());
            }
        }
    } else if text.starts_with('@') {
        // Example @chat_name/1234
        let split_text = text.split_once('@');
        // // split expected result @ and chat_name/1234
        if let Some((_first, second)) = split_text {
            if second.contains('/') {
                (chat_name, message_number) = split_tg_link(second);
            } else if !second.is_empty() {
                chat_name = Some(second.to_string());
            }
        }
    } else {
        // Example chat_name. If invalid, it will still be caught here, later will get verified by the telegram side
        if text.contains('/') {
            (chat_name, message_number) = split_tg_link(text);
        } else {
            chat_name = Some(text.to_string());
        };
    }
    (chat_name, message_number)
}

/// Splits a string on slash and tries to get the tg chat name and message number
fn split_tg_link(text: &str) -> (Option<String>, Option<i32>) {
    let mut chat_name = None;
    let mut message_number = None;

    if let Some((name, num)) = text.split_once('/') {
        chat_name = Some(name.to_string());
        if let Ok(num) = num.parse() {
            message_number = Some(num);
        }
    };

    (chat_name, message_number)
}

/// Returns the proper emoji based on light or dark value
pub fn theme_hover_text(is_light_theme: bool) -> String {
    if is_light_theme {
        "Switch to dark theme".to_string()
    } else {
        "Switch to light theme".to_string()
    }
}

/// Convenient tokio runtime getter
pub fn get_runtime() -> Runtime {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Tries to read the API key json file
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
            if !result.api_id.is_empty() && !result.api_hash.is_empty() {
                to_return = Some(result);
            }
        }
    }

    to_return
}

/// Saves the API keys in a json file
pub fn save_api_keys(api_keys: &TGKeys) {
    let data = serde_json::to_string(api_keys);

    if let Ok(data) = data {
        let mut api_key_path = PathBuf::from(".");
        api_key_path.push("api_keys.json");
        let mut file = File::create(api_key_path).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    };
}

/// Reads the whitelisted user `PackedChat` Hex IDs and returns them
pub fn get_whitelisted() -> Result<Vec<PackedWhitelistedUser>, Box<dyn Error>> {
    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push("whitelist.json");

    let file = File::open(whitelist_path);

    if let Ok(mut file) = file {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let users = serde_json::from_str(&contents)?;
        Ok(users)
    } else {
        Ok(Vec::new())
    }
}

/// Reads the blacklisted user `PackedChat` Hex IDs and returns them
pub fn get_blacklisted() -> Result<Vec<PackedBlacklistedUser>, Box<dyn Error>> {
    let mut blacklist_path = PathBuf::from(".");
    blacklist_path.push("blacklist.json");

    let file = File::open(blacklist_path);

    if let Ok(mut file) = file {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let users = serde_json::from_str(&contents)?;
        Ok(users)
    } else {
        Ok(Vec::new())
    }
}
/// Saves `PackedChat` Hex strings to a json file
pub fn save_whitelisted_users(packed_chats: Vec<PackedWhitelistedUser>, overwrite: bool) {
    // HashSet to avoid duplicate whitelisted users
    let mut existing_data = HashSet::new();

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

/// Saves `PackedChat` Hex strings to a json file
pub fn save_blacklisted_users(packed_chats: Vec<PackedBlacklistedUser>, overwrite: bool) {
    // HashSet to avoid duplicate whitelisted users
    let mut existing_data = HashSet::new();

    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push("blacklist.json");

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

/// Tries to find and read the Font files
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

/// Convenient function to format `NaiveDateTime` to string. Used for the Chart UI
pub fn time_to_string(time: &NaiveDateTime, timing: ChartTiming) -> String {
    match timing {
        ChartTiming::Hourly => time.to_string(),
        _ => time.format("%Y-%m-%d").to_string(),
    }
}

/// Convenient function to convert u8 to a Week name string. used for the Chart UI
pub fn weekday_num_to_string(weekday: u8) -> String {
    match weekday {
        0 => String::from("Monday"),
        1 => String::from("Tuesday"),
        2 => String::from("Wednesday"),
        3 => String::from("Thursday"),
        4 => String::from("Friday"),
        5 => String::from("Saturday"),
        6 => String::from("Sunday"),
        _ => unreachable!(),
    }
}

pub fn create_export_file(export_data: &str, file_name: String) {
    let mut export_file_location = PathBuf::from(".");
    export_file_location.push(file_name);
    let mut file = File::create(export_file_location).unwrap();
    file.write_all(export_data.as_bytes()).unwrap();
}

pub fn export_table_data(rows: Vec<UserRowData>, name: &str) {
    let mut export_file_location = PathBuf::from(".");
    let current_time = Local::now();
    let formatted_time = current_time.format("%Y-%m-%d %H-%M-%S").to_string();
    let file_name = format!("{name} Table Export {formatted_time}.csv");

    export_file_location.push(file_name);
    let file = File::create(export_file_location).unwrap();

    let mut wtr = csv::Writer::from_writer(file);

    for row in rows {
        if let Err(e) = wtr.serialize(row) {
            error!("Failed to add one row, skipping. Error: {e}");
        }
    }

    wtr.flush().unwrap();
}

pub fn separate_whitelist_by_seen(
    whitelist_data: Vec<PackedWhitelistedUser>,
) -> HashMap<String, Vec<String>> {
    let mut separated_data = HashMap::new();

    for data in whitelist_data {
        let entry = separated_data.entry(data.seen_by).or_insert(Vec::new());
        entry.push(data.hex_value);
    }

    separated_data
}

pub fn separate_blacklist_by_seen(
    whitelist_data: Vec<PackedBlacklistedUser>,
) -> HashMap<String, Vec<String>> {
    let mut separated_data = HashMap::new();

    for data in whitelist_data {
        let entry = separated_data.entry(data.seen_by).or_insert(Vec::new());
        entry.push(data.hex_value);
    }

    separated_data
}
/// Checks for a value in the `HashMap` of a `HashMap`
pub fn entry_insert_user(
    user_data: &mut HashMap<NaiveDate, HashMap<i64, UserRowData>>,
    user_row_data: UserRowData,
    id: i64,
    date: NaiveDate,
) {
    let entry = user_data.entry(date).or_default();
    entry.entry(id).or_insert(user_row_data.clone());
}

pub fn to_chart_name(user_name: String, full_name: &str, user_id: i64) -> String {
    if user_name != "Empty" {
        user_name
    } else if full_name == "Deleted Account" {
        user_id.to_string()
    } else {
        format!("{full_name} {user_id}")
    }
}
