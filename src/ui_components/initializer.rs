use eframe::{egui, App, Frame};
use egui::{
    vec2, Align, Button, CentralPanel, Context, FontData, FontDefinitions, FontFamily, Layout,
    Spinner, ViewportCommand, Visuals,
};
use egui_extras::{Size, StripBuilder};
use egui_modal::Modal;
use log::info;
use std::collections::{BTreeMap, HashMap};
use std::slice::IterMut;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::tg_handler::{start_process, NewProcess, ProcessResult, ProcessStart, TGClient};
use crate::ui_components::processor::{
    check_version, download_font, AppState, CounterCounts, ParsedChat, ProcessState, TabState,
};
use crate::ui_components::tab_ui::{
    BlacklistData, ChartsData, CounterData, SessionData, UserTableData, WhitelistData,
};
use crate::ui_components::TGKeys;
use crate::utils::{find_session_files, get_api_keys, get_font_data, get_theme_emoji};

pub struct MainWindow {
    pub app_state: AppState,
    pub tg_keys: TGKeys,
    pub counter: CounterData,
    table: Vec<UserTableData>,
    chart: Vec<ChartsData>,
    pub session: SessionData,
    pub whitelist: WhitelistData,
    pub blacklist: BlacklistData,
    tab_state: TabState,
    pub process_state: ProcessState,
    pub tg_sender: Sender<ProcessResult>,
    pub tg_receiver: Receiver<ProcessResult>,
    pub tg_clients: BTreeMap<String, TGClient>,
    pub incomplete_tg_client: Option<TGClient>,
    existing_sessions_checked: bool,
    is_light_theme: bool,
    pub is_processing: bool,
    new_version_body: Arc<Mutex<Option<String>>>,
    pub counter_chat_index: usize,
    pub table_chat_index: usize,
    pub chart_chat_index: usize,
    pub initial_chart_reset: bool,
    pub cancel_count: Arc<AtomicBool>,
}

impl Default for MainWindow {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            app_state: AppState::default(),
            tg_keys: TGKeys::default(),
            counter: CounterData::default(),
            // default value with an existing one with default
            table: vec![UserTableData::default()],
            chart: vec![ChartsData::default()],
            whitelist: WhitelistData::default(),
            blacklist: BlacklistData::default(),
            session: SessionData::default(),
            tab_state: TabState::Counter,
            process_state: ProcessState::Idle,
            tg_sender: sender,
            tg_receiver: receiver,
            tg_clients: BTreeMap::new(),
            incomplete_tg_client: None,
            existing_sessions_checked: false,
            is_light_theme: true,
            is_processing: false,
            new_version_body: Arc::new(Mutex::new(None)),
            counter_chat_index: 0,
            table_chat_index: 0,
            chart_chat_index: 0,
            initial_chart_reset: false,
            cancel_count: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        // If asked to close the app, search for any temporary client and if any, logout then close the window
        if ctx.input(|i| i.viewport().close_requested()) {
            let mut joins = Vec::new();
            for (_, client) in self.tg_clients.clone() {
                if client.is_temporary() {
                    let joiner =
                        thread::spawn(move || client.start_process(ProcessStart::SessionLogout));
                    joins.push(joiner);
                }
            }

            for join in joins {
                join.join().unwrap();
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            match self.app_state {
                AppState::LoadingFontsAPI => {
                    ctx.set_pixels_per_point(1.1);
                    self.set_fonts(ctx);

                    // If API keys are found, start the main UI otherwise show the UI to input the api keys
                    if get_api_keys().is_some() {
                        self.app_state = AppState::InitializedUI;
                    } else {
                        self.app_state = AppState::InputAPIKeys;
                    }
                }
                AppState::InputAPIKeys => self.show_tg_keys_ui(ui),
                AppState::InitializedUI => {
                    ui.horizontal(|ui| {
                        let (theme_emoji, hover_text) = get_theme_emoji(self.is_light_theme);

                        if ui
                            .add(Button::new(theme_emoji).frame(false))
                            .on_hover_text(hover_text)
                            .clicked()
                        {
                            self.switch_theme(ctx);
                        };

                        ui.separator();
                        let counter_tab =
                            ui.selectable_value(&mut self.tab_state, TabState::Counter, "Counter");
                        ui.separator();
                        let user_table_tab = ui.selectable_value(
                            &mut self.tab_state,
                            TabState::UserTable,
                            "User Table",
                        );
                        ui.separator();
                        let chart_tab =
                            ui.selectable_value(&mut self.tab_state, TabState::Charts, "Charts");
                        ui.separator();
                        let whitelist_tab = ui.selectable_value(
                            &mut self.tab_state,
                            TabState::Whitelist,
                            "Whitelist",
                        );
                        ui.separator();
                        let blacklist_tab = ui.selectable_value(
                            &mut self.tab_state,
                            TabState::Blacklist,
                            "Blacklist",
                        );
                        ui.separator();
                        let session_tab =
                            ui.selectable_value(&mut self.tab_state, TabState::Session, "Session");

                        // Set window size on tab switch
                        if counter_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(550.0, 400.0)));
                        }
                        if user_table_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(1250.0, 700.0)));
                        }
                        if chart_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(1000.0, 700.0)));
                        }
                        if whitelist_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(550.0, 600.0)));
                        }
                        if blacklist_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(550.0, 600.0)));
                        }
                        if session_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(550.0, 320.0)));
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
                                TabState::Blacklist => self.show_blacklist_ui(ui),
                                TabState::Session => self.show_session_ui(ui),
                            });
                            strip.cell(|ui| {
                                ui.separator();
                                let status_text = self.process_state.to_string();
                                ui.horizontal(|ui| {
                                    ui.label(status_text);
                                    if self.is_processing {
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.add(Spinner::new());
                                            },
                                        );
                                    };
                                });
                            });
                        });
                    if !self.existing_sessions_checked {
                        self.existing_sessions_checked = true;
                        let existing_sessions = find_session_files();
                        if !existing_sessions.is_empty() {
                            self.is_processing = true;
                            let sender_clone = self.tg_sender.clone();
                            let ctx_clone = ctx.clone();
                            thread::spawn(move || {
                                start_process(
                                    NewProcess::InitialSessionConnect(existing_sessions),
                                    sender_clone,
                                    ctx_clone,
                                );
                            });
                        }
                        let version_body = self.new_version_body.clone();
                        thread::spawn(|| {
                            check_version(version_body);
                        });
                    } else {
                        // At each UI loop, check on the receiver channel to check if there is anything
                        // limit total number of messages to check on the receiver to prevent frame freeze
                        for _ in 0..self.counter.get_comm_limit() {
                            if !self.check_receiver() {
                                break;
                            }
                        }
                    }

                    if self.new_version_body.lock().unwrap().is_some() {
                        let modal = Modal::new(ctx, "version_modal");

                        modal.show(|ui| {
                            modal.title(ui, "New Version Available");
                            modal.frame(ui, |ui| {
                                let modal_text =
                                    self.new_version_body.lock().unwrap().clone().unwrap();
                                modal.body(ui, modal_text);
                            });
                            modal.buttons(ui, |ui| {
                                if modal.button(ui, "Close").clicked() {
                                    *self.new_version_body.lock().unwrap() = None;
                                };
                                if modal.button(ui, "Update").clicked() {
                                    *self.new_version_body.lock().unwrap() = None;
                                    let _ = open::that(
                                        "https://github.com/TheRustyPickle/Talon/releases/latest",
                                    );
                                };
                            });
                        });
                        modal.open();
                    }
                }
            }
        });
    }
}

impl MainWindow {
    pub fn clear_overlap(&mut self, parsed: &HashMap<String, ParsedChat>) {
        self.counter_chat_index = 0;
        self.table_chat_index = 0;
        self.chart_chat_index = 0;
        for key in parsed.keys() {
            if self.counter.contains_chat(key) {
                let target_index = self.counter.chat_index(key);
                self.counter.remove_chat(target_index);
                self.table.remove(target_index);
                self.chart.remove(target_index);
            }
        }
    }
    pub fn reset_table(&mut self) {
        self.table = vec![UserTableData::default()];
    }

    pub fn reset_counts(&mut self) {
        self.counter.reset();
    }

    pub fn reset_chart(&mut self) {
        let mut chart = ChartsData::default();
        chart.reset_chart();

        self.chart = vec![chart];
    }

    /// Only called once after the Start button is pressed for the first time
    pub fn initial_chart_reset(&mut self) {
        if !self.initial_chart_reset {
            self.initial_chart_reset = true;
            let chart = &mut self.chart[0];
            chart.reset_chart();
        }
    }

    pub fn append_structs(&mut self, amount: usize, previous_amount: usize) {
        let amount = amount + previous_amount;
        while self.table.len() != amount {
            self.table.push(UserTableData::default())
        }

        while self.chart.len() != amount {
            let mut chart = ChartsData::default();
            chart.reset_chart();
            self.chart.push(chart)
        }

        while self.counter.counts.len() != amount {
            self.counter.counts.push(CounterCounts::default())
        }
    }

    /// Switch to light or dark mode
    fn switch_theme(&mut self, ctx: &Context) {
        if self.is_light_theme {
            ctx.set_visuals(Visuals::dark());
            self.is_light_theme = false;
        } else {
            ctx.set_visuals(Visuals::light());
            self.is_light_theme = true;
        }
    }

    /// Get all the added session names
    pub fn get_session_names(&self) -> Vec<String> {
        self.tg_clients.keys().map(ToString::to_string).collect()
    }

    /// Set the fonts for egui to use or download them if does not exist
    pub fn set_fonts(&self, ctx: &Context) {
        let font_data = get_font_data();

        if let Some((cjk, gentium)) = font_data {
            // Add the fonts on top of the default ones
            let font_cjk = FontData::from_owned(cjk);
            let font_gentium = FontData::from_owned(gentium);
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
        } else {
            info!("Could not find font data. Starting download");
            let ctx_clone = ctx.clone();

            thread::spawn(move || {
                download_font(ctx_clone);
            });
        }
    }

    /// Return the currently selected table data as mutable
    pub fn table(&mut self) -> &mut UserTableData {
        if self.counter.total_chats() > 1 {
            &mut self.table[self.table_chat_index]
        } else {
            &mut self.table[0]
        }
    }

    /// Return the currently selected table data as reference
    pub fn table_i(&self) -> &UserTableData {
        if self.counter.total_chats() > 1 {
            &self.table[self.table_chat_index]
        } else {
            &self.table[0]
        }
    }

    /// Returns the target table where new data should be added as mutable
    pub fn t_table(&mut self) -> &mut UserTableData {
        let ongoing = self.counter.ongoing_chat();
        &mut self.table[ongoing]
    }

    /// Return the currently selected chart data as mutable
    pub fn chart(&mut self) -> &mut ChartsData {
        if self.counter.total_chats() > 1 {
            &mut self.chart[self.chart_chat_index]
        } else {
            &mut self.chart[0]
        }
    }

    /// Return the currently selected chart data as reference
    pub fn chart_i(&self) -> &ChartsData {
        if self.counter.total_chats() > 1 {
            &self.chart[self.chart_chat_index]
        } else {
            &self.chart[0]
        }
    }

    /// Returns the target chart where new data should be added as mutable
    pub fn t_chart(&mut self) -> &mut ChartsData {
        let ongoing = self.counter.ongoing_chat();
        &mut self.chart[ongoing]
    }

    /// Return the currently selected chart data as mutable
    pub fn count(&mut self) -> &mut CounterCounts {
        if self.counter.total_chats() > 1 {
            &mut self.counter.counts[self.counter_chat_index]
        } else {
            &mut self.counter.counts[0]
        }
    }

    /// Returns the target chart where new data should be added as mutable
    pub fn t_count(&mut self) -> &mut CounterCounts {
        let ongoing = self.counter.ongoing_chat();
        &mut self.counter.counts[ongoing]
    }

    pub fn chart_all(&mut self) -> IterMut<ChartsData> {
        self.chart.iter_mut()
    }

    pub fn table_all(&mut self) -> IterMut<UserTableData> {
        self.table.iter_mut()
    }
}
