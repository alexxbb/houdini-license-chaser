use crate::app::Message;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Size};

#[derive(Debug, Clone)]
pub struct ErrorPage {
    pub size: Size<u32>,
    pub error_message: String,
}

impl ErrorPage {
    pub fn new() -> Self {
        ErrorPage {
            size: Size::new(300, 450),
            error_message: "Error Page".to_owned(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let btn = button("Close").on_press(Message::SwitchPage(crate::app::PageType::Main));
        let content = column![text(&self.error_message).width(Length::Fill), btn];
        container(content)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
