use crate::chaser;
use crate::config::{AppCache, UserConfig};
use crate::settings::SettingsPage;
use iced::futures::channel::mpsc;
use iced::keyboard::{Event as KeyBoardEvent, KeyCode};
use iced::mouse::{Button, Event as MouseEvent};
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, checkbox, column, container, image as image_widget, row, text, tooltip,
};
use iced::{
    alignment::*, theme, Alignment, Application, Command, Element, Event, Font, Length,
    Subscription,
};
use iced::{Color, Point};
use std::time::Duration;

use crate::widgets::{IconState, StatusImage};

const DETACHED_PROCESS: u32 = 0x00000008;

use iced::widget::image::Handle;
use image::{GenericImage, Rgba};

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

enum Pages {
    Main,
    Settings(SettingsPage),
}

pub struct App {
    frame: u32,
    chaser_subscribe: bool,
    chaser_running: bool,
    chaser_num_pings: u32,
    num_core_lic: Option<i32>,
    status_image: StatusImage,
    status_message: String,
    auto_launch_houdini: bool,
    configs: (AppCache, UserConfig),
    pages: Pages,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartChaser,
    StopChaser,
    ChaserEvent(chaser::ChaserEvent),
    ExitApp,
    Tick,
    MouseMoved(Point),
    WindowMoved(i32, i32),
    AutoLaunchHoudini(bool),
    HoudiniLaunched(bool),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::theme::Theme;
    type Flags = (AppCache, UserConfig);

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                chaser_subscribe: false,
                chaser_running: false,
                chaser_num_pings: 0,
                num_core_lic: None,
                frame: 1,
                status_image: StatusImage::new(),
                status_message: String::new(),
                auto_launch_houdini: true,
                configs: flags,
                pages: Pages::Main,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Houdini License Chaser".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Tick => {
                self.frame = self.frame.wrapping_add(1);
                self.status_image.set_frame(self.frame);
            }
            Message::MouseMoved(_point) => {
                // return iced::window::drag()
            }
            Message::WindowMoved(x, y) => {
                self.configs.0.window_position = [x, y];
            }
            Message::HoudiniLaunched(launched) => match launched {
                true => {
                    self.chaser_subscribe = false;
                }
                false => {
                    self.status_message = String::from("Could not launch Houdini");
                    self.status_image.set_state(IconState::Error)
                }
            },
            Message::ExitApp => {
                let _ = self.configs.0.save();
                return iced::window::close();
            }
            Message::StartChaser => {
                self.chaser_subscribe = true;
                self.status_image.set_state(IconState::Working)
            }
            Message::StopChaser => {
                self.chaser_subscribe = false;
                self.status_image.set_state(IconState::Idle)
            }
            Message::ChaserEvent(event) => match event {
                chaser::ChaserEvent::ServerStarted => {
                    self.chaser_running = true;
                    self.status_image.set_state(IconState::Working)
                }
                chaser::ChaserEvent::ServerResponse(resp) => {
                    self.chaser_num_pings += 1;
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
                    self.num_core_lic = Some(available_core_lic);

                    if self.auto_launch_houdini && available_core_lic > 0 {
                        let hfs = std::env::var("HFS").expect("TODO");
                        let hbin = std::path::Path::new(&hfs).join("bin").join("houdinicore");
                        self.chaser_subscribe = false;
                        self.chaser_running = false;
                        self.status_image.set_state(IconState::Idle);
                        return Command::perform(
                            async move {
                                let mut command = tokio::process::Command::new(&hbin);
                                command
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .stdin(std::process::Stdio::null());

                                #[cfg(windows)]
                                command.creation_flags(DETACHED_PROCESS);

                                match command.spawn() {
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
                    self.status_image.set_state(IconState::Error);
                    self.chaser_subscribe = false;
                    self.status_image.set_state(IconState::Error);
                }
            },
            Message::AutoLaunchHoudini(value) => {
                self.auto_launch_houdini = value;
            }
        }
        Command::none()
    }

    // #[rustfmt::skip]
    fn view(&self) -> Element<'_, Self::Message> {
        let (button_label, message) = if self.chaser_subscribe {
            ("Stop Chasing", Message::StopChaser)
        } else {
            ("Start Chasing", Message::StartChaser)
        };

        let spacer = iced::widget::Space::with_height(80);
        let launch_houdini_chb = checkbox(
            "Auto-Launch Houdini",
            self.auto_launch_houdini,
            Message::AutoLaunchHoudini,
        )
        .size(20)
        .spacing(5);
        let num_license_text = {
            text(format!(
                "Core Licenses Count: {}",
                self.num_core_lic
                    .map(|v| v.to_string())
                    .unwrap_or("---".to_string())
            ))
            .width(180)
        };
        let mut start_btn = button(text(button_label).horizontal_alignment(Horizontal::Center))
            .on_press(message)
            .width(180);

        if self.chaser_subscribe {
            start_btn = start_btn.style(iced::theme::Button::Secondary);
        }

        let bottom_row = row![
            action(new_icon(), "Text", None),
            button(
                text("Exit")
                    .horizontal_alignment(Horizontal::Center)
                    .line_height(LineHeight::Relative(1.2))
            )
            .on_press(Message::ExitApp)
            .width(Length::Fill)
        ]
        .align_items(Alignment::Center)
        .width(180);
        let content = column![
            start_btn,
            self.status_image.view(),
            text(format!("Server Ping Count: {}", self.chaser_num_pings)).width(180),
            num_license_text,
            launch_houdini_chb.width(180),
            bottom_row,
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            if self.chaser_subscribe {
                Subscription::batch(vec![
                    iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick),
                    chaser::subscribe(),
                ])
            } else {
                Subscription::none()
            },
            iced::subscription::events_with(|event, status| match event {
                Event::Keyboard(KeyBoardEvent::KeyPressed {
                    key_code: KeyCode::Escape,
                    ..
                }) => Some(Message::ExitApp),
                Event::Window(e) => match e {
                    iced::window::Event::Moved { x, y } => Some(Message::WindowMoved(x, y)),
                    _ => None,
                },
                e => None,
            }),
        ])
    }
}

#[derive(Debug, Clone)]
enum SubState {
    Starting,
    Ready,
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).width(30).center_x());

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box)
        .into()
    } else {
        action.style(theme::Button::Secondary).into()
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    // U+1F6E0
    icon('\u{1F6E0}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}
