#![allow(unused)]
#![windows_subsystem = "windows"]

mod app;
mod chaser;
mod request;
mod response;

use anyhow::{Context, Result};
use iced::application::Application;
use iced::Settings;

fn main() -> Result<()> {
    use iced::{Sandbox, Settings};
    dotenv::dotenv().ok();

    let mut settings = Settings::default();
    settings.window.icon = Some(iced::window::icon::from_file(format!(
        "{}/assets/eye.png",
        env!("CARGO_MANIFEST_DIR")
    ))?);
    settings.window.min_size = Some((200, 200));
    settings.window.max_size = Some((200, 200));
    settings.window.resizable = false;
    settings.window.decorations = false;
    let app = app::App::run(settings)?;

    Ok(())
}
