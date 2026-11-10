use log::error;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use crate::ui::state::ParsedChat;

/// Convert inserted chat points into textual representation
pub fn chat_to_text(start: &str, end: &str) -> String {
    let chat_data: BTreeMap<String, ParsedChat> =
        parse_chat_details(start, end).into_iter().collect();

    let mut text_data = "Detected Chats:".to_string();

    for (name, parsed) in chat_data {
        let mut chat_text = format!(" {name}");

        if let Some(end_point) = parsed.end_point() {
            let _ = write!(chat_text, ": {end_point} - ");

            if let Some(start_point) = parsed.start_point() {
                let _ = write!(chat_text, "{start_point}");
            } else {
                chat_text += "∞";
            }
        } else if let Some(start_point) = parsed.start_point() {
            chat_text += ": 1 - ";
            let _ = write!(chat_text, "{start_point}");
        } else {
            chat_text += ": ∞";
        }
        let _ = write!(text_data, " {chat_text}");
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
                    error!(
                        "End point cannot be equal or bigger than start point. Ignoring the end point for {chat}"
                    );
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
        }
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
    }

    (chat_name, message_number)
}
