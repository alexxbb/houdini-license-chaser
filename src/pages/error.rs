use crate::app::Message;
use iced::alignment::{Alignment, Horizontal, Vertical};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Command, Element, Length, Size, Subscription};

#[derive(Debug, Clone)]
pub struct ErrorPage {
    pub title: String,
    pub body: String,
}

impl ErrorPage {
    pub const SIZE: Size<u32> = Size::new(550, 200);
    pub fn new() -> Self {
        ErrorPage {
            title: "Error".to_owned(),
            body: "------".to_owned(),
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
        let btn = button("Ouch, bummer!").on_press(Message::ExitApp);
        let content = column![
            row![
                text(&self.title).width(Length::Fill).horizontal_alignment(Horizontal::Center).size(20)
            ].align_items(Alignment::Center).width(Length::Fill),

            Space::with_height(Length::Fill),

            row![
                text(&self.body).width(Length::Fill)
            ].align_items(Alignment::Center).spacing(10).width(Length::Fill),

            Space::with_height(Length::Fill),
            row![Space::with_width(Length::Fill), btn].align_items(Alignment::Center)
        ].align_items(Alignment::Center);
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(10)
            .into()
    }
}
