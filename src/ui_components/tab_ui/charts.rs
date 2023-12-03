use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Weekday};
use eframe::egui::{Align, Button, Grid, Layout, RichText, Ui};
use egui_dropdown::DropDownBox;
use egui_plot::{Bar, BarChart, Legend, Plot};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::ui_components::processor::{ChartTiming, ChartType};
use crate::ui_components::MainWindow;
use crate::utils::{days_in_month, time_to_string};

#[derive(Default)]
pub struct ChartsData {
    available_users: BTreeSet<String>,
    dropdown_user: String,
    chart_type: ChartType,
    chart_timing: ChartTiming,
    added_to_chart: BTreeSet<String>,
    button_sizes: HashMap<String, Option<f32>>,
    hourly_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    daily_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    weekly_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    monthly_message: BTreeMap<NaiveDateTime, HashMap<String, u64>>,
    last_hour: Option<NaiveDateTime>,
    last_day: Option<NaiveDateTime>,
    last_week: Option<NaiveDateTime>,
    last_month: Option<NaiveDateTime>,
    user_ids: HashMap<String, i64>,
}

impl ChartsData {
    fn add_to_chart(&mut self) {
        self.added_to_chart.insert(self.dropdown_user.to_owned());
        self.available_users.remove(&self.dropdown_user);
        self.button_sizes
            .insert(self.dropdown_user.to_owned(), None);
        self.dropdown_user.clear();
    }

    fn remove_from_chart(&mut self, user: &str) {
        self.added_to_chart.remove(user);
        self.available_users.insert(user.to_string());
    }

    pub fn add_user(&mut self, user: String, user_id: i64) {
        self.available_users.insert(user.to_owned());
        self.user_ids.insert(user, user_id);
    }

    /// Takes a message creation time to create necessary data to form a chart
    pub fn add_message(&mut self, time: NaiveDateTime, add_to: String) {
        // keep a common value among messages for example messages sent within the same hour,
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

        // We don't care on which day of the week the message was sent
        // We only care about the week number. Set it as Monday to keep a common ground
        let week_day_name = Weekday::Mon;
        let week_num = time.iso_week().week();
        let time_year = time.year();

        // On this date 2021-01-03 the year is 2021 but the week number is 53 according to ISO week.
        // Reduce the year by 1 in such cases 
        let weekly_time = if let Some(time) = NaiveDate::from_isoywd_opt(time_year, week_num, week_day_name) {
            time.and_hms_opt(0, 0, 0).unwrap()
        } else {
            NaiveDate::from_isoywd_opt(time_year - 1, week_num, week_day_name)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
        };

        // If last day is January 10, current day is January 5, add january 6 to 9 with 0 value
        // Apply the same for all of them
        if let Some(last_hour) = self.last_hour {
            let missing_hour = (last_hour - hourly_time).num_hours();

            let mut ongoing_hour = last_hour;
            for _ in 0..missing_hour {
                let to_add = ongoing_hour.checked_sub_signed(Duration::hours(1)).unwrap();
                self.hourly_message.entry(to_add).or_default();
                ongoing_hour = to_add;
            }
        }
        self.last_hour = Some(hourly_time);

        if let Some(last_day) = self.last_day {
            let missing_day = (last_day - daily_time).num_days();

            let mut ongoing_day = last_day;
            for _ in 0..missing_day {
                let to_add = ongoing_day.checked_sub_signed(Duration::days(1)).unwrap();
                self.daily_message.entry(to_add).or_default();
                ongoing_day = to_add;
            }
        }
        self.last_day = Some(daily_time);

        if let Some(last_week) = self.last_week {
            let missing_day = (last_week - weekly_time).num_weeks();

            let mut ongoing_week = last_week;
            for _ in 0..missing_day {
                let to_add = ongoing_week.checked_sub_signed(Duration::weeks(1)).unwrap();
                self.weekly_message.entry(to_add).or_default();
                ongoing_week = to_add;
            }
        }
        self.last_week = Some(weekly_time);

        if let Some(last_month) = self.last_month {
            let missing_month = (last_month.year() - monthly_time.year()) * 12
                + (last_month.month() - monthly_time.month()) as i32;

            let mut ongoing_month = last_month;
            for _ in 0..missing_month {
                let total_days = days_in_month(ongoing_month.month(), ongoing_month.year());
                let to_add = ongoing_month
                    .checked_sub_signed(Duration::days(total_days))
                    .unwrap()
                    .with_day(1)
                    .unwrap();
                self.monthly_message.entry(to_add).or_default();
                ongoing_month = to_add;
            }
        }
        self.last_month = Some(monthly_time);

        let counter = self.hourly_message.entry(hourly_time).or_default();
        let target_user = counter.entry(add_to.to_owned()).or_insert(0);
        *target_user += 1;

        let counter = self.daily_message.entry(daily_time).or_default();
        let target_user = counter.entry(add_to.to_owned()).or_insert(0);
        *target_user += 1;

        let counter = self.weekly_message.entry(weekly_time).or_default();
        let target_user = counter.entry(add_to.to_owned()).or_insert(0);
        *target_user += 1;

        let counter = self.monthly_message.entry(monthly_time).or_default();
        let target_user = counter.entry(add_to).or_insert(0);
        *target_user += 1;
    }

    pub fn reset_chart(&mut self) {
        self.available_users.clear();
        self.dropdown_user.clear();
        self.added_to_chart.clear();
        self.button_sizes.clear();
        self.last_day = None;
        self.last_hour = None;
        self.last_month = None;
        self.last_week = None;
        self.daily_message.clear();
        self.hourly_message.clear();
        self.weekly_message.clear();
        self.daily_message.clear();
        self.user_ids.clear();

        self.dropdown_user = "Show total message".to_string();
        self.add_to_chart();
        self.dropdown_user = "Show whitelisted message".to_string();
        self.add_to_chart();
    }
}

impl MainWindow {
    pub fn show_charts_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.charts_data.chart_type,
                ChartType::Message,
                "Message",
            );
            ui.separator();
            ui.selectable_value(
                &mut self.charts_data.chart_type,
                ChartType::MessageWeekDay,
                "Message Weekday",
            );
            ui.separator();
            ui.selectable_value(
                &mut self.charts_data.chart_type,
                ChartType::ActiveUser,
                "Active User",
            );
        });
        if self.charts_data.chart_type != ChartType::MessageWeekDay {
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Hourly,
                    "Hourly",
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Daily,
                    "Daily",
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Weekly,
                    "Weekly",
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.charts_data.chart_timing,
                    ChartTiming::Monthly,
                    "Monthly",
                );
            });
            ui.separator();
        } else {
            ui.separator();
        }
        if !self.charts_data.available_users.is_empty() {
            Grid::new("Chart Grid")
                .num_columns(1)
                .spacing([5.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let add_button = Button::new("Add to chart");

                        if self.charts_data.dropdown_user.is_empty()
                            || !self
                                .charts_data
                                .available_users
                                .contains(&self.charts_data.dropdown_user)
                        {
                            ui.add_enabled(false, add_button);
                        } else if ui.add(add_button).clicked() {
                            self.charts_data.add_to_chart();
                        };

                        ui.add_sized(
                            ui.available_size(),
                            DropDownBox::from_iter(
                                &self.charts_data.available_users,
                                "DropDown",
                                &mut self.charts_data.dropdown_user,
                                |ui, text| ui.selectable_label(false, text),
                            ),
                        )
                        .on_hover_text("Select an user to add to the chart");
                    });
                });
        }

        if !self.charts_data.added_to_chart.is_empty() {
            ui.separator();

            ui.vertical(|ui| {
                let mut to_add: Vec<String> = Vec::new();
                let mut already_added = 0.0;
                let max_size = ui.available_width();
                for (index, user) in self
                    .charts_data
                    .added_to_chart
                    .to_owned()
                    .iter()
                    .enumerate()
                {
                    // Check if the button size is saved previously or try to estimate a size
                    let button_size =
                        if let Some(size) = self.charts_data.button_sizes.get(user).unwrap() {
                            size.to_owned()
                        } else {
                            (user.len() as f32 * (ui.style().spacing.button_padding.x * 2.0))
                                + ui.spacing().item_spacing.x
                        };
                    already_added += button_size;
                    to_add.push(user.to_string());

                    // When total size is above the max width, place the buttons in the horizontal layout
                    if already_added >= max_size {
                        ui.horizontal(|ui| {
                            // If max size is 500, after the latest addition it became 550, the last button should not be in this layout
                            // Pop it and use it in the next horizontal layout
                            let last_value = to_add.pop().unwrap();
                            for button in &to_add {
                                let text_data = RichText::new(button);

                                let resp = ui
                                    .button(text_data)
                                    .on_hover_text("Click to remove from chart");

                                if resp.clicked() {
                                    self.charts_data.remove_from_chart(button)
                                }

                                *self.charts_data.button_sizes.get_mut(button).unwrap() =
                                    Some(resp.rect.width() + ui.spacing().item_spacing.x);
                            }
                            to_add.clear();
                            to_add.push(last_value);
                            // The size of the last value will be used for the next horizontal layout
                            already_added = button_size;
                        });
                    }

                    // If any pending button remains, add them
                    if index == self.charts_data.added_to_chart.len() - 1 {
                        ui.horizontal(|ui| {
                            for button in &to_add {
                                let text_data = RichText::new(button);

                                let resp = ui
                                    .button(text_data)
                                    .on_hover_text("Click to remove from chart");
                                if resp.clicked() {
                                    self.charts_data.remove_from_chart(button)
                                }
                                *self.charts_data.button_sizes.get_mut(button).unwrap() =
                                    Some(resp.rect.width() + ui.spacing().item_spacing.x);
                            }
                        });
                    }
                }
            });
            ui.separator();
        }
        ui.label("Use CTRL + scroll to zoom, drag or scroll to move and double click to fit/reset the chart");

        match self.charts_data.chart_type {
            ChartType::Message => self.display_message_chart(ui),
            _ => {}
        }
    }

    fn display_message_chart(&mut self, ui: &mut Ui) {
        let mut bar_list = BTreeMap::new();
        let mut ongoing_arg = 0.0;

        let (to_iter, key_length) = match self.charts_data.chart_timing {
            ChartTiming::Hourly => (
                self.charts_data.hourly_message.iter(),
                self.charts_data.hourly_message.len(),
            ),
            ChartTiming::Daily => (
                self.charts_data.daily_message.iter(),
                self.charts_data.daily_message.len(),
            ),
            ChartTiming::Weekly => (
                self.charts_data.weekly_message.iter(),
                self.charts_data.weekly_message.len(),
            ),
            ChartTiming::Monthly => (
                self.charts_data.monthly_message.iter(),
                self.charts_data.monthly_message.len(),
            ),
        };

        let show_total_message = self
            .charts_data
            .added_to_chart
            .contains(&"Show total message".to_string());
        let show_whitelisted_message = self
            .charts_data
            .added_to_chart
            .contains(&"Show whitelisted message".to_string());
        // Keep the max range of x axis to 100
        let point_value = 100.0 / key_length as f64;

        // Key = The common time where one or more message may have been sent
        // user = All users that sent messages to this common time + the amount of message
        for (key, user) in to_iter {
            let mut total_message = 0;
            let mut whitelisted_message = 0;

            // All of the bar charts must have the same amount of Bar.
            // In case a common time does not include a user that is added in the chart
            // add a 0 value bar
            for i in &self.charts_data.added_to_chart {
                if !user.contains_key(i)
                    && i != "Show total message"
                    && i != "Show whitelisted message"
                {
                    let bar = Bar::new(ongoing_arg, 0.0)
                        .name(time_to_string(key, &self.charts_data.chart_timing));
                    let bar_value = bar_list.entry(i.to_owned()).or_insert(Vec::new());

                    bar_value.push(bar);
                }
            }

            // Go through all the users that sent message in this common time
            // Regardless of whether total message or whitelisted message is on the chart,
            // they are calculated
            for (user_name, num) in user {
                let is_whitelisted = self
                    .whitelist_data
                    .is_user_whitelisted(self.charts_data.user_ids.get(user_name).unwrap());
                total_message += num;

                if is_whitelisted {
                    whitelisted_message += num;
                }

                // If user in the chart, add the message count otherwise ignore
                let user_in_chart = self.charts_data.added_to_chart.contains(user_name);
                if user_in_chart {
                    let user_bar = Bar::new(ongoing_arg, num.to_owned() as f64).name(format!(
                        "{user_name} {}",
                        time_to_string(key, &self.charts_data.chart_timing)
                    ));
                    let bar_value = bar_list.entry(user_name.to_owned()).or_insert(Vec::new());
                    bar_value.push(user_bar);
                }
            }

            if show_total_message {
                let bar = Bar::new(ongoing_arg, total_message as f64).name(format!(
                    "Total message {}",
                    time_to_string(key, &self.charts_data.chart_timing)
                ));
                let bar_value = bar_list
                    .entry("Show total message".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            if show_whitelisted_message {
                let bar = Bar::new(ongoing_arg, whitelisted_message as f64).name(format!(
                    "Whitelisted message {}",
                    time_to_string(key, &self.charts_data.chart_timing)
                ));
                let bar_value = bar_list
                    .entry("Show whitelisted message".to_owned())
                    .or_insert(Vec::new());
                bar_value.push(bar);
            }

            ongoing_arg += point_value;
        }

        let mut all_charts = Vec::new();

        // Whitelist message should be above the total message
        if show_whitelisted_message {
            let whitelist_bar = bar_list.remove("Show whitelisted message").unwrap();
            let mut whitelist_chart = BarChart::new(whitelist_bar)
                .width(point_value)
                .name("Whitelisted Messages");

            if show_total_message {
                let total_message_bars = bar_list.remove("Show total message").unwrap();
                let total_message_chart = BarChart::new(total_message_bars)
                    .width(point_value)
                    .name("Total Messages");

                whitelist_chart = whitelist_chart.stack_on(&[&total_message_chart]);
                all_charts.push(total_message_chart);
            }
            all_charts.push(whitelist_chart);
        } else if show_total_message {
            let total_message_bars = bar_list.remove("Show total message").unwrap();
            let total_message_chart = BarChart::new(total_message_bars)
                .width(point_value)
                .name("Total Messages");
            all_charts.push(total_message_chart);
        };
        // Find the users that are added in the chart
        // All charts must be stacked by all the previous charts
        // Chart 3 will be stacked by chart 1 and 2
        if !bar_list.is_empty() {
            for (name, bar) in bar_list {
                let current_chart = BarChart::new(bar).width(point_value).name(name);

                if !all_charts.is_empty() {
                    let current_chart =
                        current_chart.stack_on(&all_charts.iter().collect::<Vec<&BarChart>>());
                    all_charts.push(current_chart);
                } else {
                    all_charts.push(current_chart);
                }
            }
        }

        Plot::new("Message Plot")
            .legend(Legend::default())
            .auto_bounds_x()
            .auto_bounds_y()
            .clamp_grid(true)
            .show(ui, |plot_ui| {
                for chart in all_charts {
                    plot_ui.bar_chart(chart);
                }
            });
    }
}
