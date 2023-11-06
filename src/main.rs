#![allow(unused)]

mod app;
mod chaser;
mod request;
mod response;

use anyhow::{Context, Result};
use iced::application::Application;
use iced::Settings;

fn main() -> Result<()> {
    use iced::{Sandbox, Settings};

    let mut settings = Settings::default();
    settings.window.min_size = Some((200, 200));
    settings.window.max_size = Some((200, 200));
    let app = app::App::run(settings)?;

    Ok(())
}
