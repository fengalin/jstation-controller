use iced::{
    widget::{button, checkbox, container, radio, toggler},
    Background, Color,
};

pub enum Button {
    Default,
    ModalClose,
    ListItem,
    ListItemSelected,
}

impl button::StyleSheet for Button {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let appearance = style.active(&iced::theme::Button::Primary);

        use Button::*;
        match self {
            Default => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Color::WHITE,
                ..appearance
            },
            ListItem => button::Appearance {
                background: None,
                text_color: Color::from_rgb(0.5, 0.5, 0.55),
                ..appearance
            },
            ListItemSelected => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.55, 0.0, 0.0))),
                text_color: Color::from_rgb(0.5, 0.5, 0.55),
                ..appearance
            },
            ModalClose => button::Appearance {
                background: None,
                text_color: Color::from_rgb(0.5, 0.5, 0.55),
                ..appearance
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let appearance = style.hovered(&iced::theme::Button::Primary);

        use Button::*;
        match self {
            Default => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
                text_color: Color::WHITE,
                ..appearance
            },
            ListItem => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Color::from_rgb(0.5, 0.5, 0.55),
                ..appearance
            },
            ListItemSelected => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.55, 0.0, 0.0))),
                text_color: Color::from_rgb(0.5, 0.5, 0.55),
                ..appearance
            },
            ModalClose => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Color::from_rgb(0.5, 0.5, 0.55),
                border_radius: 20.0,
                ..appearance
            },
        }
    }
}

impl From<Button> for iced::theme::Button {
    fn from(style: Button) -> Self {
        iced::theme::Button::Custom(Box::new(style))
    }
}

pub struct Checkbox;

impl Checkbox {
    const BORDER_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.55);
}

impl checkbox::StyleSheet for Checkbox {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: Background::Color(Color::TRANSPARENT),
            border_color: Self::BORDER_COLOR,
            ..style.active(&iced::theme::Checkbox::Primary, is_checked)
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            border_color: Self::BORDER_COLOR,
            ..style.hovered(&iced::theme::Checkbox::Primary, is_checked)
        }
    }
}

impl From<Checkbox> for iced::theme::Checkbox {
    fn from(style: Checkbox) -> Self {
        iced::theme::Checkbox::Custom(Box::new(style))
    }
}

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

pub struct Radio;

impl Radio {
    const BORDER_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.55);
}

impl radio::StyleSheet for Radio {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style, is_selected: bool) -> radio::Appearance {
        radio::Appearance {
            background: Background::Color(Color::TRANSPARENT),
            dot_color: Self::BORDER_COLOR,
            border_color: Self::BORDER_COLOR,
            ..style.active(&iced::theme::Radio::Default, is_selected)
        }
    }

    fn hovered(&self, style: &Self::Style, is_selected: bool) -> radio::Appearance {
        radio::Appearance {
            background: Background::Color(Color::from_rgb8(101, 101, 102)),
            border_color: Self::BORDER_COLOR,
            ..style.hovered(&iced::theme::Radio::Default, is_selected)
        }
    }
}

impl From<Radio> for iced::theme::Radio {
    fn from(style: Radio) -> Self {
        iced::theme::Radio::Custom(Box::new(style))
    }
}

pub struct Toggler;

impl Toggler {
    const ACTIVE_BACKGROUND: Color = Color::from_rgb(0.55, 0.0, 0.0);
}

impl toggler::StyleSheet for Toggler {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style, is_selected: bool) -> toggler::Appearance {
        let mut appearance = style.active(&iced::theme::Toggler::Default, is_selected);
        if is_selected {
            appearance.background = Self::ACTIVE_BACKGROUND;
        }

        appearance
    }

    fn hovered(&self, style: &Self::Style, is_selected: bool) -> toggler::Appearance {
        let mut appearance = style.hovered(&iced::theme::Toggler::Default, is_selected);
        if is_selected {
            appearance.background = Self::ACTIVE_BACKGROUND;
        }

        appearance
    }
}

impl From<Toggler> for iced::theme::Toggler {
    fn from(style: Toggler) -> Self {
        iced::theme::Toggler::Custom(Box::new(style))
    }
}
