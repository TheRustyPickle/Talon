use eframe::{egui, App, Frame};
use egui::{
    vec2, Align, Button, CentralPanel, Context, FontData, FontDefinitions, FontFamily, Layout,
    Spinner, Visuals,
};
use egui_extras::{Size, StripBuilder};
use log::info;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::tg_handler::{start_process, NewProcess, ProcessResult, ProcessStart, TGClient};
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
    pub tg_clients: HashMap<String, TGClient>,
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
            tg_clients: HashMap::new(),
            existing_sessions_checked: false,
            is_light_theme: true,
            is_processing: false,
        }
    }
}

impl App for MainWindow {
    fn on_close_event(&mut self) -> bool {
        self.process_state = ProcessState::LoggingOut;
        self.is_processing = true;
        let mut joins = Vec::new();
        for (_, client) in self.tg_clients.clone().into_iter() {
            if client.is_temporary() {
                let joiner =
                    thread::spawn(move || client.start_process(ProcessStart::SessionLogout));
                joins.push(joiner)
            }
        }

        for join in joins {
            join.join().unwrap();
        }

        true
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_pixels_per_point(1.1);

        let font_data_cjk = include_bytes!("../../fonts/NotoSansCJK-Regular.ttc");
        let font_data_gentium = include_bytes!("../../fonts/GentiumBookPlus-Regular.ttf");

        let font_cjk = FontData::from_static(font_data_cjk);
        let font_gentium = FontData::from_static(font_data_gentium);

        let mut font_definitions = FontDefinitions::default();

        font_definitions
            .font_data
            .insert("NotoSansCJK".to_owned(), font_cjk);
        font_definitions
            .font_data
            .insert("GentiumBookPlus".to_owned(), font_gentium);

        font_definitions
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .extend(["NotoSansCJK".to_owned(), "GentiumBookPlus".to_owned()]);

        ctx.set_fonts(font_definitions);

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
                ui.separator();
                let user_table_tab =
                    ui.selectable_value(&mut self.tab_state, TabState::UserTable, "User Table");
                ui.separator();
                ui.selectable_value(&mut self.tab_state, TabState::Charts, "Charts");
                ui.separator();
                ui.selectable_value(&mut self.tab_state, TabState::Whitelist, "Whitelist");
                ui.separator();
                let session_tab =
                    ui.selectable_value(&mut self.tab_state, TabState::Session, "Session");

                if counter_tab.clicked() {
                    frame.set_window_size(vec2(500.0, 300.0));
                }
                if user_table_tab.clicked() {
                    frame.set_window_size(vec2(1000.0, 700.0));
                }
                if session_tab.clicked() {
                    frame.set_window_size(vec2(500.0, 300.0));
                }
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
                        start_process(
                            NewProcess::InitialSessionConnect(session_name),
                            sender_clone,
                            ctx_clone,
                        );
                    });
                }
            } else {
                self.check_receiver()
            }
        });
    }
}

impl MainWindow {
    // TODO update to combo box
    pub fn update_counter_session(&mut self) {
        if let Some((session_name, _)) = self.tg_clients.iter().next() {
            self.counter_data
                .update_selected_session(session_name.to_owned());
        }
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

        let selected_client = self.counter_data.get_selected_session();

        if selected_client.is_empty() {
            self.process_state = ProcessState::EmptySelectedSession;
            return;
        }

        info!("Starting counting");
        self.user_table.clear_row_data();
        self.process_state = ProcessState::Counting(0);
        self.counter_data.counting_started();
        self.is_processing = true;

        let client = self.tg_clients.get(&selected_client);

        if let Some(client) = client {
            let client = client.clone();
            thread::spawn(move || {
                client.start_process(ProcessStart::StartCount(start_chat, start_num, end_num));
            });
        } else {
            panic!("TO be handled")
        }
    }

    pub fn request_login_code(&mut self, context: Context) {
        let phone_num = self.session_data.get_phone_number();
        let session_name = self.session_data.get_session_name();
        let is_temporary = self.session_data.get_is_temporary();

        let sender_clone = self.tg_sender.clone();

        self.is_processing = true;
        self.process_state = ProcessState::SendingTGCode;

        thread::spawn(move || {
            start_process(
                NewProcess::SendLoginCode(session_name, phone_num, is_temporary),
                sender_clone,
                context,
            );
        });
    }

    pub fn sign_in_code(&mut self) {
        self.is_processing = true;
        self.process_state = ProcessState::LogInWithCode;

        let code = self.session_data.get_tg_code();
        let token = self.session_data.get_tg_code_token();
        let session_name = self.session_data.get_session_name();

        let client = self.tg_clients.get(&session_name);
        if let Some(client) = client {
            let client = client.clone();
            thread::spawn(move || client.start_process(ProcessStart::SignInCode(token, code)));
        }
    }

    pub fn sign_in_password(&mut self) {
        self.is_processing = true;
        self.process_state = ProcessState::LogInWithPassword;

        let password = self.session_data.get_password();
        let token = self.session_data.get_password_token();
        let session_name = self.session_data.get_session_name();

        let client = self.tg_clients.get(&session_name);
        if let Some(client) = client {
            let client = client.clone();
            thread::spawn(move || {
                client.start_process(ProcessStart::SignInPasswords(token, password))
            });
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
