use iced::{
    Background, Border, Color, Theme,
    widget::{button, container::rounded_box},
};

pub fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        // background: Some(Color::TRANSPARENT.into()),
        background: None,
        text_color: Color::WHITE,
        ..Default::default()
    }
}
