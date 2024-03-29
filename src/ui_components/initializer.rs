use eframe::{egui, App, Frame};
use egui::{
    vec2, Align, Button, CentralPanel, Context, FontData, FontDefinitions, FontFamily, Layout,
    Spinner, ViewportCommand, Visuals,
};
use egui_extras::{Size, StripBuilder};
use egui_modal::Modal;
use log::info;
use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::tg_handler::{start_process, NewProcess, ProcessResult, ProcessStart, TGClient};
use crate::ui_components::processor::{
    check_version, download_font, AppState, ProcessState, TabState,
};
use crate::ui_components::tab_ui::{
    ChartsData, CounterData, SessionData, UserTableData, WhitelistData,
};
use crate::ui_components::TGKeys;
use crate::utils::{find_session_files, get_api_keys, get_font_data, get_theme_emoji};

pub struct MainWindow {
    pub app_state: AppState,
    pub tg_keys: TGKeys,
    pub counter_data: CounterData,
    pub user_table: UserTableData,
    pub charts_data: ChartsData,
    pub session_data: SessionData,
    pub whitelist_data: WhitelistData,
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
}

impl Default for MainWindow {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            app_state: AppState::default(),
            tg_keys: TGKeys::default(),
            counter_data: CounterData::default(),
            user_table: UserTableData::default(),
            charts_data: ChartsData::default(),
            whitelist_data: WhitelistData::default(),
            session_data: SessionData::default(),
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
                        let session_tab =
                            ui.selectable_value(&mut self.tab_state, TabState::Session, "Session");

                        // Set window size on tab switch
                        if counter_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(550.0, 350.0)));
                        }
                        if user_table_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(1250.0, 700.0)));
                        }
                        if session_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(500.0, 320.0)));
                        }
                        if whitelist_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(500.0, 600.0)));
                        }
                        if chart_tab.clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(1000.0, 700.0)));
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
                        for _ in 0..self.counter_data.get_comm_limit() {
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
}
