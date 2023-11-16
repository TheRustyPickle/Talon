use eframe::{egui, App, Frame};
use egui::{vec2, Align, Button, CentralPanel, Context, Layout, Spinner, Visuals};
use egui_extras::{Size, StripBuilder};
use log::info;
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
    pub tg_receiver: Receiver<ProcessResult>,
    pub tg_clients: Vec<TGClient>,
    existing_sessions_checked: bool,
    is_light_theme: bool,
    pub is_processing: bool,
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
            is_processing: false,
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
                    frame.set_window_size(vec2(1000.0, 700.0));
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
                        ui.horizontal(|ui| {
                            ui.label(status_text);
                            if self.is_processing {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add(Spinner::new());
                                });
                            };
                        });
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
    pub fn update_counter_session(&mut self) {
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

        if let (Some(start_num), Some(end_num)) = (start_num, end_num) {
            if start_num < end_num {
                self.process_state = ProcessState::SmallerStartNumber;
                return;
            }
        }

        info!("Starting counting");
        self.user_table.clear_row_data();
        self.process_state = ProcessState::Counting(0);
        self.counter_data.counting_started();
        self.is_processing = true;

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
