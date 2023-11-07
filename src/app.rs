use crate::chaser;
use iced::futures::channel::mpsc;
use iced::keyboard::{Event as KeyBoardEvent, KeyCode};
use iced::widget::tooltip::Position;
use iced::widget::{button, checkbox, column, container, image, row, text, tooltip};
use iced::{alignment::*, Alignment, Application, Command, Element, Event, Length, Subscription};

struct Chaser {
    running: bool,
}

const DETACHED_PROCESS: u32 = 0x00000008;

use iced::widget::image::Handle;
#[derive(Debug, Clone)]
enum StatusIcon {
    Normal(&'static [u8]),
    Warning(&'static [u8]),
    Error(&'static [u8]),
}

use crate::ICON;

impl StatusIcon {
    fn normal() -> Self {
        Self::Normal(ICON)
    }

    fn warning() -> Self {
        Self::Warning(ICON)
    }

    fn error() -> Self {
        Self::Warning(ICON)
    }
}

pub struct App {
    chaser: Chaser,
    num_core_lic: i32,
    status_icon: StatusIcon,
    status_message: String,
    auto_launch_houdini: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartChaser,
    StopChaser,
    ChaserEvent(chaser::ChaserEvent),
    ExitApp,
    AutoLaunchHoudini(bool),
    HoudiniLaunched(bool),
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
                status_icon: StatusIcon::normal(),
                status_message: String::new(),
                auto_launch_houdini: true,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Houdini License Chaser".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::HoudiniLaunched(launched) => match launched {
                true => return iced::window::close(),
                false => {
                    self.status_message = String::from("Could not launch Houdini");
                    self.status_icon = StatusIcon::error();
                }
            },
            Message::ExitApp => return iced::window::close(),
            Message::StartChaser => {
                self.chaser.running = true;
            }
            Message::StopChaser => {
                self.chaser.running = false;
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

                    if self.auto_launch_houdini && self.num_core_lic > 0 {
                        let hfs = std::env::var("HFS").expect("TODO");
                        let hbin = std::path::Path::new(&hfs).join("bin").join("houdinicore");
                        self.chaser.running = false;
                        return Command::perform(
                            async move {
                                match tokio::process::Command::new(&hbin)
                                    .creation_flags(DETACHED_PROCESS)
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .stdin(std::process::Stdio::null())
                                    .spawn()
                                {
                                    Ok(_) => true,
                                    Err(e) => {
                                        eprintln!("Could not start Houdini!");
                                        false
                                    }
                                }
                            },
                            Message::HoudiniLaunched,
                        );
                    }
                }
                chaser::ChaserEvent::ServerErrored => {
                    self.status_message = String::from("Chaser Error");
                    self.status_icon = StatusIcon::error();
                }
            },
            Message::AutoLaunchHoudini(value) => {
                self.auto_launch_houdini = value;
            }
        }
        Command::none()
    }

    #[rustfmt::skip]
    fn view(&self) -> Element<'_, Self::Message> {
        let (button_label, message) = if self.chaser.running {
            ("Stop Chasing", Message::StopChaser)
        } else {("Start Chasing", Message::StartChaser)};

        let spacer = iced::widget::Space::with_height(80);
        let icon_file = match self.status_icon {
            StatusIcon::Normal(f) => f,
            StatusIcon::Warning(f) => f,
            StatusIcon::Error(f) => f,
        };
        let launch_houdini_chb = checkbox("Auto-Launch Houdini",
                                          self.auto_launch_houdini,
                                          Message::AutoLaunchHoudini).size(20).spacing(5);
        let handle = Handle::from_memory(icon_file);
        let content = column![
            row![button(text(button_label).horizontal_alignment(Horizontal::Center))
                .on_press(message)
                .width(Length::Fill)
                ].align_items(Alignment::Center).width(180),
            tooltip(image(handle).width(50), &self.status_message, Position::FollowCursor),
            launch_houdini_chb.width(180),
            row![button(text("Exit").horizontal_alignment(Horizontal::Center))
            .on_press(Message::ExitApp)
                .width(Length::Fill)
            ].align_items(Alignment::Center).width(180)
        ].spacing(10).align_items(Alignment::Center);

        container(content).width(Length::Fill).height(Length::Fill).center_x().center_y().into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            if self.chaser.running {
                chaser::subscribe()
            } else {
                Subscription::none()
            },
            iced::subscription::events_with(|event, status| match event {
                Event::Keyboard(KeyBoardEvent::KeyPressed {
                    key_code: KeyCode::Escape,
                    ..
                }) => Some(Message::ExitApp),
                _ => None,
            }),
        ])
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }
}

#[derive(Debug, Clone)]
enum SubState {
    Starting,
    Ready,
}
