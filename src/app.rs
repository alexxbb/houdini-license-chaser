use crate::chaser;
use iced::futures::channel::mpsc;
use iced::widget::{button, column, row, text};
use iced::{alignment::*, Alignment, Application, Command, Element, Length, Subscription};

struct Chaser {
    running: bool,
}

pub struct App {
    chaser: Chaser,
    num_core_lic: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartChaser,
    StopChaser,
    ChaserStarted(mpsc::Sender<()>),
    ChaserEvent(chaser::ChaserEvent),
    ExitApp,
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
                num_core_lic: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Houdini License Chaser".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::ExitApp => return iced::window::close(),
            Message::StartChaser => {
                self.chaser.running = true;
            }
            Message::StopChaser => {
                self.chaser.running = false;
            }
            Message::ChaserStarted(sender) => {
                dbg!("Chaser Started");
            }
            Message::ChaserEvent(event) => match event {
                chaser::ChaserEvent::ServerStarted => {
                    eprintln!("Started");
                }
                chaser::ChaserEvent::ServerResponse(resp) => {
                    let licenses = &resp[&String::from("licenses")];
                    let available_core_lic = licenses
                        .iter()
                        .filter_map(|lic| match lic.product_id {
                            crate::response::Product::HoudiniCore if lic.version.major == 20 => {
                                Some(lic.available)
                            }
                            _ => None,
                        })
                        .sum::<i32>();
                    self.num_core_lic = available_core_lic;
                }
                chaser::ChaserEvent::ServerErrored => {
                    eprintln!("App received server error");
                }
            },
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
                    .padding(10)
                    .width(Length::Fill)],
            text(format!("{}", self.num_core_lic)),
            button(text("Exit").horizontal_alignment(Horizontal::Center)).width(Length::Fill).on_press(Message::ExitApp)
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.chaser.running {
            chaser::subscribe()
        } else {
            Subscription::none()
        }
    }
}

#[derive(Debug, Clone)]
enum SubState {
    Starting,
    Ready,
}
