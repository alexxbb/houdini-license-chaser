#![allow(unused)]
// #![windows_subsystem = "windows"]

mod app;
mod chaser;
mod request;
mod response;

const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/eye.png"));
const ICON_WARN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/warn.png"));

use anyhow::{Context, Result};
use iced::application::Application;
use iced::Settings;

fn main() -> Result<()> {
    use iced::{Sandbox, Settings};
    dotenv::dotenv().ok();

    let mut settings = Settings::default();
    settings.window.icon = Some(iced::window::icon::from_file_data(
        ICON,
        Some(image::ImageFormat::Png),
    )?);
    settings.window.min_size = Some((200, 220));
    settings.window.max_size = Some((200, 220));
    settings.window.resizable = false;
    settings.window.decorations = false;
    let app = app::App::run(settings)?;

    Ok(())
}
