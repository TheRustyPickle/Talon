use eframe::egui::{vec2, Align, Button, Checkbox, Context, Grid, Label, Layout, TextEdit, Ui};
use grammers_client::types::{LoginToken, PasswordToken};
use std::sync::Arc;
use std::thread;
use tokio::sync::Mutex;

use crate::tg_handler::{start_process, NewProcess, ProcessStart};
use crate::ui_components::processor::ProcessState;
use crate::ui_components::MainWindow;

#[derive(Default)]
pub struct SessionData {
    session_name: String,
    phone_number: String,
    tg_code: String,
    tg_password: String,
    is_temporary: bool,
    password_token: Option<Arc<Mutex<PasswordToken>>>,
    tg_code_token: Option<Arc<Mutex<LoginToken>>>,
}

impl SessionData {
    fn get_session_name(&self) -> String {
        self.session_name.to_string()
    }

    fn get_phone_number(&self) -> String {
        self.phone_number.replace('+', "")
    }

    fn get_password(&self) -> String {
        self.tg_password.to_string()
    }

    fn get_is_temporary(&self) -> bool {
        self.is_temporary
    }

    pub fn set_login_token(&mut self, token: LoginToken) {
        self.tg_code_token = Some(Arc::new(Mutex::new(token)));
    }

    pub fn set_password_token(&mut self, token: PasswordToken) {
        self.password_token = Some(Arc::new(Mutex::new(token)));
    }

    fn get_tg_code(&self) -> String {
        self.tg_code.to_string()
    }

    fn get_tg_code_token(&self) -> Arc<Mutex<LoginToken>> {
        self.tg_code_token.clone().unwrap()
    }

    fn get_password_token(&self) -> Arc<Mutex<PasswordToken>> {
        self.password_token.clone().unwrap()
    }

    pub fn reset_data(&mut self) {
        self.session_name = String::new();
        self.phone_number = String::new();
        self.tg_code = String::new();
        self.tg_password = String::new();
        self.is_temporary = false;
        self.password_token = None;
        self.tg_code_token = None;
    }
}

impl MainWindow {
    pub fn show_session_ui(&mut self, ui: &mut Ui) {
        Grid::new("Session Grid")
            .num_columns(2)
            .spacing([5.0, 10.0])
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Label::new("Session Name:"));
                });
                let text_edit = TextEdit::singleline(&mut self.session_data.session_name)
                    .hint_text("My TG Session Name");
                if self.session_data.tg_code_token.is_some()
                    || self.session_data.password_token.is_some()
                {
                    ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                } else {
                    let hover_text = if self.session_data.is_temporary {
                        "A name for the temporary session"
                    } else {
                        "Save the session in a file with this name for later access"
                    };
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_sized(ui.available_size(), text_edit)
                            .on_hover_text(hover_text);
                    });
                }

                ui.end_row();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Label::new("Phone Number:"));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text_edit = TextEdit::singleline(&mut self.session_data.phone_number)
                        .hint_text("International format: +1234567890");
                    if self.session_data.password_token.is_some()
                        || self.session_data.tg_code_token.is_some()
                    {
                        ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                    } else {
                        ui.add_sized(ui.available_size(), text_edit).on_hover_text(
                            "The phone number of your Telegram in international format",
                        );
                    }
                });

                ui.end_row();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Label::new("Login Code:"));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let button = Button::new("Send Code");
                    if self.session_data.phone_number.is_empty()
                        || self.session_data.password_token.is_some()
                        || self.session_data.session_name.is_empty()
                    {
                        ui.add_enabled(false, button);
                    } else if ui
                        .add(button)
                        .on_hover_text("Send the code to login to the account")
                        .clicked()
                    {
                        self.request_login_code(ui.ctx().clone());
                    }

                    let text_edit =
                        TextEdit::singleline(&mut self.session_data.tg_code).hint_text("12345");
                    if self.session_data.tg_code_token.is_some() {
                        ui.add_sized(ui.available_size(), text_edit).on_hover_text(
                            "The Telegram login code either sent to TG or to the phone number",
                        );
                    } else {
                        ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                    }
                });

                ui.end_row();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Label::new("Login Password:"));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text_edit = TextEdit::singleline(&mut self.session_data.tg_password)
                        .password(true)
                        .hint_text("Your Telegram login password");
                    if self.session_data.password_token.is_some() {
                        ui.add_sized(ui.available_size(), text_edit)
                            .on_hover_text("Telegram login password");
                    } else {
                        ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                    }
                });

                ui.end_row();

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Label::new("Temporary:"));
                });
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    if self.session_data.tg_code_token.is_some() || self.is_processing {
                        ui.add_enabled(
                            false,
                            Checkbox::new(
                                &mut self.session_data.is_temporary,
                                "Do not create a session file",
                            ),
                        );
                    } else {
                        ui.checkbox(
                            &mut self.session_data.is_temporary,
                            "Do not create a session file",
                        )
                        .on_hover_text(
                            "Create a new temporary session?
If yes, it will try to log out before the app is closed and no session file will be created",
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button("Reset Session")
                            .on_hover_text("Reset and clear new session creation data")
                            .clicked()
                        {
                            self.session_data.reset_data();
                            self.incomplete_tg_client = None;
                        }
                    })
                });
            });

        ui.add_space(40.0);

        ui.vertical_centered(|ui| {
            let button = Button::new("Create Session").min_size(vec2(100.0, 40.0));
            if self.is_processing
                || self.session_data.phone_number.is_empty()
                || self.session_data.session_name.is_empty()
                || self.session_data.tg_code.is_empty()
                || (self.session_data.password_token.is_some()
                    && self.session_data.tg_password.is_empty())
            {
                ui.add_enabled(false, button);
            } else if ui
                .add(button)
                .on_hover_text("Try to create a new session with the Telegram login info")
                .clicked()
            {
                if self.session_data.password_token.is_some() {
                    self.sign_in_password();
                } else {
                    self.sign_in_code();
                }
            }
        });
    }

    /// Starts a thread to send a Telegram login code to the phone number
    fn request_login_code(&mut self, context: Context) {
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

    /// Starts a thread to try to log in with the given login code
    fn sign_in_code(&mut self) {
        self.is_processing = true;
        self.process_state = ProcessState::LogInWithCode;

        let code = self.session_data.get_tg_code();
        let token = self.session_data.get_tg_code_token();

        let client = self.incomplete_tg_client.clone().unwrap();
        thread::spawn(move || client.start_process(ProcessStart::SignInCode(token, code)));
    }

    /// Starts a thread to try to log in with the given login password
    fn sign_in_password(&mut self) {
        self.is_processing = true;
        self.process_state = ProcessState::LogInWithPassword;

        let password = self.session_data.get_password();
        let token = self.session_data.get_password_token();

        let client = self.incomplete_tg_client.clone().unwrap();
        thread::spawn(move || client.start_process(ProcessStart::SignInPasswords(token, password)));
    }
}
