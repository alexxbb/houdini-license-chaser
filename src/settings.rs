use crate::app::Message;
use iced::widget::{button, column, container};
use iced::{Element, Length};

#[derive(Debug)]
pub struct SettingsPage {
    width: i32,
    height: i32,
}

impl SettingsPage {
    pub fn new() -> Self {
        SettingsPage {
            width: 200,
            height: 100,
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let btn = button("Check");
        let content = column![btn];
        container(content).width(Length::Fill).into()
    }
}
