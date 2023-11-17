use eframe::egui::{vec2, Align, Button, Checkbox, Grid, Label, Layout, TextEdit, Ui};
use grammers_client::types::{LoginToken, PasswordToken};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub fn get_session_name(&self) -> String {
        self.session_name.to_string()
    }

    pub fn get_phone_number(&self) -> String {
        self.phone_number.replace('+', "")
    }

    pub fn get_password(&self) -> String {
        self.tg_password.to_string()
    }

    pub fn get_is_temporary(&self) -> bool {
        self.is_temporary
    }

    pub fn set_login_token(&mut self, token: LoginToken) {
        self.tg_code_token = Some(Arc::new(Mutex::new(token)));
    }

    pub fn set_password_token(&mut self, token: PasswordToken) {
        self.password_token = Some(Arc::new(Mutex::new(token)));
    }

    pub fn get_tg_code(&self) -> String {
        self.tg_code.to_string()
    }

    pub fn get_tg_code_token(&self) -> Arc<Mutex<LoginToken>> {
        self.tg_code_token.clone().unwrap()
    }

    pub fn get_password_token(&self) -> Arc<Mutex<PasswordToken>> {
        self.password_token.clone().unwrap()
    }

    pub fn reset_data(&mut self) {
        self.session_name = "".to_string();
        self.phone_number = "".to_string();
        self.tg_code = "".to_string();
        self.tg_password = "".to_string();
        self.is_temporary = false;
        self.password_token = None;
        self.tg_code_token = None;
    }
}

impl MainWindow {
    pub fn show_session_ui(&mut self, ui: &mut Ui) {
        Grid::new("my grid")
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
                    let hover_text = if !self.session_data.is_temporary {
                        "Save the session in a file with this name for later access"
                    } else {
                        "A name for the temporary session"
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
                        self.request_login_code(ui.ctx().clone())
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
                            self.session_data.reset_data()
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
                    self.sign_in_password()
                } else {
                    self.sign_in_code();
                }
            }
        });
    }
}
