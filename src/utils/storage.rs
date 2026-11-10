use grammers_session::types::PeerRef;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::ui::TGKeys;
use crate::ui::state::{PackedBlacklistedUser, PackedWhitelistedUser};

const WHITELIST_FILE: &str = "whitelist_v2.json";

const BLACKLIST_FILE: &str = "blacklist_v2.json";

#[derive(Serialize, Deserialize)]
pub struct IsLightTheme {
    is_light: bool,
}

/// Finds all the saved session files
pub fn find_session_files() -> Vec<String> {
    let mut sessions = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries {
            if let Ok(file_name) = entry.unwrap().file_name().into_string()
                && file_name.ends_with(".sqlite")
            {
                info!("Found existing session file {file_name}");
                sessions.push(file_name);
            }
        }
    }
    sessions
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
        if let Ok(result) = result
            && !result.api_id.is_empty()
            && !result.api_hash.is_empty()
        {
            to_return = Some(result);
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
    }
}

/// Reads the whitelisted user `PeerRef` values and returns them
pub fn get_whitelisted() -> Result<Vec<PackedWhitelistedUser>, Box<dyn Error>> {
    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push(WHITELIST_FILE);

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

/// Reads the blacklisted user `PeerRef` values and returns them
pub fn get_blacklisted() -> Result<Vec<PackedBlacklistedUser>, Box<dyn Error>> {
    let mut blacklist_path = PathBuf::from(".");
    blacklist_path.push(BLACKLIST_FILE);

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

/// Saves `PeerRef` values to a json file
pub fn save_whitelisted_users(packed_chats: Vec<PackedWhitelistedUser>, overwrite: bool) {
    // HashSet to avoid duplicate whitelisted users
    let mut existing_data = HashSet::new();

    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push(WHITELIST_FILE);

    let file = File::open(&whitelist_path);

    if !overwrite && let Ok(mut file) = file {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");

        existing_data = serde_json::from_str(&contents).unwrap();
    }

    existing_data.extend(packed_chats);

    let data = serde_json::to_string(&existing_data).unwrap();

    let mut file = File::create(whitelist_path).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

/// Saves `PeerRef` values to a json file
pub fn save_blacklisted_users(packed_chats: Vec<PackedBlacklistedUser>, overwrite: bool) {
    // HashSet to avoid duplicate whitelisted users
    let mut existing_data = HashSet::new();

    let mut whitelist_path = PathBuf::from(".");
    whitelist_path.push(BLACKLIST_FILE);

    let file = File::open(&whitelist_path);

    if !overwrite && let Ok(mut file) = file {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");

        existing_data = serde_json::from_str(&contents).unwrap();
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

pub fn separate_whitelist_by_seen(
    whitelist_data: Vec<PackedWhitelistedUser>,
) -> HashMap<String, Vec<PeerRef>> {
    let mut separated_data = HashMap::new();

    for data in whitelist_data {
        let Some(peer_ref) = data.peer_ref() else {
            error!(
                "Invalid peer id {} found in whitelist, skipping",
                data.peer_id
            );
            continue;
        };
        let entry = separated_data.entry(data.seen_by).or_insert(Vec::new());
        entry.push(peer_ref);
    }

    separated_data
}

pub fn separate_blacklist_by_seen(
    blacklist_data: Vec<PackedBlacklistedUser>,
) -> HashMap<String, Vec<PeerRef>> {
    let mut separated_data = HashMap::new();

    for data in blacklist_data {
        let Some(peer_ref) = data.peer_ref() else {
            error!(
                "Invalid peer id {} found in blacklist, skipping",
                data.peer_id
            );
            continue;
        };
        let entry = separated_data.entry(data.seen_by).or_insert(Vec::new());
        entry.push(peer_ref);
    }

    separated_data
}

pub fn last_theme() -> Result<bool, Box<dyn Error>> {
    let mut theme_json = PathBuf::from(".");
    theme_json.push("theme.json");

    if let Ok(mut file) = File::open(theme_json) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let existing_data: IsLightTheme = serde_json::from_str(&contents)?;
        Ok(existing_data.is_light)
    } else {
        Ok(true)
    }
}

pub fn save_theme(status: bool) {
    let new_data = IsLightTheme { is_light: status };

    let data = serde_json::to_string(&new_data);

    if let Ok(data) = data {
        let mut theme_path = PathBuf::from(".");
        theme_path.push("theme.json");
        let mut file = File::create(theme_path).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }
}
