use crate::app::Message;
use crate::config::{ConfigError, UserConfig};
use crate::icons::icon;
use iced::alignment::*;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Color, Command, Element, Length, Size, Subscription};

#[derive(Debug, Clone)]
pub struct SettingsPage {
    pub config: UserConfig,
    pub error: String,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Save,
    ExecutableChanged(String),
    LicServerChanged(String),
    BrowseHoudiniExec,
}

impl SettingsPage {
    pub const SIZE: Size<u32> = Size::new(550, 250);

    pub fn new(config: UserConfig) -> Self {
        SettingsPage {
            config,
            error: "".to_owned(),
        }
    }

    pub fn check_input(&mut self) -> bool {
        if !self.config.houdini_executable.exists() {
            self.error = "Error: Invalid path to Houdini executable".to_owned();
            return false;
        }
        if self.config.server_url.is_empty() {
            self.error = "Error: Empty license server URL".to_owned();
            return false;
        }
        true
    }

    #[rustfmt::skip]
    pub fn view(&self) -> Element<'_, Message> {
        let save_button = button(text("Save").horizontal_alignment(Horizontal::Center))
            .width(100)
            .on_press(Message::Settings(SettingsMessage::Save));
        let config_path = UserConfig::config_file();
        let config_path = config_path.as_deref().map(|p|p.to_string_lossy())
            .expect("UserConfig path is valid");
        let mut font = iced::Font::default();
        font.weight = iced::font::Weight::Bold;
        font.style = iced::font::Style::Italic;
        let content = column![


            row![text("Settings")
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center)
                .size(20)]
            .align_items(Alignment::Center)
            .width(Length::Fill),

            Space::with_height(10),

            text(config_path).size(14).width(Length::Fill).horizontal_alignment(Horizontal::Center).font(font),

            Space::with_height(10),

            Space::with_height(Length::Fill),

            row![
                text("Houdini Executable:"),
                text_input("Path to Houdini executable", &self.config.houdini_executable.to_string_lossy())
                    .width(Length::Fill)
                    .on_input(|input|Message::Settings(SettingsMessage::ExecutableChanged(input))),
                button(icon('\u{e900}')).on_press(Message::Settings(SettingsMessage::BrowseHoudiniExec)),
            ]
                .align_items(Alignment::Center)
                .spacing(10)
                .width(Length::Fill),

            Space::with_height(5),

            row![
                text("License Server URL:"),
                text_input("License server URL", &self.config.server_url)
                    .width(Length::Fill)
                    .on_input(|input|Message::Settings(SettingsMessage::LicServerChanged(input)))
            ]
                .align_items(Alignment::Center)
                .spacing(10)
                .width(Length::Fill),


            text(&self.error).style(Color::from_rgb(0.9, 0.1, 0.0)),

            Space::with_height(Length::Fill),

            row![Space::with_width(Length::Fill), save_button].align_items(Alignment::Center)
        ]
        .align_items(Alignment::Center);
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(10)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Settings(msg) => match msg {
                SettingsMessage::ExecutableChanged(value) => {
                    self.config.houdini_executable = std::path::PathBuf::from(value);
                }
                SettingsMessage::LicServerChanged(value) => {
                    self.config.server_url = value;
                }
                SettingsMessage::BrowseHoudiniExec => {
                    if let Some(picked) = rfd::FileDialog::new()
                        .set_title("Select Houdini executable")
                        .pick_file()
                    {
                        self.config.houdini_executable = picked;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Command::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
