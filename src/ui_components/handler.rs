use eframe::App;
use eframe::{egui, Frame};
use egui::{CentralPanel, Context};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tracing::info;

use crate::tg_handler::{start_tg_client, ProcessResult, ProcessStart, TGClient};
use crate::ui_components::CounterData;
use crate::utils::{find_session_files, parse_tg_chat};

pub struct MainWindow {
    pub counter_data: CounterData,
    tab_state: TabState,
    tg_sender: Sender<ProcessResult>,
    tg_receiver: Receiver<ProcessResult>,
    tg_clients: Vec<TGClient>,
    existing_sessions_checked: bool,
}

#[derive(PartialEq)]
enum TabState {
    Counter,
    UserTable,
    Charts,
    Whitelist,
    Sessions,
}

impl Default for MainWindow {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            counter_data: CounterData::default(),
            tab_state: TabState::Counter,
            tg_sender: sender,
            tg_receiver: receiver,
            tg_clients: Vec::new(),
            existing_sessions_checked: false,
        }
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab_state, TabState::Counter, "Counter");
                ui.selectable_value(&mut self.tab_state, TabState::UserTable, "User Table");
                ui.selectable_value(&mut self.tab_state, TabState::Charts, "Charts");
                ui.selectable_value(&mut self.tab_state, TabState::Whitelist, "Whitelist");
                ui.selectable_value(&mut self.tab_state, TabState::Sessions, "Sessions");
            });
            ui.separator();
            match self.tab_state {
                TabState::Counter => self.show_counter_ui(ui),
                _ => {}
            }

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
    fn check_receiver(&mut self) {
        match self.tg_receiver.try_recv() {
            Ok(data) => match data {
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
            },
            Err(_) => {}
        }
    }

    fn update_counter_session(&mut self) {
        let first_session = self.tg_clients.first().unwrap().name();
        self.counter_data.update_selected_session(first_session)
    }

    pub fn start_counting(&mut self) {
        info!("Starting counting");
        let start_from = self.counter_data.get_start_from();
        let end_at = self.counter_data.get_end_at();

        let (start_chat, start_num) = parse_tg_chat(start_from);
        let (end_chat, end_num) = parse_tg_chat(end_at);

        if start_chat.is_none() {
            return;
        }
        let start_chat = start_chat.unwrap();

        if let Some(end_chat) = end_chat {
            if end_chat != start_chat {
                return;
            }
        }

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
}
