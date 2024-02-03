use log::{error, info};
use reqwest::Error;
use semver::Version;
use serde::Deserialize;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Deserialize)]
struct GithubRelease {
    name: String,
    body: String,
}

pub fn check_version(version_body: Arc<Mutex<Option<String>>>) {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let user_agent = "Talon";

    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(2))
        .build();

    let Ok(client) = client else {
        error!("Failed to create reqwest client");
        return;
    };

    let response_result = client
        .get("https://api.github.com/repos/TheRustyPickle/Talon/releases/latest")
        .send();

    let Ok(response) = response_result else {
        error!("Failed to get a response from github API");
        return;
    };

    let caller: Result<GithubRelease, Error> = response.json();

    let Ok(release_data) = caller else {
        error!("Failed to deserialize release data");
        return;
    };

    let github_version = Version::parse(&release_data.name.replace('v', "")).unwrap();

    if github_version > current_version {
        info!("New version {} is available.", release_data.name);
        let mut initial_text = "A new version is available. Release notes:\n".to_string();
        initial_text += &parse_github_body(&release_data.body);
        let mut locked_body = version_body.lock().unwrap();
        *locked_body = Some(initial_text);
    } else {
        info!("The app version is the latest version");
    }
}

fn parse_github_body(body: &str) -> String {
    let body = body.replace("## Updates", "");
    let body = body.replace('*', "•");
    let body = body.replace('\r', "");
    let end_point = body.find("## Changes").unwrap();
    format!("\n{}\n", &body[..end_point].trim())
}
