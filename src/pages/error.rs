use crate::app::Message;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::widget::{button, column, container, row, text};
use iced::{Command, Element, Length, Size, Subscription};

#[derive(Debug, Clone)]
pub struct ErrorPage {
    pub size: Size<u32>,
    pub error_message: String,
}

impl ErrorPage {
    pub fn new() -> Self {
        ErrorPage {
            size: Size::new(450, 250),
            error_message: "Error Page".to_owned(),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        Command::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    #[rustfmt::skip]
    pub fn view(&self) -> Element<'_, Message> {
        let btn = button("Close").on_press(Message::ExitApp);
        let content = column![
            row![
                text(&self.error_message).width(Length::Fill)
            ].align_items(Alignment::Center).spacing(10),
            row![btn].align_items(Alignment::End)
        ];
        container(content)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
