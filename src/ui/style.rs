use iced::{
    widget::{button, checkbox, container, radio, toggler},
    Color,
};

pub struct Background;

impl container::StyleSheet for Background {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let appearance = style.appearance(&iced::theme::Container::default());
        match style {
            iced::Theme::Light => container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.99, 0.99))),
                ..appearance
            },
            _ => appearance,
        }
    }
}

impl From<Background> for iced::theme::Container {
    fn from(style: Background) -> Self {
        iced::theme::Container::Custom(Box::new(style))
    }
}

#[derive(Clone, Copy)]
pub enum Button {
    Default,
    ModalClose,
    ListItem,
    ListItemSelected,
    Store,
}

impl button::StyleSheet for Button {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let appearance = style.active(&iced::theme::Button::Primary);

        use Button::*;
        match self {
            Default => button::Appearance {
                background: match style {
                    iced::Theme::Dark => {
                        Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2)))
                    }
                    iced::Theme::Light => {
                        Some(iced::Background::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    }
                    _ => appearance.background,
                },
                text_color: Color::WHITE,
                ..appearance
            },
            ListItem => button::Appearance {
                background: match style {
                    iced::Theme::Dark => {
                        Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1)))
                    }
                    iced::Theme::Light => {
                        Some(iced::Background::Color(Color::from_rgb(0.97, 0.98, 0.98)))
                    }
                    _ => appearance.background,
                },
                text_color: match style {
                    iced::Theme::Dark => Color::from_rgb(0.7, 0.7, 0.75),
                    iced::Theme::Light => Color::from_rgb(0.45, 0.4, 0.4),
                    _ => appearance.text_color,
                },
                ..appearance
            },
            ListItemSelected | Store => button::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.55, 0.0, 0.0))),
                text_color: Color::from_rgb(0.9, 0.9, 0.95),
                ..appearance
            },
            ModalClose => button::Appearance {
                background: None,
                text_color: match style {
                    iced::Theme::Dark => Color::from_rgb(0.7, 0.7, 0.75),
                    iced::Theme::Light => Color::from_rgb(0.25, 0.2, 0.2),
                    _ => appearance.text_color,
                },
                ..appearance
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let appearance = style.hovered(&iced::theme::Button::Primary);

        use Button::*;
        match self {
            Default => button::Appearance {
                background: match style {
                    iced::Theme::Dark => {
                        Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.3)))
                    }
                    iced::Theme::Light => {
                        Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.5)))
                    }
                    _ => appearance.background,
                },
                text_color: Color::WHITE,
                ..appearance
            },
            ListItem => button::Appearance {
                background: match style {
                    iced::Theme::Dark => {
                        Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2)))
                    }
                    iced::Theme::Light => {
                        Some(iced::Background::Color(Color::from_rgb(0.7, 0.7, 0.7)))
                    }
                    _ => appearance.background,
                },
                text_color: match style {
                    iced::Theme::Dark => Color::from_rgb(0.6, 0.6, 0.65),
                    iced::Theme::Light => Color::from_rgb(0.35, 0.3, 0.3),
                    _ => appearance.text_color,
                },
                ..appearance
            },
            ListItemSelected | Store => button::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.75, 0.0, 0.0))),
                text_color: Color::WHITE,
                ..appearance
            },
            ModalClose => button::Appearance {
                background: match style {
                    iced::Theme::Dark => {
                        Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2)))
                    }
                    iced::Theme::Light => {
                        Some(iced::Background::Color(Color::from_rgb(0.8, 0.8, 0.8)))
                    }
                    _ => appearance.background,
                },
                text_color: match style {
                    iced::Theme::Dark => Color::from_rgb(0.7, 0.7, 0.75),
                    iced::Theme::Light => Color::from_rgb(0.25, 0.2, 0.2),
                    _ => appearance.text_color,
                },
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
        let appearance = style.active(&iced::theme::Checkbox::Primary, is_checked);
        checkbox::Appearance {
            background: iced::Background::Color(Color::TRANSPARENT),
            border_color: Self::BORDER_COLOR,
            checkmark_color: match style {
                iced::Theme::Light => Color::from_rgb(0.5, 0.5, 0.55),
                _ => appearance.checkmark_color,
            },
            ..appearance
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let appearance = style.hovered(&iced::theme::Checkbox::Primary, is_checked);
        checkbox::Appearance {
            background: match style {
                iced::Theme::Dark => iced::Background::Color(Color::from_rgb8(101, 101, 102)),
                iced::Theme::Light => iced::Background::Color(Color::from_rgb8(202, 201, 201)),
                _ => appearance.background,
            },
            border_color: Self::BORDER_COLOR,
            ..appearance
        }
    }
}

impl From<Checkbox> for iced::theme::Checkbox {
    fn from(style: Checkbox) -> Self {
        iced::theme::Checkbox::Custom(Box::new(style))
    }
}

pub struct DspContainer;

impl container::StyleSheet for DspContainer {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let appearance = style.appearance(&iced::theme::Container::default());
        container::Appearance {
            background: match style {
                iced::Theme::Dark => {
                    Some(iced::Background::Color(Color::from_rgb(0.28, 0.28, 0.3)))
                }
                iced::Theme::Light => {
                    Some(iced::Background::Color(Color::from_rgb(0.925, 0.92, 0.92)))
                }
                _ => appearance.background,
            },
            border_radius: 4.0,
            border_color: match style {
                iced::Theme::Dark => Color::from_rgb(0.28, 0.28, 0.3),
                iced::Theme::Light => Color::from_rgb(0.925, 0.92, 0.92),
                _ => appearance.border_color,
            },
            ..appearance
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
        let appearance = style.active(&iced::theme::Radio::Default, is_selected);
        radio::Appearance {
            background: iced::Background::Color(Color::TRANSPARENT),
            dot_color: Self::BORDER_COLOR,
            border_color: Self::BORDER_COLOR,
            ..appearance
        }
    }

    fn hovered(&self, style: &Self::Style, is_selected: bool) -> radio::Appearance {
        let appearance = style.hovered(&iced::theme::Radio::Default, is_selected);
        radio::Appearance {
            background: match style {
                iced::Theme::Dark => iced::Background::Color(Color::from_rgb8(101, 101, 102)),
                iced::Theme::Light => iced::Background::Color(Color::from_rgb8(202, 201, 201)),
                _ => appearance.background,
            },
            border_color: Self::BORDER_COLOR,
            ..appearance
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
