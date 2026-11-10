use eframe::egui::{Align, Button, Checkbox, Context, Grid, Label, Layout, TextEdit, Ui, vec2};
use grammers_client::client::{LoginToken, PasswordToken};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::telegram::{NewProcess, ProcessStart, start_process};
use crate::ui::MainWindow;
use crate::ui::state::ProcessState;
use crate::ui::widgets::{Card, PrimaryButton};

#[derive(Default)]
pub struct SessionData {
    session_name: String,
    phone_number: String,
    tg_code: String,
    tg_password: String,
    is_temporary: bool,
    /// Taken while a sign in attempt is running and handed back on a wrong password
    password_token: Option<PasswordToken>,
    /// Whether the account asked for a password, regardless of the token being in flight
    password_mode: bool,
    tg_code_token: Option<Arc<Mutex<LoginToken>>>,
}

impl SessionData {
    fn get_session_name(&self) -> String {
        self.session_name.clone()
    }

    fn get_phone_number(&self) -> String {
        self.phone_number.replace('+', "")
    }

    fn get_password(&self) -> String {
        self.tg_password.clone()
    }

    fn get_is_temporary(&self) -> bool {
        self.is_temporary
    }

    pub fn set_login_token(&mut self, token: LoginToken) {
        self.tg_code_token = Some(Arc::new(Mutex::new(token)));
    }

    pub fn set_password_token(&mut self, token: PasswordToken) {
        self.password_token = Some(token);
        self.password_mode = true;
    }

    fn get_tg_code(&self) -> String {
        self.tg_code.clone()
    }

    fn get_tg_code_token(&self) -> Arc<Mutex<LoginToken>> {
        self.tg_code_token.clone().unwrap()
    }

    fn take_password_token(&mut self) -> Option<PasswordToken> {
        self.password_token.take()
    }

    pub fn reset_data(&mut self) {
        self.session_name = String::new();
        self.phone_number = String::new();
        self.tg_code = String::new();
        self.tg_password = String::new();
        self.is_temporary = false;
        self.password_token = None;
        self.password_mode = false;
        self.tg_code_token = None;
    }
}

impl MainWindow {
    pub fn show_session_ui(&mut self, ui: &mut Ui) {
        // Once a code was sent or a password is required, the account details are locked in
        let locked = self.session.tg_code_token.is_some() || self.session.password_mode;

        Card::new().title("ACCOUNT").show(ui, |ui| {
            Grid::new("Session Grid")
                .num_columns(2)
                .spacing([8.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Label::new("Session Name:"));
                    });

                    let text_edit = TextEdit::singleline(&mut self.session.session_name)
                        .hint_text("My TG Session Name");

                    if locked {
                        ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                    } else {
                        let hover_text = if self.session.is_temporary {
                            "A name for the temporary session"
                        } else {
                            "Save the session in a file with this name for later access"
                        };
                        ui.add_sized(ui.available_size(), text_edit)
                            .on_hover_text(hover_text);
                    }

                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Label::new("Phone Number:"));
                    });

                    let text_edit = TextEdit::singleline(&mut self.session.phone_number)
                        .hint_text("International format: +1234567890");

                    if locked {
                        ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                    } else {
                        ui.add_sized(ui.available_size(), text_edit).on_hover_text(
                            "The phone number of your Telegram in international format",
                        );
                    }

                    ui.end_row();

                    ui.label("");

                    let checkbox = Checkbox::new(
                        &mut self.session.is_temporary,
                        "Do not create a session file",
                    );

                    if self.session.tg_code_token.is_some() || self.is_processing {
                        ui.add_enabled(false, checkbox);
                    } else {
                        ui.add(checkbox).on_hover_text(
                            "Create a new temporary session?
If yes, it will try to log out before the app is closed and no session file will be created",
                        );
                    }

                    ui.end_row();
                });
        });

        ui.add_space(8.0);

        Card::new().title("VERIFICATION").show(ui, |ui| {
            Grid::new("Verification Grid")
                .num_columns(2)
                .spacing([8.0, 10.0])
                .show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(Label::new("Login Code:"));
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let button = Button::new("Send Code");
                        if self.session.phone_number.is_empty()
                            || self.session.password_mode
                            || self.session.session_name.is_empty()
                            || self.is_processing
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
                            TextEdit::singleline(&mut self.session.tg_code).hint_text("12345");

                        if self.session.tg_code_token.is_some() {
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

                    let text_edit = TextEdit::singleline(&mut self.session.tg_password)
                        .password(true)
                        .hint_text("Only needed if the account has two-step verification");

                    if self.session.password_mode {
                        ui.add_sized(ui.available_size(), text_edit)
                            .on_hover_text("Telegram login password");
                    } else {
                        ui.add_enabled(false, text_edit.min_size(ui.available_size()));
                    }

                    ui.end_row();
                });
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui
                .add(Button::new("Reset").min_size(vec2(80.0, 40.0)))
                .on_hover_text("Reset and clear new session creation data")
                .clicked()
            {
                self.session.reset_data();
                self.incomplete_tg_client = None;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let button = PrimaryButton::new("Log In").size(vec2(ui.available_width(), 40.0));
                if self.is_processing
                    || self.session.phone_number.is_empty()
                    || self.session.session_name.is_empty()
                    || self.session.tg_code.is_empty()
                    || (self.session.password_mode
                        && (self.session.password_token.is_none()
                            || self.session.tg_password.is_empty()))
                {
                    ui.add_enabled(false, button);
                } else if ui
                    .add(button)
                    .on_hover_text("Try to log in with the Telegram login info")
                    .clicked()
                {
                    if self.session.password_mode {
                        self.sign_in_password();
                    } else {
                        self.sign_in_code();
                    }
                }
            });
        });
    }

    /// Starts a thread to send a Telegram login code to the phone number
    fn request_login_code(&mut self, context: Context) {
        let phone_num = self.session.get_phone_number();
        let session_name = self.session.get_session_name();
        let is_temporary = self.session.get_is_temporary();

        let sender_clone = self.tg_sender.clone();

        self.is_processing = true;
        self.process_state = ProcessState::SendingTGCode;

        self.runtime.spawn(async move {
            start_process(
                NewProcess::SendLoginCode(session_name, phone_num, is_temporary),
                sender_clone,
                context,
            )
            .await;
        });
    }

    /// Starts a thread to try to log in with the given login code
    fn sign_in_code(&mut self) {
        self.is_processing = true;
        self.process_state = ProcessState::LogInWithCode;

        let code = self.session.get_tg_code();
        let token = self.session.get_tg_code_token();

        let client = self.incomplete_tg_client.clone().unwrap();
        self.runtime.spawn(async move {
            client
                .start_process(ProcessStart::SignInCode(token, code))
                .await;
        });
    }

    /// Starts a thread to try to log in with the given login password
    fn sign_in_password(&mut self) {
        self.is_processing = true;
        self.process_state = ProcessState::LogInWithPassword;

        let password = self.session.get_password();
        let Some(token) = self.session.take_password_token() else {
            return;
        };

        let client = self.incomplete_tg_client.clone().unwrap();
        self.runtime.spawn(async move {
            client
                .start_process(ProcessStart::SignInPasswords(Box::new(token), password))
                .await;
        });
    }
}
