use crate::chaser;
use iced::futures::channel::mpsc;
use iced::widget::{button, column, row, text};
use iced::{
    alignment::*, Alignment, Application, Command, Element, Length, Settings, Subscription,
};

struct Chaser {
    running: bool,
}

pub struct App {
    chaser: Chaser,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartChaser,
    StopChaser,
    ChaserStarted(mpsc::Sender<()>),
    ChaserEvent { something: i32 },
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::theme::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                chaser: Chaser { running: false },
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Houdini License Chaser".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::StartChaser => {
                self.chaser.running = true;
            }
            Message::StopChaser => {
                self.chaser.running = false;
            }
            Message::ChaserStarted(receiver) => {
                dbg!("Chaser Started");
            }
            Message::ChaserEvent { something } => {
                dbg!(something);
            }
        }
        Command::none()
    }

    #[rustfmt::skip]
    fn view(&self) -> Element<'_, Self::Message> {
        let (button_label, message) = if self.chaser.running {
            ("Stop Chaser", Message::StopChaser)
        } else {("Start Chaser", Message::StartChaser)};

        column![
            row![
                button(text(button_label).horizontal_alignment(Horizontal::Center))
                    .on_press(message)
                    .width(Length::Fill)]
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        chaser::subscribe()
    }
}
