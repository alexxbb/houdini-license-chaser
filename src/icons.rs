use iced::widget::text;
use iced::{Element, Font};

pub const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/eye.png"));
pub const ICON_WARN: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/warn.png"));
pub const ICONS_TTF: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons.ttf"));

pub fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("icomoon");

    text(codepoint).font(ICON_FONT).size(16).into()
}
