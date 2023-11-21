use crate::chaser;
use crate::config::{AppCache, ConfigError, UserConfig};
use crate::pages::{ErrorPage, SettingsMessage, SettingsPage};
use crate::response::Product;
use anyhow::Result;
use iced::futures::channel::mpsc;
use iced::keyboard::{Event as KeyBoardEvent, KeyCode};
use iced::mouse::{Button, Event as MouseEvent};
use iced::widget::text::LineHeight;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, checkbox, column, container, image as image_widget, radio, row, text, tooltip,
};
use iced::{
    alignment::*, theme, Alignment, Application, Command, Element, Event, Font, Length, Size,
    Subscription,
};
use iced::{Color, Point};
use std::fmt::Write;
use std::sync::Arc;
use std::time::Duration;

use crate::widgets::{IconState, StatusImage};

pub(crate) const APP_NAME: &'static str = "houdini.license.chaser";

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LicenseType {
    Core,
    Fx,
    Other,
}

impl From<LicenseType> for String {
    fn from(value: LicenseType) -> Self {
        String::from(match value {
            LicenseType::Core => "Core",
            LicenseType::Fx => "Fx",
            LicenseType::Other => "Other",
        })
    }
}

impl From<&Product> for LicenseType {
    fn from(value: &Product) -> Self {
        match value {
            Product::HoudiniCore => LicenseType::Core,
            Product::HoudiniFx => LicenseType::Fx,
            _ => LicenseType::Other,
        }
    }
}

pub struct MainPage {
    frame: u32,
    chaser_subscribe: bool,
    chaser_running: bool,
    chaser_num_pings: u32,
    num_core_lic: Option<i32>,
    status_image: StatusImage,
    status_message: String,
    auto_launch_houdini: bool,
    chase_license: Option<LicenseType>,
}

impl MainPage {
    pub const SIZE: Size<u32> = Size::new(200, 280);
    fn new() -> Self {
        Self {
            chaser_subscribe: false,
            chaser_running: false,
            chaser_num_pings: 0,
            num_core_lic: None,
            frame: 1,
            status_image: StatusImage::new(),
            status_message: String::new(),
            auto_launch_houdini: true,
            chase_license: Some(LicenseType::Core),
        }
    }

    fn update(&mut self, message: Message, config: &UserConfig) -> Command<Message> {
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
            Message::LicenseSelected(license) => self.chase_license = Some(license),
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
                        .filter_map(|lic| {
                            let lic_type = LicenseType::from(&lic.product_id);
                            let selected_lic = self.chase_license.expect("always some");
                            (selected_lic == lic_type && lic.version.major == 20)
                                .then_some(lic.available)
                        })
                        .sum::<i32>();
                    self.num_core_lic = Some(available_core_lic);

                    if self.auto_launch_houdini && available_core_lic > 0 {
                        let hbin = config.houdini_executable.clone();
                        self.chaser_subscribe = false;
                        self.chaser_running = false;
                        self.status_image.set_state(IconState::Idle);
                        return Command::perform(
                            async move {
                                let mut command = tokio::process::Command::new(hbin);
                                command
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .stdin(std::process::Stdio::null());

                                #[cfg(windows)]
                                command.creation_flags(0x00000008);

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
        Command::none()
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
        let license_selectors = row![
            text("License:").size(18),
            radio(
                LicenseType::Core,
                LicenseType::Core,
                self.chase_license,
                Message::LicenseSelected
            )
            .size(15)
            .spacing(5),
            radio(
                LicenseType::Fx,
                LicenseType::Fx,
                self.chase_license,
                Message::LicenseSelected
            )
            .spacing(5)
            .size(15)
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        let num_license_text = {
            text(format!(
                "{} Licenses Count: {}",
                String::from(self.chase_license.unwrap()),
                self.num_core_lic
                    .map(|v| v.to_string())
                    .unwrap_or("---".to_string())
            ))
            .width(Length::Fill)
        };
        let mut start_btn = button(text(button_label).horizontal_alignment(Horizontal::Center))
            .on_press(message)
            .width(Length::Fill);

        if self.chaser_subscribe {
            start_btn = start_btn.style(theme::Button::Secondary);
        }

        let bottom_row = row![
            action(new_icon(), Message::SwitchPage(PageType::Settings)),
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
        .width(Length::Fill);
        let content = column![
            row![start_btn].align_items(Alignment::Center),
            self.status_image.view(),
            text(format!("Server Ping Count: {}", self.chaser_num_pings)).width(180),
            num_license_text,
            license_selectors.width(Length::Fill),
            launch_houdini_chb.width(Length::Fill),
            bottom_row,
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .into()
    }
    fn subscription(&self, config: &UserConfig) -> Subscription<Message> {
        Subscription::batch(vec![if self.chaser_subscribe {
            let server_url: Arc<str> = Arc::from(config.server_url.as_str());
            Subscription::batch(vec![
                iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick),
                chaser::subscribe(server_url),
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
    LicenseSelected(LicenseType),
    SwitchPage(PageType),
    Settings(SettingsMessage),
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
        let mut error_message = String::new();
        let config = match config {
            Ok(config) => {
                if !config.is_valid() {
                    current_page = PageType::Settings;
                    error_message.write_str("Error: Check config values!");
                    commands.push(iced::window::resize(SettingsPage::SIZE));
                }
                config
            }
            Err(config_error) => {
                match config_error {
                    ConfigError::Missing => {
                        current_page = PageType::Settings;
                        commands.push(iced::window::resize(SettingsPage::SIZE));
                    }
                    e => {
                        current_page = PageType::Error;
                        commands.push(iced::window::resize(ErrorPage::SIZE));
                        error_page.title = "Error Loading Config File".to_owned();
                        error_page.footer = "Tip: delete the config file and try again".to_owned();
                        error_page.body = e.to_string();
                    }
                }
                UserConfig::default()
            }
        };
        let mut settings_page = SettingsPage::new(config);
        settings_page.error = error_message;

        let app = App {
            main_page: MainPage::new(),
            settings_page,
            error_page,
            current: current_page,
            cache,
        };
        (app, Command::batch(commands))
    }

    fn title(&self) -> String {
        "Houdini License Chaser".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match &message {
            Message::MouseMoved(_point) => Command::none(),
            Message::WindowMoved(x, y) => {
                self.cache.window_position = [*x, *y];
                Command::none()
            }
            Message::ExitApp => {
                let _ = self.cache.save();
                return iced::window::close();
            }
            Message::Settings(settings_message) => match settings_message {
                SettingsMessage::OkPressed => {
                    if self.settings_page.check_input() {
                        if let Err(e) = self.settings_page.config.save() {
                            // TODO
                            eprintln!("Could not save config");
                        }
                        self.current = PageType::Main;
                        iced::window::resize(MainPage::SIZE)
                    } else {
                        Command::none()
                    }
                }
                _ => self.settings_page.update(message.clone()),
            },
            Message::SwitchPage(page) => {
                self.current = page.clone();
                let window_size = match page {
                    PageType::Main => MainPage::SIZE,
                    PageType::Settings => {
                        self.settings_page.error.clear();
                        SettingsPage::SIZE
                    }
                    PageType::Error => ErrorPage::SIZE,
                };
                iced::window::resize(window_size)
            }
            other => match &mut self.current {
                PageType::Main => self
                    .main_page
                    .update(other.clone(), &self.settings_page.config),
                PageType::Settings => self.settings_page.update(other.clone()),
                PageType::Error => self.error_page.update(other.clone()),
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
                PageType::Main => self.main_page.subscription(&self.settings_page.config),
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
    on_press: Message,
) -> Element<'a, Message> {
    let action = button(
        container(content)
            .width(25)
            .height(Length::Fill)
            .padding(0)
            .center_x()
            .center_y(),
    );
    action
        .style(theme::Button::Secondary)
        .on_press(on_press)
        .into()
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{e994}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("icomoon");

    text(codepoint).font(ICON_FONT).size(16).into()
}
