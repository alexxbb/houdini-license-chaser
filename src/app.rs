use crate::chaser;
use crate::config::{AppCache, UserConfig};
use iced::futures::channel::mpsc;
use iced::keyboard::{Event as KeyBoardEvent, KeyCode};
use iced::mouse::{Button, Event as MouseEvent};
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, checkbox, column, container, image as image_widget, row, text, tooltip,
};
use iced::Point;
use iced::{alignment::*, Alignment, Application, Command, Element, Event, Length, Subscription};
use std::time::Duration;

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

pub struct App {
    frame: u32,
    chaser_subscribe: bool,
    chaser_running: bool,
    chaser_num_pings: u32,
    num_core_lic: Option<i32>,
    status_icon: StatusIcon,
    status_message: String,
    auto_launch_houdini: bool,
    configs: (AppCache, UserConfig),
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
                status_icon: StatusIcon::normal(),
                status_message: String::new(),
                auto_launch_houdini: true,
                configs: flags,
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
                    self.status_icon = StatusIcon::error();
                }
            },
            Message::ExitApp => {
                let _ = self.configs.0.save();
                return iced::window::close();
            }
            Message::StartChaser => {
                self.chaser_subscribe = true;
            }
            Message::StopChaser => {
                self.chaser_subscribe = false;
            }
            Message::ChaserEvent(event) => match event {
                chaser::ChaserEvent::ServerStarted => {
                    self.chaser_running = true;
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
        let (button_label, message) = if self.chaser_subscribe {
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
        let num_license_text = {
            text(format!("Core Licenses Count: {}", self.num_core_lic.map(|v| v.to_string()).unwrap_or("---".to_string()))).width(180)
        };
        let icon_widget = {
            let mut img = image::load_from_memory_with_format(ICON, image::ImageFormat::Png).unwrap();
            let width = img.width();
            let height = img.height();
            let img_bytes = if self.chaser_subscribe {
                image::imageops::colorops::huerotate(&img, (self.frame * 2) as i32).to_vec()
            } else {
                for (x, y, pixel) in img.to_luma_alpha8().enumerate_pixels() {
                    let luma = pixel[0];
                    let alpha = pixel[1];
                    img.put_pixel(x, y, Rgba([luma, luma, luma, alpha]));
                }
                img.into_bytes()
            };
            image_widget(Handle::from_pixels(width, height, img_bytes)).width(70)
        };
        let content = column![
            row![button(text(button_label).horizontal_alignment(Horizontal::Center))
                .on_press(message)
                .width(Length::Fill)
                ].align_items(Alignment::Center).width(180),
            icon_widget,
            text(format!("Server Ping Count: {}", self.chaser_num_pings)).width(180),
            num_license_text,
            launch_houdini_chb.width(180),
            row![button(text("Exit").horizontal_alignment(Horizontal::Center).line_height(LineHeight::Relative(1.2)))
            .on_press(Message::ExitApp)
                .width(Length::Fill)
            ].align_items(Alignment::Center).width(180)
        ].spacing(10).align_items(Alignment::Center);

        container(content).width(Length::Fill).height(Length::Fill).center_x().center_y()
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
