use eframe::{egui, App, Frame};
use egui::{vec2, Button, CentralPanel, Context, Visuals};
use egui_extras::{Size, StripBuilder};
use log::{debug, info};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::tg_handler::{start_tg_client, ProcessResult, ProcessStart, TGClient};
use crate::ui_components::{
    ChartsData, CounterData, ProcessState, SessionData, TabState, UserTableData, WhitelistData,
};
use crate::utils::{find_session_files, get_theme_emoji, parse_tg_chat};

pub struct MainWindow {
    pub counter_data: CounterData,
    pub user_table: UserTableData,
    pub charts_data: ChartsData,
    pub session_data: SessionData,
    pub whitelist_data: WhitelistData,
    tab_state: TabState,
    pub process_state: ProcessState,
    tg_sender: Sender<ProcessResult>,
    tg_receiver: Receiver<ProcessResult>,
    tg_clients: Vec<TGClient>,
    existing_sessions_checked: bool,
    is_light_theme: bool,
}

impl Default for MainWindow {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            counter_data: CounterData::default(),
            user_table: UserTableData::default(),
            charts_data: ChartsData::default(),
            whitelist_data: WhitelistData::default(),
            session_data: SessionData::default(),
            tab_state: TabState::Counter,
            process_state: ProcessState::Idle,
            tg_sender: sender,
            tg_receiver: receiver,
            tg_clients: Vec::new(),
            existing_sessions_checked: false,
            is_light_theme: true,
        }
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (theme_emoji, hover_text) = get_theme_emoji(self.is_light_theme);
                let theme_button = ui
                    .add(Button::new(theme_emoji).frame(false))
                    .on_hover_text(hover_text);

                if theme_button.clicked() {
                    self.switch_theme(ctx)
                }
                ui.separator();
                let counter_tab =
                    ui.selectable_value(&mut self.tab_state, TabState::Counter, "Counter");
                if counter_tab.clicked() {
                    frame.set_window_size(vec2(500.0, 300.0));
                }
                ui.separator();
                let user_table_tab =
                    ui.selectable_value(&mut self.tab_state, TabState::UserTable, "User Table");
                if user_table_tab.clicked() {
                    frame.set_window_size(vec2(850.0, 700.0));
                }
                ui.separator();
                ui.selectable_value(&mut self.tab_state, TabState::Charts, "Charts");
                ui.separator();
                ui.selectable_value(&mut self.tab_state, TabState::Whitelist, "Whitelist");
                ui.separator();
                ui.selectable_value(&mut self.tab_state, TabState::Session, "Session");
            });
            ui.separator();

            // Split the UI in 2 parts. First part takes all the remaining space to show the main UI
            // The second part takes a small amount of space to show the status text
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0))
                .size(Size::exact(20.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| match self.tab_state {
                        TabState::Counter => self.show_counter_ui(ui),
                        TabState::UserTable => self.show_user_table_ui(ui),
                        TabState::Charts => self.show_charts_ui(ui),
                        TabState::Whitelist => self.show_whitelist_ui(ui),
                        TabState::Session => self.show_session_ui(ui),
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        let status_text = self.process_state.to_string();
                        ui.label(status_text);
                    })
                });
            if !self.existing_sessions_checked {
                self.existing_sessions_checked = true;
                let existing_sessions = find_session_files();

                // All sessions gets the same sender and receiver to avoid having to check multiple channels
                for session_name in existing_sessions {
                    let sender_clone = self.tg_sender.clone();
                    let ctx_clone = ctx.clone();
                    thread::spawn(move || {
                        start_tg_client(session_name, sender_clone, ctx_clone);
                    });
                }
            } else {
                self.check_receiver()
            }
        });
    }
}

impl MainWindow {
    // TODO move this to a separate file
    fn check_receiver(&mut self) {
        while let Ok(data) = self.tg_receiver.try_recv() {
            match data {
                ProcessResult::NewClient(client) => {
                    self.tg_clients.push(client);
                    if self.tg_clients.len() == 1 {
                        self.update_counter_session()
                    }
                }
                ProcessResult::InvalidChat => {
                    info!("Invalid chat found")
                }
                ProcessResult::UnauthorizedClient => info!("The client is not authorized"),
                ProcessResult::CountingEnd => {
                    info!("Counting ended");
                    self.process_state = ProcessState::Idle;
                    self.counter_data.counting_ended()
                }
                ProcessResult::CountingMessage(message, start_from, end_at) => {
                    self.process_state = self.process_state.next_dot();
                    let sender_option = message.sender();
                    let mut sender_id = None;

                    if let Some(sender) = sender_option {
                        sender_id = Some(sender.id());
                        self.user_table.add_user(sender);
                    } else {
                        self.user_table.add_unknown_user();
                    }

                    self.user_table.count_user_message(sender_id, &message);

                    let total_user = self.user_table.get_total_user();
                    self.counter_data.set_total_user(total_user);

                    let total_to_iter = start_from - end_at;
                    let message_value = 100.0 / total_to_iter as f32;
                    let current_message_number = message.id();

                    let total_processed = start_from - current_message_number;
                    let processed_percentage = if total_processed != 0 {
                        total_processed as f32 * message_value
                    } else {
                        message_value
                    };
                    self.counter_data.add_one_total_message();
                    debug!(
                        "Bar percentage: {}. Current message: {current_message_number} Total message: {}, Started from: {}",
                        processed_percentage, total_to_iter, start_from
                    );
                    self.counter_data
                        .set_bar_percentage(processed_percentage / 100.0);
                }
            }
        }
    }

    fn update_counter_session(&mut self) {
        let first_session = self.tg_clients.first().unwrap().name();
        self.counter_data.update_selected_session(first_session)
    }

    pub fn start_counting(&mut self) {
        let start_from = self.counter_data.get_start_from();
        let end_at = self.counter_data.get_end_at();

        let (start_chat, start_num) = parse_tg_chat(start_from);
        let (end_chat, end_num) = parse_tg_chat(end_at);

        if start_chat.is_none() {
            self.process_state = ProcessState::InvalidStartChat;
            return;
        }

        let start_chat = start_chat.unwrap();

        if let Some(end_chat) = end_chat {
            if end_chat != start_chat {
                self.process_state = ProcessState::UnmatchedChat;
                return;
            }
        }

        if start_num.is_some() && end_num.is_some() {
            if start_num.unwrap() < end_num.unwrap() {
                self.process_state = ProcessState::SmallerStartNumber;
                return;
            }
        }

        info!("Starting counting");
        self.process_state = ProcessState::Counting(0);
        self.counter_data.counting_started();

        let selected_client = self.counter_data.get_selected_session();

        for client in self.tg_clients.iter() {
            if client.name() == selected_client {
                let client = client.clone();
                thread::spawn(move || {
                    client.start_process(ProcessStart::StartCount(start_chat, start_num, end_num));
                });

                break;
            }
        }
    }

    fn switch_theme(&mut self, ctx: &Context) {
        if self.is_light_theme {
            ctx.set_visuals(Visuals::dark());
            self.is_light_theme = false;
        } else {
            ctx.set_visuals(Visuals::light());
            self.is_light_theme = true;
        }
    }
}
