use eframe::egui::{
    Align, Button, ComboBox, Grid, Label, Layout, ProgressBar, TextEdit, Ui, ViewportCommand, vec2,
};
use log::info;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

use crate::tg_handler::ProcessStart;
use crate::ui_components::MainWindow;
use crate::ui_components::processor::{CounterCounts, ParsedChat, ProcessState};
use crate::utils::{chat_to_text, parse_chat_details};

const LIMIT_SELECTION: [&str; 5] = ["30", "40", "50", "80", "100"];

#[derive(Clone)]
pub struct CounterData {
    session_index: usize,
    start_from: String,
    end_at: String,
    pub counts: Vec<CounterCounts>,
    bar_percentage: f32,
    use_all_sessions: bool,
    counting: bool,
    session_count: usize,
    session_percentage: HashMap<String, f32>,
    comm_limit: usize,
    parsed_chat_list: HashMap<String, ParsedChat>,
    chat_list: Vec<String>,
    ongoing_chat: usize,
    detected_chat: String,
    retain_data: bool,
}

impl Default for CounterData {
    fn default() -> Self {
        Self {
            session_index: usize::default(),
            start_from: String::default(),
            end_at: String::default(),
            counts: vec![CounterCounts::default()],
            bar_percentage: f32::default(),
            use_all_sessions: true,
            counting: bool::default(),
            session_count: usize::default(),
            session_percentage: HashMap::default(),
            comm_limit: 4,
            parsed_chat_list: HashMap::default(),
            chat_list: Vec::default(),
            ongoing_chat: usize::default(),
            detected_chat: String::default(),
            retain_data: true,
        }
    }
}

impl CounterData {
    pub fn reset(&mut self) {
        self.counts = vec![CounterCounts::default()];
        self.chat_list = Vec::new();
        self.ongoing_chat = 0;
        self.session_percentage = HashMap::new();
    }
    pub fn ongoing_chat(&self) -> usize {
        self.ongoing_chat
    }

    pub fn add_session(&mut self, to_add: String) {
        self.session_percentage.entry(to_add).or_default();
    }

    pub fn set_session_percentage(&mut self, key: &str, value: f32) {
        *self.session_percentage.get_mut(key).unwrap() = value;
        let mut progress_bar = 0.0;

        for val in self.session_percentage.values() {
            progress_bar += val;
        }

        progress_bar /= self.session_percentage.len() as f32;
        self.bar_percentage = progress_bar;
    }

    pub fn session_remaining(&self) -> usize {
        self.session_count
    }
    pub fn set_session_count(&mut self, count: usize) {
        self.session_count = count;
    }

    pub fn reduce_session(&mut self) {
        self.session_count -= 1;
    }

    fn get_start_from(&mut self) -> String {
        let new_string = self
            .start_from
            .split_inclusive(|c: char| c.is_whitespace())
            .map(|s| s.replace('\n', " "))
            .collect::<String>();
        self.start_from = new_string;
        self.start_from.clone()
    }

    fn get_end_at(&mut self) -> String {
        let new_string = self
            .end_at
            .split_inclusive(|c: char| c.is_whitespace())
            .map(|s| s.replace('\n', " "))
            .collect::<String>();
        self.end_at = new_string;
        self.end_at.clone()
    }

    fn counting_started(&mut self) {
        self.counting = true;
        self.bar_percentage = 0.0;
        self.session_count = 0;
        self.session_percentage.clear();
    }

    pub fn counting_ended(&mut self) {
        if self.counting {
            self.counting = false;
            self.bar_percentage = 1.0;
        }
    }

    pub fn set_bar_percentage(&mut self, percentage: f32) {
        self.bar_percentage = percentage;
    }

    pub fn get_comm_limit(&self) -> usize {
        LIMIT_SELECTION[self.comm_limit].parse().unwrap()
    }

    pub fn total_parsed_chats(&self) -> usize {
        self.parsed_chat_list.len()
    }

    pub fn total_chats(&self) -> usize {
        self.chat_list.len()
    }

    fn set_parsed_chat(&mut self, chat_list: HashMap<String, ParsedChat>) {
        self.parsed_chat_list = chat_list;
    }

    fn get_parsed_chat(&mut self) -> Option<ParsedChat> {
        if self.total_parsed_chats() > 0 {
            let mut first_key = String::new();
            for key in self.parsed_chat_list.keys() {
                first_key = key.to_string();
            }
            let data = self.parsed_chat_list.remove(&first_key).unwrap();
            Some(data)
        } else {
            None
        }
    }

    pub fn counting(&self) -> bool {
        self.counting
    }

    pub fn add_to_chat(&mut self, name: String) {
        self.chat_list.push(name);
    }

    pub fn set_ongoing_chat(&mut self, index: usize) {
        self.ongoing_chat = index;
    }

    pub fn increment_ongoing(&mut self) {
        self.ongoing_chat += 1;
    }

    pub fn get_chat_list(&self) -> Vec<String> {
        self.chat_list.clone()
    }

    pub fn contains_chat(&self, chat: &String) -> bool {
        self.chat_list.contains(chat)
    }

    pub fn chat_index(&self, chat: &String) -> usize {
        for (index, ch) in self.chat_list.iter().enumerate() {
            if ch == chat {
                return index;
            }
        }
        unreachable!()
    }

    pub fn remove_chat(&mut self, index: usize) {
        self.chat_list.remove(index);
        self.counts.remove(index);
    }

    pub fn selected_chat_name(&self, index: usize) -> String {
        self.chat_list[index].clone()
    }
}

impl MainWindow {
    pub fn show_counter_ui(&mut self, ui: &mut Ui) {
        Grid::new("Counter Grid")
            .num_columns(2)
            .spacing([5.0, 10.0])
            .show(ui, |ui| self.show_grid_data(ui));

        ui.add_space(10.0);
        ui.label(self.counter.detected_chat.clone());
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            Grid::new("Main")
                .num_columns(2)
                .spacing([5.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Messages Checked:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.count().total_message));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Whitelisted Messages:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.count().whitelisted_message));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Users Found:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.count().total_user));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Whitelisted Users:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.count().total_whitelisted()));
                    });

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Deleted Message:")
                    });
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.label(format!("{}", &mut self.count().deleted_message));
                    });

                    ui.end_row();
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(80.0);

                if self.is_processing && self.counter.counting() {
                    let cancel_button = ui.add_sized([80.0, 40.0], Button::new("Cancel"));
                    if cancel_button.clicked() {
                        self.cancel_count();
                    }
                } else if self.is_processing {
                    ui.add_enabled(false, Button::new("Start").min_size(vec2(80.0, 40.0)));
                } else {
                    let start_button = ui.add_sized([80.0, 40.0], Button::new("Start"));
                    if start_button.clicked() {
                        self.start_counting();
                    }
                }
            });
        });

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            let progress_bar = ProgressBar::new(self.counter.bar_percentage)
                .show_percentage()
                .animate(self.counter.counting);
            ui.add(progress_bar);
        });

        self.counter.detected_chat =
            chat_to_text(&self.counter.get_start_from(), &self.counter.get_end_at());
    }

    fn show_grid_data(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Selected Chat:"));
        });
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let (values, len) = {
                let names = self.counter.get_chat_list();

                if names.is_empty() {
                    (vec!["No chat available".to_string()], 0)
                } else {
                    let total_val = names.len();
                    (names, total_val)
                }
            };
            ComboBox::from_id_salt("Chat Box").show_index(
                ui,
                &mut self.counter_chat_index,
                len,
                |i| &values[i],
            );
            ui.separator();
            ui.checkbox(&mut self.counter.retain_data, "Retain previous data")
                .on_hover_text("Whether to retain all previous data on a new counting session");
        });
        ui.end_row();

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Selected Session:"));
        });
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let values = {
                let names = self.get_session_names();

                if names.is_empty() {
                    vec!["No Session Found".to_string()]
                } else {
                    names
                }
            };
            ComboBox::from_id_salt("Session Box").show_index(
                ui,
                &mut self.counter.session_index,
                self.tg_clients.len(),
                |i| &values[i],
            );
            ui.checkbox(&mut self.counter.use_all_sessions, "Use all sessions")
                .on_hover_text(
                    "Whether to use all the available sessions for counting
            
Automatically divides the tasks among sessions, dramatically increasing speed.

How to get more sessions?
Login to one or more accounts multiple times with different session names!",
                );

            ui.separator();

            ui.label("Limit:");

            ComboBox::from_id_salt("Limit Box")
                .width(60.0)
                .show_index(
                    ui,
                    &mut self.counter.comm_limit,
                    LIMIT_SELECTION.len(),
                    |i| LIMIT_SELECTION[i],
                )
                .on_hover_text(
                    "When counting how many messages to count in each frame load? Default: 100

Higher can increase the counting speed but flood wait will also be more
visible in the UI.

Info: Each session can count about 3000 messages before flood wait is triggered.",
                );
        });
        ui.end_row();

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Starting Point:"));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let clear_paste_button = if self.counter.start_from.is_empty() {
                ui.add(Button::new("Paste"))
                    .on_hover_text("Paste copied content")
            } else {
                ui.add(Button::new("Clear"))
                    .on_hover_text("Clear text box content")
            };

            let target_textbox = ui
                .add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut self.counter.start_from)
                        .hint_text("https://t.me/chat_name/1234"),
                )
                .on_hover_text(
                    "The message link from where message counting should start. 
Multiple points can be inserted separated by a space

Multiple input format is supported:

(message number optional)
1. https://t.me/chat_name/1234 https://t.me/chat_name_2/1234
2. t.me/chat_name/1234 t.me/chat_name_2/1234
3. @chat_name/1234 @chat_name_2/1234
4. chat_name/1234 chat_name_2/1234

If message number is not specified, starts from the latest message.
Starting message number will always be bigger than the ending message.
To count all messages in a chat, paste the latest message link.",
                );
            if clear_paste_button.clicked() {
                if self.counter.start_from.is_empty() {
                    target_textbox.request_focus();
                    ui.ctx().send_viewport_cmd(ViewportCommand::RequestPaste);
                } else {
                    self.counter.start_from = String::new();
                }
            }
        });

        ui.end_row();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(Label::new("Ending Point:"));
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let clear_paste_button = if self.counter.end_at.is_empty() {
                ui.add(Button::new("Paste"))
                    .on_hover_text("Paste copied content")
            } else {
                ui.add(Button::new("Clear"))
                    .on_hover_text("Clear text box content")
            };

            let target_textbox = ui
                .add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut self.counter.end_at)
                        .hint_text("(Optional) https://t.me/chat_name/1234"),
                )
                .on_hover_text(
                    "Optional message link where counting will stop after including it. 
Multiple points can be inserted separated by a space

Multiple input format is supported:

(message number optional)
1. https://t.me/chat_name/1234
2. t.me/chat_name/1234
3. @chat_name/1234
4. chat_name/1234

If message number is not specified or is empty, counts all messages.
Ending message number will always be smaller than the starting message.
To count all messages in a chat, paste the very first message link or keep it empty.",
                );
            if clear_paste_button.clicked() {
                if self.counter.end_at.is_empty() {
                    target_textbox.request_focus();
                    ui.ctx().send_viewport_cmd(ViewportCommand::RequestPaste);
                } else {
                    self.counter.end_at = String::new();
                }
            }
        });
        ui.end_row();
    }

    fn start_counting(&mut self) {
        let selected_client = self.get_selected_session();

        if selected_client.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        let start_from = self.counter.get_start_from();
        let end_at = self.counter.get_end_at();

        let parsed_chat_data = parse_chat_details(&start_from, &end_at);

        if parsed_chat_data.is_empty() {
            self.process_state = ProcessState::InvalidStartChat;
            return;
        }

        let total_parsed = parsed_chat_data.len();

        if self.counter.retain_data {
            self.clear_overlap(&parsed_chat_data);
        }

        self.counter.set_parsed_chat(parsed_chat_data);

        if !self.counter.retain_data {
            self.reset_counts();
            self.reset_table();
            self.reset_chart();
        }
        self.initial_chart_reset();
        self.append_structs(total_parsed, self.counter.total_chats());
        self.process_next_count();
    }

    pub fn process_next_count(&mut self) {
        let target_chat = self.counter.get_parsed_chat();

        let Some(chat) = target_chat else {
            info!("No other chat to process.");
            self.stop_process();
            self.process_state = ProcessState::Idle;
            return;
        };

        self.counter.add_to_chat(chat.name());
        let ongoing_index = self.counter.total_chats() - 1;

        self.counter.set_ongoing_chat(ongoing_index);

        info!("Starting counting for {}", chat.name());

        let selected_client = self.get_selected_session();

        self.process_state = ProcessState::Counting(0);
        self.counter.counting_started();
        self.is_processing = true;

        let chat_name = chat.name();
        let start_num = chat.start_point();
        let end_num = chat.end_point();

        let client = self.tg_clients.get(&selected_client).unwrap().clone();

        if self.counter.use_all_sessions && self.tg_clients.len() > 1 {
            self.runtime.spawn(async move {
                client
                    .start_process(ProcessStart::CheckChatExistence(
                        chat_name, start_num, end_num,
                    ))
                    .await;
            });
        } else {
            let cancel = self.cancel_count.clone();
            self.runtime.spawn(async move {
                client
                    .start_process(ProcessStart::StartCount(
                        chat_name, start_num, end_num, false, cancel,
                    ))
                    .await;
            });
        }
    }

    /// Returns the session name that is selected on the combo box
    pub fn get_selected_session(&self) -> String {
        let all_sessions = self.get_session_names();
        let selected_index = self.counter.session_index;
        if let Some(session) = all_sessions.get(selected_index) {
            session.to_owned()
        } else {
            String::new()
        }
    }

    fn cancel_count(&mut self) {
        self.cancel_count.store(true, Ordering::Release);
    }
}
