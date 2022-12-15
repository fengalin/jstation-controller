macro_rules! param_knob {
    ($group:ident, $param:ident, $variant:ident) => {
        param_knob!(@name $group::$variant::NAME, $group, $param, $variant, $group.$param)
    };

    ($group:ident, $param:ident, $variant:ident, $display:expr $(,)?) => {
        param_knob!(@name $group::$variant::NAME, $group, $param, $variant, $display)
    };

    (@name $name:expr, $group:ident, $param:ident, $variant:ident $(,)?) => {
        param_knob!(@name $name, $group, $param, $variant, $group.$param)
    };

    (@name $name:expr, $group:ident, $param:ident, $variant:ident, $display:expr $(,)?) => {
        column![
            text($name)
                .size(crate::ui::LABEL_TEXT_SIZE)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(crate::ui::LABEL_WIDTH),
            Knob::new(to_ui_param($group.$param), |normal| {
                $variant($group::$variant::from_snapped(to_jstation_normal(normal))).into()
            })
            .size(crate::ui::KNOB_SIZE),
            text($display).size(crate::ui::VALUE_TEXT_SIZE),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
    };
}

macro_rules! param_switch {
    (@name $name:literal, $group:ident, $param:ident, $variant:ident) => {
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
