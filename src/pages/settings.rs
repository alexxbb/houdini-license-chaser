use crate::app::Message;
use crate::config::{ConfigError, UserConfig};
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
    OkPressed,
    HFSChanged(String),
}

impl SettingsPage {
    pub const SIZE: Size<u32> = Size::new(450, 300);

    pub fn new(config: UserConfig) -> Self {
        SettingsPage {
            config,
            error: "".to_owned(),
        }
    }

    pub fn check_input(&mut self) -> bool {
        if !self.config.hfs.exists() {
            self.error = "Error: Incorrect HFS path".to_owned();
            return false;
        }
        true
    }

    #[rustfmt::skip]
    pub fn view(&self) -> Element<'_, Message> {
        let save_button = button("Save").on_press(Message::Settings(SettingsMessage::OkPressed));
        let content = column![


            row![text("Settings")
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center)
                .size(20)]
            .align_items(Alignment::Center)
            .width(Length::Fill),

            Space::with_height(10),

            text("TODO: Display config path").width(Length::Fill).horizontal_alignment(Horizontal::Center),

            Space::with_height(Length::Fill),

            row![
                text("HFS:"),
                text_input("Path to Houdini install", &self.config.hfs.to_string_lossy())
                    .width(Length::Fill)
                    .on_input(|input|Message::Settings(SettingsMessage::HFSChanged(input)))
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
                SettingsMessage::HFSChanged(value) => {
                    self.config.hfs = std::path::PathBuf::from(value);
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
