use egui_selectable_table::SelectableRow;
use jiff::Zoned;
use jiff::civil::{Date, DateTime};
use log::error;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use crate::ui::state::{ChartTiming, ColumnName};
use crate::ui::tabs::UserRowData;

/// Returns the proper text based on light or dark value
pub fn theme_hover_text(is_light_theme: bool) -> String {
    if is_light_theme {
        "Switch to dark theme".to_string()
    } else {
        "Switch to light theme".to_string()
    }
}

/// Formats a date and time as `2025-01-31 14:05:09`. Used by the table and the charts
pub fn format_datetime(time: &DateTime) -> String {
    time.strftime("%Y-%m-%d %H:%M:%S").to_string()
}

/// Convenient function to format a `DateTime` to string. Used for the Chart UI
pub fn time_to_string(time: &DateTime, timing: ChartTiming) -> String {
    match timing {
        ChartTiming::Hourly => format_datetime(time),
        _ => time.date().to_string(),
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

pub fn export_table_data(rows: &Vec<SelectableRow<UserRowData, ColumnName>>, name: &str) {
    let mut export_file_location = PathBuf::from(".");
    let formatted_time = Zoned::now().strftime("%Y-%m-%d %H-%M-%S").to_string();
    let file_name = format!("{name} Table Export {formatted_time}.csv");

    export_file_location.push(file_name);
    let file = File::create(export_file_location).unwrap();

    let mut wtr = csv::Writer::from_writer(file);

    for row in rows {
        let row = &row.row_data;
        if let Err(e) = wtr.serialize(row) {
            error!("Failed to add one row, skipping. Error: {e}");
        }
    }

    wtr.flush().unwrap();
}

/// Checks for a value in the `HashMap` of a `HashMap`
pub fn entry_insert_user(
    user_data: &mut HashMap<Date, HashMap<i64, UserRowData>>,
    user_row_data: UserRowData,
    id: i64,
    date: Date,
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
