use crate::app::Message;
use iced::alignment::*;
use iced::widget::{button, column, container};
use iced::{Command, Element, Length, Size, Subscription};

#[derive(Debug, Clone)]
pub struct SettingsPage {
    pub size: Size<u32>,
}

impl SettingsPage {
    pub fn new() -> Self {
        SettingsPage {
            size: Size::new(450, 300),
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let btn = button("Close").on_press(Message::SwitchPage(crate::app::PageType::Main));
        let content = column![btn];
        container(content)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        Command::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
