#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod chaser;
mod config;
mod icons;
mod pages;
mod response;
mod widgets;

use anyhow::{Context, Result};
use iced::{Application, Settings};

const WIN_SIZE: Option<(u32, u32)> = Some((app::MainPage::SIZE.width, app::MainPage::SIZE.height));

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let mut settings = Settings::<()>::default();
    settings.window.icon = Some(iced::window::icon::from_file_data(
        icons::ICON,
        Some(image::ImageFormat::Png),
    )?);
    settings.fonts = vec![icons::ICONS_TTF.into()];
    settings.default_font = iced::Font::DEFAULT;
    settings.window.level = iced::window::Level::AlwaysOnTop;
    settings.window.min_size = WIN_SIZE;
    settings.window.max_size = WIN_SIZE;
    settings.window.resizable = false;
    settings.window.decorations = true;
    let app = app::App::run(settings)?;

    Ok(())
}
