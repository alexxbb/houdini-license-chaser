use crate::chaser;
use crate::config::{AppCache, UserConfig};
use crate::pages::{ErrorPage, SettingsPage};
use anyhow::Result;
use iced::futures::channel::mpsc;
use iced::keyboard::{Event as KeyBoardEvent, KeyCode};
use iced::mouse::{Button, Event as MouseEvent};
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, checkbox, column, container, image as image_widget, row, text, tooltip,
};
use iced::{
    alignment::*, theme, Alignment, Application, Command, Element, Event, Font, Length, Size,
    Subscription,
};
use iced::{Color, Point};
use std::time::Duration;

use crate::widgets::{IconState, StatusImage};

const DETACHED_PROCESS: u32 = 0x00000008;

use iced::widget::image::Handle;
use image::{GenericImage, Rgba};

use crate::ICON;
#[derive(Debug, Clone)]
enum StatusIcon {
    Normal(&'static [u8]),
    Warning(&'static [u8]),
    Error(&'static [u8]),
}

struct PageIndex;
#[allow(non_upper_case_globals)]
impl PageIndex {
    const Main: usize = 0;
    const Settings: usize = 1;
}

pub struct App {
    main_page: MainPage,
    settings_page: SettingsPage,
    error_page: ErrorPage,
    current: PageType,
    config: UserConfig,
    cache: AppCache,
}

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

#[derive(Debug, Clone, Copy)]
pub enum PageType {
    Main,
    Settings,
    Error,
}

enum Page {
    Main(MainPage),
    Settings(SettingsPage),
}

pub struct MainPage {
    frame: u32,
    size: Size<u32>,
    chaser_subscribe: bool,
    chaser_running: bool,
    chaser_num_pings: u32,
    num_core_lic: Option<i32>,
    status_image: StatusImage,
    status_message: String,
    auto_launch_houdini: bool,
}

impl MainPage {
    fn new() -> Self {
        Self {
            size: Size::new(200u32, 250u32),
            chaser_subscribe: false,
            chaser_running: false,
            chaser_num_pings: 0,
            num_core_lic: None,
            frame: 1,
            status_image: StatusImage::new(),
            status_message: String::new(),
            auto_launch_houdini: true,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                self.frame = self.frame.wrapping_add(1);
                self.status_image.set_frame(self.frame);
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
            _ => {}
        }
        todo!()
    }

    fn view(&self) -> Element<'_, Message> {
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
            action(
                new_icon(),
                "Text",
                Some(Message::SwitchPage(PageType::Settings))
            ),
            button(
                text("Exit")
                    .horizontal_alignment(Horizontal::Center)
                    .line_height(LineHeight::Relative(1.2))
            )
            .on_press(Message::ExitApp)
            .width(Length::Fill)
        ]
        .align_items(Alignment::Center)
        .spacing(10)
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
    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![if self.chaser_subscribe {
            Subscription::batch(vec![
                iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick),
                chaser::subscribe(),
            ])
        } else {
            Subscription::none()
        }])
    }
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
    SwitchPage(PageType),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::theme::Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let mut commands = vec![];
        let cache = AppCache::load().unwrap_or_default();
        let [x, y] = cache.window_position;
        commands.push(iced::window::move_to(x, y));
        let config = UserConfig::load();
        let mut current_page = PageType::Main;
        let mut error_page = ErrorPage::new();
        if let Err(e) = &config {
            current_page = PageType::Error;
            error_page.error_message = e.to_string();
            commands.push(iced::window::resize(error_page.size));
        }

        let app = App {
            main_page: MainPage::new(),
            settings_page: SettingsPage::new(),
            error_page,
            current: current_page,
            config: config.unwrap_or_default(),
            cache,
        };
        (app, Command::batch(commands))
    }

    fn title(&self) -> String {
        "Houdini License Chaser".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::MouseMoved(_point) => Command::none(),
            Message::WindowMoved(x, y) => {
                self.cache.window_position = [x, y];
                Command::none()
            }
            Message::ExitApp => {
                let _ = self.cache.save();
                return iced::window::close();
            }
            Message::SwitchPage(page) => {
                self.current = page;
                let window_size = match page {
                    PageType::Main => self.main_page.size,
                    PageType::Settings => self.settings_page.size,
                    PageType::Error => self.error_page.size,
                };
                iced::window::resize(window_size)
            }
            other => match &mut self.current {
                PageType::Main => self.main_page.update(other),
                PageType::Settings => self.settings_page.update(other),
                PageType::Error => self.error_page.update(other),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.current {
            PageType::Main => self.main_page.view(),
            PageType::Settings => self.settings_page.view(),
            PageType::Error => self.error_page.view(),
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            match &self.current {
                PageType::Main => self.main_page.subscription(),
                PageType::Settings => self.settings_page.subscription(),
                PageType::Error => self.error_page.subscription(),
            },
            iced::event::listen_with(|event, status| match event {
                Event::Keyboard(KeyBoardEvent::KeyPressed {
                    key_code: KeyCode::Escape,
                    ..
                }) => Some(Message::ExitApp),
                Event::Window(e) => match e {
                    iced::window::Event::Moved { x, y } => Some(Message::WindowMoved(x, y)),
                    iced::window::Event::CloseRequested => Some(Message::ExitApp),
                    _ => None,
                },
                e => None,
            }),
        ])
    }
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
    icon('\u{e994}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("icomoon");

    text(codepoint).font(ICON_FONT).size(18).into()
}
