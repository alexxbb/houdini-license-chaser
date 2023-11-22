use crate::app::Message;
use crate::icons::{ICON, ICON_WARN};
use iced::widget::{column, image as image_widget, image::Handle, row};
use iced::Element;
use image::imageops::colorops;

#[derive(Debug, Clone)]
pub enum IconState {
    Idle,
    Working,
    Error,
}

pub struct StatusImage {
    state: IconState,
    frame: u32,
    image_bytes: Vec<u8>,
}

impl StatusImage {
    pub fn new() -> Self {
        StatusImage {
            state: IconState::Idle,
            image_bytes: vec![],
            frame: 0,
        }
    }
    pub fn set_state(&mut self, state: IconState) {
        self.state = state;
    }

    pub fn set_frame(&mut self, frame: u32) {
        self.frame = frame;
    }
    pub fn view(&self) -> Element<Message> {
        let image_handle = match &self.state {
            IconState::Idle => Handle::from_memory(ICON),
            IconState::Working => {
                let mut img =
                    image::load_from_memory_with_format(ICON, image::ImageFormat::Png).unwrap();
                let colored = colorops::huerotate(&img, (self.frame * 2) as i32);
                Handle::from_pixels(img.width(), img.height(), colored.to_vec())
            }
            IconState::Error => Handle::from_memory(ICON_WARN),
        };
        image_widget(image_handle).width(70).into()
    }
}
