#![allow(unused)]
#![windows_subsystem = "windows"]

mod app;
mod chaser;
mod config;
mod request;
mod response;

const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/eye2.png"));
// const ICON_WARN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/warn.png"));

use crate::config::{AppCache, UserConfig};
use anyhow::{Context, Result};
use iced::application::Application;
use iced::window::Position;
use iced::Settings;

fn main() -> Result<()> {
    use iced::{Sandbox, Settings};
    let app_cache = AppCache::load()?;
    let user_config = UserConfig::load()?;
    dotenv::dotenv().ok();

    let mut settings = Settings::<(AppCache, UserConfig)>::default();
    settings.window.icon = Some(iced::window::icon::from_file_data(
        ICON,
        Some(image::ImageFormat::Png),
    )?);
    settings.window.level = iced::window::Level::AlwaysOnTop;
    settings.window.min_size = Some((200, 250));
    settings.window.max_size = Some((200, 250));
    settings.window.resizable = false;
    let [x, y] = app_cache.window_position;
    settings.window.position = Position::Specific(x, y);
    // settings.window.decorations = false;
    settings.flags = (app_cache, user_config);
    let app = app::App::run(settings)?;

    Ok(())
}
