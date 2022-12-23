use iced::{widget::container, Background, Color};

pub struct DspContainer;

impl DspContainer {
    const COLOR: Color = Color::from_rgb(0.5, 0.5, 0.55);
}

impl container::StyleSheet for DspContainer {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color {
                a: 0.2,
                ..Self::COLOR
            })),
            border_radius: 4.0,
            border_color: Self::COLOR,
            ..container::Appearance::default()
        }
    }
}

impl From<DspContainer> for iced::theme::Container {
    fn from(style: DspContainer) -> Self {
        iced::theme::Container::Custom(Box::new(style))
    }
}
