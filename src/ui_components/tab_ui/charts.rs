use chrono::{Datelike, Days, Duration, Months, NaiveDate, NaiveDateTime, Timelike, Weekday};
use eframe::egui::{
    CentralPanel, ComboBox, Id, Key, Modal, ScrollArea, TextEdit, TopBottomPanel, Ui,
};
use egui_extras::DatePickerButton;
use egui_plot::{Bar, BarChart, Legend, Plot, PlotPoint};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::Matcher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use strum::IntoEnumIterator;

use crate::ui_components::processor::{ChartTiming, ChartType, DateNavigator, NavigationType};
use crate::ui_components::widgets::AnimatedLabel;
use crate::ui_components::MainWindow;
use crate::utils::{time_to_string, weekday_num_to_string};

#[derive(Default)]
pub struct ChartsData {
    matcher: Matcher,
    search_text: String,
    modal_open: bool,
    available_users: BTreeMap<String, bool>,
    chart_type: ChartType,
    last_chart_type: ChartType,
    chart_timing: ChartTiming,
    added_to_chart: BTreeSet<String>,
    hourly_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    daily_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    weekly_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    monthly_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    weekday_message: BTreeMap<u8, HashMap<String, u64>>,
    last_hour: HashMap<String, Option<NaiveDateTime>>,
    last_day: HashMap<String, Option<NaiveDateTime>>,
    last_week: HashMap<String, Option<NaiveDateTime>>,
    last_month: HashMap<String, Option<NaiveDateTime>>,
    user_ids: HashMap<String, i64>,
    /// Pre-saved Bars to for the hourly chart, used for both Message and User
    hourly_bars: Option<BTreeMap<String, Vec<Bar>>>,
    /// Pre-saved Bars to for the daily chart, used for both Message and User
    daily_bars: Option<BTreeMap<String, Vec<Bar>>>,
    date_nav: DateNavigator,
    /// Hover labels, key = x value in chart. Values = (date, total message, whitelist message)
    labels: HashMap<i64, (NaiveDateTime, u64, u64)>,
    /// Hover labels for the hourly chart, key = x value in chart. Values = (date, total message, whitelist message)
    hourly_labels: HashMap<i64, (NaiveDateTime, u64, u64)>,
    /// Hover labels for the daily chart, key = x value in chart. Values = (date, total message, whitelist message)
    daily_labels: HashMap<i64, (NaiveDateTime, u64, u64)>,
}

impl ChartsData {
    /// Reset chart data
    pub fn reset_chart(&mut self) {
        self.available_users.clear();
        self.added_to_chart.clear();
        self.last_day = HashMap::new();
        self.last_hour = HashMap::new();
        self.last_month = HashMap::new();
        self.last_week = HashMap::new();
        self.daily_message.clear();
        self.hourly_message.clear();
        self.weekly_message.clear();
        self.daily_message.clear();
        self.user_ids.clear();
        self.weekday_message.clear();
        self.reset_saved_bars();

        let mut ongoing_value = Some(Weekday::Mon);

        while let Some(value) = ongoing_value {
            self.weekday_message.insert(value as u8, HashMap::new());
            let next_day = value.succ();

            if next_day == Weekday::Mon {
                ongoing_value = None;
            } else {
                ongoing_value = Some(next_day);
            }
        }

        // These two are added to the chart by default
        self.added_to_chart.insert("Show total data".to_string());
        self.added_to_chart
            .insert("Show whitelisted data".to_string());
        self.available_users
            .insert("Show total data".to_string(), true);
        self.available_users
            .insert("Show whitelisted data".to_string(), true);

        self.date_nav = DateNavigator::default();
    }
    /// Adds the user specified in the text edit in the chart
    fn add_to_chart(&mut self, to_add: String) {
        self.added_to_chart.insert(to_add);
        self.reset_saved_bars();
    }

    /// Removes the user that was clicked on from the chart
    fn remove_from_chart(&mut self, user: &str) {
        self.added_to_chart.remove(user);
        self.reset_saved_bars();
    }

    /// Adds a user available for adding in the chart
    pub fn add_user(&mut self, user: String, user_id: i64) {
        self.available_users.insert(user.clone(), false);
        self.user_ids.insert(user, user_id);
    }

    /// Takes a message creation time and the unique user to create necessary data to form a chart
    pub fn add_message(
        &mut self,
        time: NaiveDateTime,
        date: NaiveDate,
        add_to: String,
        client_name: &str,
    ) {
        // Keep a common value among messages for example messages sent within the same hour,
        // reset the second and minute value to 0 so these messages can be grouped
        let hourly_time = time.with_second(0).unwrap().with_minute(0).unwrap();
        let daily_time = time
            .with_second(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_hour(0)
            .unwrap();
        let monthly_time = time
            .with_second(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_day(1)
            .unwrap();

        let sent_on = time.weekday();

        // We only care about the week number for this. Set it as Monday to keep a common ground
        let week_day_name = Weekday::Mon;
        let week_num = time.iso_week().week();
        let time_year = time.iso_week().year();

        let weekly_time = NaiveDate::from_isoywd_opt(time_year, week_num, week_day_name)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // If last day is January 10, current day is January 5, add January 6 to 9 with 0 value
        // Apply the same for all of them
        if let Some(Some(last_hour)) = self.last_hour.get_mut(client_name) {
            let missing_hour = (*last_hour - hourly_time).num_hours();

            let mut ongoing_hour = *last_hour;
            for _ in 0..missing_hour {
                let to_add = ongoing_hour.checked_sub_signed(Duration::hours(1)).unwrap();
                self.hourly_message.entry(to_add).or_default();
                ongoing_hour = to_add;
            }
            *last_hour = hourly_time;
        } else {
            self.last_hour
                .insert(client_name.to_string(), Some(hourly_time));
        }

        if let Some(Some(last_day)) = self.last_day.get_mut(client_name) {
            let missing_day = (*last_day - daily_time).num_days();

            let mut ongoing_day = *last_day;
            for _ in 0..missing_day {
                let to_add = ongoing_day.checked_sub_signed(Duration::days(1)).unwrap();
                self.daily_message.entry(to_add).or_default();
                ongoing_day = to_add;
            }
            *last_day = daily_time;
        } else {
            self.last_day
                .insert(client_name.to_string(), Some(daily_time));
        }

        if let Some(Some(last_week)) = self.last_week.get_mut(client_name) {
            let missing_week = (*last_week - weekly_time).num_weeks();

            let mut ongoing_week = *last_week;
            for _ in 0..missing_week {
                let to_add = ongoing_week.checked_sub_signed(Duration::weeks(1)).unwrap();
                self.weekly_message.entry(to_add).or_default();
                ongoing_week = to_add;
            }
            *last_week = weekly_time;
        } else {
            self.last_week
                .insert(client_name.to_string(), Some(weekly_time));
        }

        if let Some(Some(last_month)) = self.last_month.get_mut(client_name) {
            let mut ongoing_month = *last_month;

            // All monthly date has the day set as 1. Reducing 2 days would take us to the previous month
            while ongoing_month > monthly_time {
                let to_add = ongoing_month
                    .checked_sub_signed(Duration::days(2))
                    .unwrap()
                    .with_day(1)
                    .unwrap();
                self.monthly_message.entry(to_add).or_default();
                ongoing_month = to_add;
            }
            *last_month = monthly_time;
        } else {
            self.last_month
                .insert(client_name.to_string(), Some(monthly_time));
        }

        let counter = self.hourly_message.entry(hourly_time).or_default();
        let target_user = counter.entry(add_to.clone()).or_insert(0);
        *target_user += 1;

        let counter = self.daily_message.entry(daily_time).or_default();
        let target_user = counter.entry(add_to.clone()).or_insert(0);
        *target_user += 1;

        let counter = self.weekly_message.entry(weekly_time).or_default();
        let target_user = counter.entry(add_to.clone()).or_insert(0);
        *target_user += 1;

        let counter = self.monthly_message.entry(monthly_time).or_default();
        let target_user = counter.entry(add_to.clone()).or_insert(0);
        *target_user += 1;

        let counter = self.weekday_message.get_mut(&(sent_on as u8)).unwrap();
        let target_user = counter.entry(add_to).or_insert(0);
        *target_user += 1;

        self.reset_saved_bars();
        self.date_nav.handler().update_dates(date);
    }

    /// Clears all pre-saved bars
    pub fn reset_saved_bars(&mut self) {
        self.hourly_bars = None;
        self.daily_bars = None;
        self.hourly_labels.clear();
        self.daily_labels.clear();
    }

    fn show_modal_popup(&mut self, ui: &mut Ui) {
        let response = Modal::new(Id::new("customize_view")).show(ui.ctx(), |ui| {
            ui.set_width(300.0);
            ui.set_height(300.0);
            TopBottomPanel::top("customize_top_view").show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Customize View");
                });
            });

            TopBottomPanel::bottom(Id::new("customize_bottom_view")).show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.vertical_centered_justified(|ui| {
                    if ui.button("Confirm").clicked() {
                        self.modal_open = false;
                        self.search_text.clear();
                    }
                })
            });

            CentralPanel::default().show_inside(ui, |ui| {
                let text_edit =
                    TextEdit::singleline(&mut self.search_text).hint_text("Search user");

                ui.add(text_edit);

                ScrollArea::vertical().show(ui, |ui| {
                    let all_keys = self.available_users.keys().cloned().collect::<Vec<_>>();

                    if self.search_text.is_empty() {
                        for val in all_keys {
                            ui.horizontal(|ui| {
                                ui.checkbox(self.available_users.get_mut(&val).unwrap(), val);
                                ui.allocate_space(ui.available_size());
                            });
                        }
                    } else {
                        let pattern = Pattern::parse(
                            &self.search_text,
                            CaseMatching::Ignore,
                            Normalization::Smart,
                        );
                        let matches = pattern.match_list(all_keys.iter(), &mut self.matcher);

                        for (val, _) in matches {
                            ui.horizontal(|ui| {
                                ui.checkbox(self.available_users.get_mut(val).unwrap(), val);
                                ui.allocate_space(ui.available_size());
                            });
                        }
                    }
                });
            });
        });

        if response.should_close() {
            self.modal_open = false;
            self.search_text.clear();
        }

        if !self.modal_open {
            for (key, val) in self.available_users.clone() {
                if val {
                    self.add_to_chart(key);
                } else {
                    self.remove_from_chart(&key);
                }
            }
        }
    }

    /// Whether total message and whitelist message are added to the chart
    fn message_whitelist_added(&self, row_len: usize) -> (bool, bool) {
        // If there are no whitelisted users, this will be considered as not-shown. Adds extra bars
        // to the ui => consume more power.
        let whitelist = self.added_to_chart.contains("Show whitelisted data") && row_len > 0;

        (self.added_to_chart.contains("Show total data"), whitelist)
    }

    pub fn clear_blacklisted(&mut self, names: &[String]) {
        for n in names {
            self.available_users.remove(n);
            self.added_to_chart.remove(n);
            self.hourly_message.iter_mut().for_each(|(_d, data)| {
                data.remove(n);
            });
            self.daily_message.iter_mut().for_each(|(_d, data)| {
                data.remove(n);
            });
            self.weekly_message.iter_mut().for_each(|(_d, data)| {
                data.remove(n);
            });
            self.monthly_message.iter_mut().for_each(|(_d, data)| {
                data.remove(n);
            });
            self.weekday_message.iter_mut().for_each(|(_d, data)| {
                data.remove(n);
            });
            self.user_ids.remove(n);
        }
        self.reset_saved_bars();
    }
}

impl MainWindow {
    pub fn show_charts_ui(&mut self, ui: &mut Ui) {
        let (values, len) = {
            let names = self.counter.get_chat_list();

            if names.is_empty() {
                (vec!["No chat available".to_string()], 0)
            } else {
                let total_val = names.len();
                (names, total_val)
            }
        };
        ui.horizontal(|ui| {
            ui.label("Selected chat:");
            ComboBox::from_id_salt("Table Box").show_index(
                ui,
                &mut self.chart_chat_index,
                len,
                |i| &values[i],
            );
        });
        ui.separator();
        let not_weekday_chart = self.chart_i().chart_type != ChartType::MessageWeekDay
            && self.chart_i().chart_type != ChartType::ActiveUserWeekDay;

        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.chart().chart_type,
                ChartType::Message,
                ChartType::Message.to_string(),
            ).on_hover_text("Chart displaying the total count of messages in the selected time frame (e.g., hourly, daily, weekly).");
            ui.separator();
            ui.selectable_value(
                &mut self.chart().chart_type,
                ChartType::ActiveUser,
                ChartType::ActiveUser.to_string(),
            ).on_hover_text("Chart showing the total count of active users in the selected time frame (e.g., hourly, daily, weekly).");
            ui.separator();
            ui.selectable_value(
                &mut self.chart().chart_type,
                ChartType::MessageWeekDay,
                ChartType::MessageWeekDay.to_string(),
            ).on_hover_text("Chart showing the total count of messages each day of the week.");
            ui.separator();
            ui.selectable_value(
                &mut self.chart().chart_type,
                ChartType::ActiveUserWeekDay,
                ChartType::ActiveUserWeekDay.to_string(),
            ).on_hover_text("Chart displaying the total count of active users for each day of the week.");
            ui.separator();
            if ui.button("Customize View").clicked() {
                self.chart().modal_open = true;
            }
        });
        if not_weekday_chart {
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.chart().chart_timing,
                    ChartTiming::Hourly,
                    ChartTiming::Hourly.to_string(),
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.chart().chart_timing,
                    ChartTiming::Daily,
                    ChartTiming::Daily.to_string(),
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.chart().chart_timing,
                    ChartTiming::Weekly,
                    ChartTiming::Weekly.to_string(),
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.chart().chart_timing,
                    ChartTiming::Monthly,
                    "Monthly",
                );
            });
            ui.separator();
        } else {
            ui.separator();
        }

        if self.chart().modal_open {
            self.chart().show_modal_popup(ui);
        }

        // Don't show any date stuff if it's a weekday chart
        if not_weekday_chart {
            let date_enabled = !self.is_processing && !self.chart().available_users.is_empty();

            ui.add_enabled_ui(date_enabled, |ui| {
                ui.horizontal(|ui| {
                    let chart = self.chart();

                    ui.label("From:");
                    ui.add(
                        DatePickerButton::new(chart.date_nav.handler().from())
                            .id_salt("1"),
                    )
                    .on_hover_text("Show data only after this date, including the date itself");
                    ui.label("To:");

                    ui.add(
                        DatePickerButton::new(chart.date_nav.handler().to()).id_salt("2"),
                    )
                    .on_hover_text("Show data only before this date, incluyding the date itself");

                    let reset_button = ui.button("Reset Date Selection").on_hover_text("Reset selected date to the oldest and the newest date with at least 1 data point");
                    if reset_button.clicked() {
                        chart.date_nav.handler().reset_dates();
                        chart.reset_saved_bars();
                    }

                    ui.separator();

                    let hover_position = ui.make_persistent_id("nav_hovered_2");
                    let selected_position = ui.make_persistent_id("nav_selected_2");
                    for nav in NavigationType::iter() {
                        let selected = chart.date_nav.nav_type_i() == nav;
                        let resp = ui.add(AnimatedLabel::new(
                            selected,
                            nav.to_string(),
                            selected_position,
                            hover_position,
                            50.0,
                            20.0,
                            None,
                            (false, false),
                        ));

                        if resp.clicked() {
                            *chart.date_nav.nav_type() = nav;
                        }
                    }

                    ui.separator();

                    let previous_hover = format!("Go back by 1 {} from the current date. Shortcut Key: CTRL + H", chart.date_nav.nav_name());
                    let next_hover = format!("Go next by 1 {} from the current date. Shortcut Key: CTRL + L", chart.date_nav.nav_name());

                    if ui.button(format!("Previous {}", chart.date_nav.nav_name())).on_hover_text(previous_hover).clicked() {
                        chart.date_nav.go_previous();
                    }

                    if ui.button(format!("Next {}", chart.date_nav.nav_name())).on_hover_text(next_hover).clicked() {
                        chart.date_nav.go_next();
                    }
                });
            });

            // Monitor for H and L key presses
            if date_enabled {
                let is_ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
                let key_h_pressed = ui.ctx().input(|i| i.key_pressed(Key::H));

                if key_h_pressed && is_ctrl_pressed {
                    self.chart().date_nav.go_previous();
                } else {
                    let key_l_pressed = ui.ctx().input(|i| i.key_pressed(Key::L));
                    if key_l_pressed && is_ctrl_pressed {
                        self.chart().date_nav.go_next();
                    }
                }
            }

            // Set pre-saved hourly and daily bars to None if date selection changes so they can be
            // rendered again and saved with the latest data
            if date_enabled && self.chart().date_nav.handler().check_date_change() {
                self.chart().reset_saved_bars();
            }

            ui.separator();
        }

        ui.horizontal(|ui| {
            ui.label("Use CTRL + scroll to zoom, drag mouse or scroll to move and double click to fit/reset the chart");
        });

        let current_type = &self.chart_i().chart_type;
        let last_type = &self.chart_i().last_chart_type;

        let is_message_user =
            current_type == &ChartType::Message || current_type == &ChartType::ActiveUser;

        // We do not care about changes in other timing as only these two are saved
        // If the last time Message type was selected, and currently it's user, reset saved bar
        // data and vice versa.
        if is_message_user && current_type != last_type {
            self.chart().last_chart_type = *current_type;
            self.chart().reset_saved_bars();
        }

        match self.chart().chart_type {
            ChartType::Message => self.display_message_chart(ui),
            ChartType::ActiveUser => self.display_active_user_chart(ui),
            ChartType::MessageWeekDay => self.display_weekday_message_chart(ui),
            ChartType::ActiveUserWeekDay => self.display_weekday_active_user_chart(ui),
        }
    }

    fn display_message_chart(&mut self, ui: &mut Ui) {
        let mut bar_list = BTreeMap::new();
        let mut point_dates = HashMap::new();

        let (show_total_message, show_whitelisted_message) = self
            .chart_i()
            .message_whitelist_added(self.whitelist.row_len());

        if self.chart().chart_timing == ChartTiming::Hourly && self.chart().hourly_bars.is_some() {
            self.display_chart(
                ui,
                show_total_message,
                show_whitelisted_message,
                self.chart_i().hourly_bars.clone().unwrap(),
            );
            return;
        }

        if self.chart().chart_timing == ChartTiming::Daily && self.chart().daily_bars.is_some() {
            self.display_chart(
                ui,
                show_total_message,
                show_whitelisted_message,
                self.chart_i().daily_bars.clone().unwrap(),
            );
            return;
        }

        let to_iter = match self.chart_i().chart_timing {
            ChartTiming::Hourly => self.chart_i().hourly_message.iter().enumerate(),
            ChartTiming::Daily => self.chart_i().daily_message.iter().enumerate(),
            ChartTiming::Weekly => self.chart_i().weekly_message.iter().enumerate(),
            ChartTiming::Monthly => self.chart_i().monthly_message.iter().enumerate(),
        };

        // Key = The common time where one or more message may have been sent
        // user = All users that sent messages to this common time + the amount of message
        for (index, (key, user)) in to_iter {
            let key_date = key.date();

            // Check whether the date is within the given range and whether before the to value
            // BTreeMap is already sorted. We are going from low to high so if already beyond the
            // to value, there is no use iterating further and break
            let within_range = self.chart_i().date_nav.handler_i().within_range(key_date);
            let before_to_range = self
                .chart_i()
                .date_nav
                .handler_i()
                .before_to_range(key_date);

            if !within_range && before_to_range {
                continue;
            }
            if !within_range {
                break;
            }

            let arg = index as f64;
            let mut total_message = 0;
            let mut whitelisted_message = 0;

            // All of the bar charts must have the same amount of Bar.
            // In case a common time does not include a user that is added in the chart
            // add a 0 value bar
            for i in &self.chart_i().added_to_chart {
                if !user.contains_key(i) && i != "Show total data" && i != "Show whitelisted data" {
                    let bar = Bar::new(arg, 0.0).name(format!(
                        "{} {i}",
                        time_to_string(key, self.chart_i().chart_timing)
                    ));
                    let bar_value = bar_list.entry(i.to_owned()).or_insert(Vec::new());

                    bar_value.push(bar);
                }
            }

            // Go through all the users that sent message in this common time and create a bar if necessary
            for (user_name, num) in user {
                if show_whitelisted_message {
                    let is_whitelisted = self
                        .whitelist
                        .is_user_whitelisted(*self.chart_i().user_ids.get(user_name).unwrap());

                    if is_whitelisted {
                        whitelisted_message += num;
                    }
                }

                if show_total_message {
                    total_message += num;
                }

                // If user in the chart, add the message count otherwise ignore
                let user_in_chart = self.chart_i().added_to_chart.contains(user_name);
                if user_in_chart {
                    let user_bar = Bar::new(arg, num.to_owned() as f64).name(format!(
                        "{} {user_name}",
                        time_to_string(key, self.chart_i().chart_timing)
                    ));
                    let bar_value = bar_list.entry(user_name.to_owned()).or_insert(Vec::new());
                    bar_value.push(user_bar);
                }
            }

            if show_total_message {
                let bar = Bar::new(arg, total_message as f64).name(format!(
                    "{} Total message",
                    time_to_string(key, self.chart_i().chart_timing)
                ));
                let bar_value = bar_list
                    .entry("Show total data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            if show_whitelisted_message {
                let bar = Bar::new(arg, whitelisted_message as f64).name(format!(
                    "{} Whitelisted message",
                    time_to_string(key, self.chart_i().chart_timing)
                ));
                let bar_value = bar_list
                    .entry("Show whitelisted data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }
            point_dates.insert(index as i64, (*key, total_message, whitelisted_message));
        }

        if self.chart().chart_timing == ChartTiming::Hourly {
            self.chart().hourly_bars = Some(bar_list.clone());
            self.chart().hourly_labels.clone_from(&point_dates);
        }

        if self.chart().chart_timing == ChartTiming::Daily {
            self.chart().daily_bars = Some(bar_list.clone());
            self.chart().daily_labels.clone_from(&point_dates);
        }

        self.chart().labels = point_dates;
        self.display_chart(ui, show_total_message, show_whitelisted_message, bar_list);
    }

    fn display_active_user_chart(&mut self, ui: &mut Ui) {
        let mut bar_list = BTreeMap::new();
        let mut point_dates = HashMap::new();

        let to_iter = match self.chart().chart_timing {
            ChartTiming::Hourly => self.chart_i().hourly_message.iter().enumerate(),
            ChartTiming::Daily => self.chart_i().daily_message.iter().enumerate(),
            ChartTiming::Weekly => self.chart_i().weekly_message.iter().enumerate(),
            ChartTiming::Monthly => self.chart_i().monthly_message.iter().enumerate(),
        };

        let (show_total_message, show_whitelisted_message) = self
            .chart_i()
            .message_whitelist_added(self.whitelist.row_len());

        if self.chart_i().chart_timing == ChartTiming::Hourly
            && self.chart_i().hourly_bars.is_some()
        {
            self.display_chart(
                ui,
                show_total_message,
                show_whitelisted_message,
                self.chart_i().hourly_bars.clone().unwrap(),
            );
            return;
        }

        if self.chart_i().chart_timing == ChartTiming::Daily && self.chart_i().daily_bars.is_some()
        {
            self.display_chart(
                ui,
                show_total_message,
                show_whitelisted_message,
                self.chart_i().daily_bars.clone().unwrap(),
            );
            return;
        }
        // Key = The common time where one or more message may have been sent
        // user = All users that sent messages to this common time + the amount of message
        for (index, (key, user)) in to_iter {
            let arg = index as f64;
            let key_date = key.date();
            // Check whether the date is within the given range and whether before the to value
            // BTreeMap is already sorted. We are going from low to high so if already beyond the
            // to value, there is no use iterating further and break
            let within_range = self.chart_i().date_nav.handler_i().within_range(key_date);
            let before_to_range = self
                .chart_i()
                .date_nav
                .handler_i()
                .before_to_range(key_date);

            if !within_range && before_to_range {
                continue;
            }
            if !within_range {
                break;
            }

            let mut total_user = 0;
            let mut whitelisted_user = 0;

            if show_whitelisted_message {
                for user_name in user.keys() {
                    let is_whitelisted = self
                        .whitelist
                        .is_user_whitelisted(*self.chart_i().user_ids.get(user_name).unwrap());

                    if is_whitelisted {
                        whitelisted_user += 1;
                    }
                    total_user += 1;
                }
            } else if show_total_message {
                total_user += user.len();
            }

            if show_total_message {
                let bar = Bar::new(arg, total_user as f64).name(format!(
                    "{} Total user",
                    time_to_string(key, self.chart_i().chart_timing)
                ));
                let bar_value = bar_list
                    .entry("Show total data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            if show_whitelisted_message {
                let bar = Bar::new(arg, f64::from(whitelisted_user)).name(format!(
                    "{} Whitelisted user",
                    time_to_string(key, self.chart_i().chart_timing)
                ));
                let bar_value = bar_list
                    .entry("Show whitelisted data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            point_dates.insert(
                index as i64,
                (*key, total_user as u64, whitelisted_user as u64),
            );
        }

        if self.chart().chart_timing == ChartTiming::Hourly {
            self.chart().hourly_bars = Some(bar_list.clone());
            self.chart().hourly_labels.clone_from(&point_dates);
        }

        if self.chart().chart_timing == ChartTiming::Daily {
            self.chart().daily_bars = Some(bar_list.clone());
            self.chart().daily_labels.clone_from(&point_dates);
        }

        self.chart().labels = point_dates;
        self.display_chart(ui, show_total_message, show_whitelisted_message, bar_list);
    }

    fn display_weekday_message_chart(&mut self, ui: &mut Ui) {
        let mut bar_list = BTreeMap::new();
        let mut point_dates = HashMap::new();

        let to_iter = self.chart_i().weekday_message.iter().enumerate();

        let (show_total_message, show_whitelisted_message) = self
            .chart_i()
            .message_whitelist_added(self.whitelist.row_len());

        // Key = Weekday num
        // user = All users that sent messages to this common time + the amount of message
        for (index, (key, user)) in to_iter {
            let arg = index as f64;
            let mut total_message = 0;
            let mut whitelisted_message = 0;

            for (user_name, num) in user {
                if show_whitelisted_message {
                    let is_whitelisted = self
                        .whitelist
                        .is_user_whitelisted(*self.chart_i().user_ids.get(user_name).unwrap());

                    if is_whitelisted {
                        whitelisted_message += num;
                    }
                }

                if show_total_message {
                    total_message += num;
                }
            }

            if show_total_message {
                let bar = Bar::new(arg, total_message as f64)
                    .name(format!("{} Total message", weekday_num_to_string(*key)));
                let bar_value = bar_list
                    .entry("Show total data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            if show_whitelisted_message {
                let bar = Bar::new(arg, whitelisted_message as f64).name(format!(
                    "{} Whitelisted message ",
                    weekday_num_to_string(*key)
                ));
                let bar_value = bar_list
                    .entry("Show whitelisted data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }
            point_dates.insert(
                index as i64,
                (NaiveDateTime::default(), total_message, whitelisted_message),
            );
        }

        self.chart().labels = point_dates;
        self.display_chart(ui, show_total_message, show_whitelisted_message, bar_list);
    }

    fn display_weekday_active_user_chart(&mut self, ui: &mut Ui) {
        let mut bar_list = BTreeMap::new();
        let mut point_dates = HashMap::new();

        let to_iter = self.chart_i().weekday_message.iter().enumerate();

        let (show_total_message, show_whitelisted_message) = self
            .chart_i()
            .message_whitelist_added(self.whitelist.row_len());

        // Key = weekday num
        // user = All users that sent messages to this common time + the amount of message
        for (index, (key, user)) in to_iter {
            let arg = index as f64;
            let mut total_user = 0;
            let mut whitelisted_user = 0;

            if show_whitelisted_message {
                for user_name in user.keys() {
                    let is_whitelisted = self
                        .whitelist
                        .is_user_whitelisted(*self.chart_i().user_ids.get(user_name).unwrap());

                    if is_whitelisted {
                        whitelisted_user += 1;
                    }
                    total_user += 1;
                }
            } else if show_total_message {
                total_user += user.len();
            }

            if show_total_message {
                let bar = Bar::new(arg, total_user as f64)
                    .name(format!("{} Total user", weekday_num_to_string(*key)));
                let bar_value = bar_list
                    .entry("Show total data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            if show_whitelisted_message {
                let bar = Bar::new(arg, f64::from(whitelisted_user))
                    .name(format!("{} Whitelisted user", weekday_num_to_string(*key)));
                let bar_value = bar_list
                    .entry("Show whitelisted data".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }
            point_dates.insert(
                index as i64,
                (
                    NaiveDateTime::default(),
                    total_user as u64,
                    whitelisted_user as u64,
                ),
            );
        }

        self.chart().labels = point_dates;
        self.display_chart(ui, show_total_message, show_whitelisted_message, bar_list);
    }

    fn display_chart(
        &mut self,
        ui: &mut Ui,
        show_total_message: bool,
        show_whitelisted_message: bool,
        mut bar_list: BTreeMap<String, Vec<Bar>>,
    ) {
        let mut all_charts = Vec::new();

        let total_data_name = match self.chart().chart_type {
            ChartType::Message | ChartType::MessageWeekDay => "Total Message",
            ChartType::ActiveUser | ChartType::ActiveUserWeekDay => "Total User",
        };

        let whitelist_data_name = match self.chart().chart_type {
            ChartType::Message | ChartType::MessageWeekDay => "Whitelisted Message",
            ChartType::ActiveUser | ChartType::ActiveUserWeekDay => "Whitelisted User",
        };

        // Whitelist message should be above the total message
        // In case the date picker is used the bar list may not contain the following bar names
        // even if they are already in the list
        if show_whitelisted_message {
            if let Some(whitelist_bar) = bar_list.remove("Show whitelisted data") {
                let mut whitelist_chart = BarChart::new("Whitelist", whitelist_bar)
                    .width(1.0)
                    .name(whitelist_data_name);

                if show_total_message {
                    if let Some(total_message_bars) = bar_list.remove("Show total data") {
                        let total_message_chart =
                            BarChart::new("Total message", total_message_bars)
                                .width(1.0)
                                .name(total_data_name);

                        whitelist_chart = whitelist_chart.stack_on(&[&total_message_chart]);
                        all_charts.push(total_message_chart);
                    }
                }
                all_charts.push(whitelist_chart);
            }
        } else if show_total_message {
            if let Some(total_message_bars) = bar_list.remove("Show total data") {
                let total_message_chart = BarChart::new("Total message", total_message_bars)
                    .width(1.0)
                    .name(total_data_name);
                all_charts.push(total_message_chart);
            }
        }

        // User data stacking only happens on Message chart
        if self.chart().chart_type == ChartType::Message {
            // Only triggered when Something other than total and whitelisted message is added to
            // the chart.
            // All charts must be stacked by all the previous charts
            // Chart 3 will be stacked by chart 1 and 2
            // The target is the bottom chart is total message => whitelist => the rest of the users
            if !bar_list.is_empty() {
                for (name, bar) in bar_list {
                    let current_chart = BarChart::new(&name, bar).width(1.0).name(name);

                    if all_charts.is_empty() {
                        all_charts.push(current_chart);
                    } else {
                        let current_chart =
                            current_chart.stack_on(&all_charts.iter().collect::<Vec<&BarChart>>());
                        all_charts.push(current_chart);
                    }
                }
            }
        }
        let timing = self.chart().chart_timing;
        let chart_type = self.chart().chart_type;
        let is_weekday =
            chart_type == ChartType::MessageWeekDay || chart_type == ChartType::ActiveUserWeekDay;

        let labels = if timing == ChartTiming::Hourly && !is_weekday {
            self.chart().hourly_labels.clone()
        } else if timing == ChartTiming::Daily && !is_weekday {
            self.chart().daily_labels.clone()
        } else {
            self.chart().labels.clone()
        };

        let label_fmt = move |_s: &str, val: &PlotPoint| {
            let x_val = val.x.round() as i64;
            if let Some((date, total, whitelist)) = labels.get(&x_val) {
                let label_type = if chart_type == ChartType::Message
                    || chart_type == ChartType::MessageWeekDay
                {
                    "Message"
                } else {
                    "User"
                };

                let date_label;

                match chart_type {
                    ChartType::Message | ChartType::ActiveUser => match timing {
                        ChartTiming::Hourly | ChartTiming::Daily => {
                            date_label = date.to_string();
                        }
                        ChartTiming::Weekly => {
                            let other_date = date.checked_add_days(Days::new(7)).unwrap();
                            date_label = format!("{date} - {other_date}");
                        }
                        ChartTiming::Monthly => {
                            let other_date = date.checked_add_months(Months::new(1)).unwrap();
                            date_label = format!("{date} - {other_date}");
                        }
                    },
                    ChartType::MessageWeekDay | ChartType::ActiveUserWeekDay => {
                        date_label = weekday_num_to_string(x_val as u8);
                    }
                }
                format!(
                    "{}\nY = {:.0}\nTotal {label_type} = {}\nWhitelisted {label_type} = {}",
                    date_label, val.y, total, whitelist
                )
            } else {
                format!("X = {:.0}\nY = {:.0}", val.x, val.y)
            }
        };

        Plot::new("Plot")
            .legend(Legend::default().background_alpha(0.0))
            .auto_bounds([true; 2])
            .clamp_grid(true)
            .label_formatter(label_fmt)
            .show(ui, |plot_ui| {
                for chart in all_charts {
                    plot_ui.bar_chart(chart);
                }
            });
    }
}
