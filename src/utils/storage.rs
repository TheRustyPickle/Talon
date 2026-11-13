use grammers_session::types::PeerRef;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::ui::TGKeys;
use crate::ui::state::{ListKind, PackedUser};

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

/// Reads the saved user `PeerRef` values of a list and returns them
pub fn get_saved_users(kind: ListKind) -> Result<Vec<PackedUser>, Box<dyn Error>> {
    let path = PathBuf::from(".").join(kind.file_name());

    if let Ok(mut file) = File::open(path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let users = serde_json::from_str(&contents)?;
        Ok(users)
    } else {
        Ok(Vec::new())
    }
}

/// Saves `PeerRef` values of a list to its json file
pub fn save_users(kind: ListKind, packed_chats: Vec<PackedUser>, overwrite: bool) {
    // HashSet to avoid duplicate users
    let mut existing_data = HashSet::new();

    let path = PathBuf::from(".").join(kind.file_name());

    if !overwrite && let Ok(mut file) = File::open(&path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");

        existing_data = serde_json::from_str(&contents).unwrap();
    }

    existing_data.extend(packed_chats);

    let data = serde_json::to_string(&existing_data).unwrap();

    let mut file = File::create(path).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

/// Groups the saved users by the session that saw them
pub fn separate_by_seen(kind: ListKind, saved: Vec<PackedUser>) -> HashMap<String, Vec<PeerRef>> {
    let mut separated_data = HashMap::new();

    for data in saved {
        let Some(peer_ref) = data.peer_ref() else {
            error!(
                "Invalid peer id {} found in {}, skipping",
                data.peer_id,
                kind.name()
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
