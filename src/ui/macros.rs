macro_rules! param_knob {
    ($group:ident, $param:ident, $variant:ident, display_raw) => {
        param_knob!($group, $param, $variant, format!("{:02}", $group.$param))
    };
    ($group:ident, $param:ident, $variant:ident, $display:expr) => {
        column![
            text($group::$variant::NAME),
            Knob::new(to_ui_param($group.$param), |normal| {
                $variant($group::$variant::from_snapped(to_jstation_normal(normal))).into()
            })
            .size(iced::Length::Units(35)),
            text($display),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
    };
}

macro_rules! param_switch {
    ($name:literal, $group:ident, $param:ident, $variant:ident) => {
        column![
            text($name),
            toggler("".to_string(), $group.$param.is_active(), |is_active| {
                $variant(is_active.into())
            })
            .width(Length::Shrink)
        ]
        .spacing(10)
        .align_items(Alignment::Start)
    };
}
