use eframe::egui::{Context, FontData, FontDefinitions};
use eframe::epaint::FontFamily;
use log::{error, info};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Downloads the app font data from github, saves them locally and loads them
pub async fn download_font(ctx: Context) {
    let cjk_font_url =
        "https://github.com/TheRustyPickle/Talon/raw/main/fonts/NotoSansCJK-Regular.ttc";
    let gentium_font_url =
        "https://github.com/TheRustyPickle/Talon/raw/main/fonts/GentiumBookPlus-Regular.ttf";

    let mut gentium_font = PathBuf::from(".");
    let mut cjk_font = PathBuf::from(".");

    gentium_font.push("fonts");
    cjk_font.push("fonts");

    gentium_font.push("GentiumBookPlus-Regular.ttf");
    cjk_font.push("NotoSansCJK-Regular.ttc");

    let client = reqwest::Client::new();

    let Ok(response) = client.get(cjk_font_url).send().await else {
        error!("Failed to get a response for CJK font");
        return;
    };

    let cjk_data = if response.status().is_success() {
        let mut file = File::create(cjk_font).unwrap();
        let response_data = response
            .bytes()
            .await
            .expect("Could not convert CJK response into bytes");
        file.write_all(&response_data).unwrap();
        info!("Saved CJK font data successfully");
        response_data
    } else {
        error!("CJK response was not a success");
        return;
    };

    let Ok(response) = client.get(gentium_font_url).send().await else {
        error!("Failed to get a response for Gentium font");
        return;
    };

    let gentium_data = if response.status().is_success() {
        let mut file = File::create(gentium_font).unwrap();
        let response_data = response
            .bytes()
            .await
            .expect("Could not convert gentium response into bytes");
        file.write_all(&response_data).unwrap();
        info!("Saved Gentium font data successfully");
        response_data
    } else {
        error!("Gentium response was not a success");
        return;
    };

    let font_cjk = FontData::from_owned(cjk_data.to_vec());
    let font_gentium = FontData::from_owned(gentium_data.to_vec());
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
}
