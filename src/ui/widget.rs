use std::{borrow::Cow, fmt, marker::PhantomData};

use iced::{
    alignment::Horizontal,
    widget::{
        column, container, row, text, vertical_space, Button, Checkbox, Column, Container,
        PickList, Radio, Text, TextInput, Toggler,
    },
    Alignment, Element, Length,
};

use crate::jstation::data::{BoolParameter, DiscreteParameter, Normal};
use crate::ui::style;

pub const DEFAULT_DSP_WIDTH: Length = Length::Units(622);
pub const DSP_PROGRAM_SPACING: Length = Length::Units(10);

pub fn button<'a, Message>(title: &str) -> Button<'a, Message, iced::Renderer> {
    Button::new(text(title).size(15))
}

pub fn checkbox<'a, Message, F>(
    is_checked: bool,
    title: impl Into<String>,
    f: F,
) -> Checkbox<'a, Message, iced::Renderer>
where
    Message: 'a,
    F: 'a + Fn(bool) -> Message,
{
    Checkbox::new(title, is_checked, f)
        .size(16)
        .text_size(15)
        .spacing(6)
        .style(style::Checkbox)
}

pub fn settings_checkbox<'a, Message, F>(
    is_checked: bool,
    title: impl Into<String>,
    f: F,
) -> Checkbox<'a, Message, iced::Renderer>
where
    Message: 'a,
    F: 'a + Fn(bool) -> Message,
{
    Checkbox::new(title, is_checked, f)
        .size(19)
        .text_size(18)
        .style(style::Checkbox)
}

pub fn dsp<'a, Message>(
    title_area: Column<'a, Message, iced::Renderer>,
    element: impl Into<Element<'a, Message, iced::Renderer>>,
) -> Container<'a, Message>
where
    Message: 'a,
{
    container(row![title_area.width(Length::Units(270)), element.into()].padding(8))
        .width(DEFAULT_DSP_WIDTH)
        .style(style::DspContainer)
}

pub fn dsp_keep_width<'a, Message>(
    element: impl Into<Element<'a, Message, iced::Renderer>>,
) -> Container<'a, Message>
where
    Message: 'a,
{
    container(row![element.into()].padding(8)).style(style::DspContainer)
}

pub fn label<'a>(text: impl Into<Cow<'a, str>>) -> Text<'a, iced::Renderer> {
    Text::new(text).size(18)
}

pub fn amp_cabinet_label<'a>(text: impl Into<Cow<'a, str>>) -> Text<'a, iced::Renderer> {
    label(text).width(Length::Units(85))
}

pub fn param_label<'a>(text: impl Into<Cow<'a, str>>) -> Text<'a, iced::Renderer> {
    label(text).width(Length::Units(55))
}

pub fn value_label(text: impl ToString) -> Text<'static, iced::Renderer> {
    Text::new(text.to_string()).size(14)
}

pub fn modal<'a, Message>(
    title: &str,
    element: impl Into<Element<'a, Message, iced::Renderer>>,
    on_hide: Message,
) -> Container<'a, Message>
where
    Message: 'a + Clone,
{
    const CLOSE_BTN_WIDTH: u16 = 25;

    container(
        column![
            row![
                container(text(title)).width(Length::Fill).center_x(),
                Button::new(text("X").size(15).horizontal_alignment(Horizontal::Center))
                    .on_press(on_hide)
                    .width(Length::Units(CLOSE_BTN_WIDTH))
                    .style(style::Button::ModalClose.into()),
            ]
            .align_items(Alignment::Center),
            vertical_space(Length::Units(30)),
            container(element.into()).width(Length::Fill).center_x(),
        ]
        .width(Length::Units(350)),
    )
    .width(Length::Fill)
    .center_x()
    .height(Length::Fill)
    .center_y()
}

pub fn pick_list<'a, T, Message>(
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl 'a + Fn(T) -> Message,
) -> PickList<'a, T, Message, iced::Renderer>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'a,
{
    PickList::new(options, selected, on_selected).text_size(15)
}

pub fn radio<V, Message>(
    label: &str,
    value: V,
    selected: Option<V>,
    f: impl Fn(V) -> Message,
) -> Radio<Message, iced::Renderer>
where
    Message: Clone,
    V: Eq + Copy,
{
    Radio::new(value, label, selected, f)
        .size(16)
        .text_size(18)
        .spacing(5)
        .style(style::Radio)
}

pub fn switch<'a, Field, Message, OnChange, Output>(
    name: &'a str,
    field: Field,
    on_change: OnChange,
) -> Column<'a, Message>
where
    Field: BoolParameter,
    Message: 'a,
    Output: Into<Message>,
    OnChange: 'a + Fn(bool) -> Output,
{
    column![
        label(name),
        vertical_space(iced::Length::Units(10)),
        toggler(field.is_true(), move |is_true| (on_change)(is_true).into())
    ]
    .align_items(Alignment::Start)
}

pub fn text_input<'a, Message>(
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> TextInput<'a, Message, iced::Renderer>
where
    Message: 'a + Clone,
{
    TextInput::new(placeholder, value, on_change).size(15)
}

pub fn toggler<'a, Message>(
    is_active: bool,
    f: impl 'a + Fn(bool) -> Message,
) -> Toggler<'a, Message, iced::Renderer> {
    Toggler::new(None, is_active, f)
        .width(Length::Shrink)
        .style(style::Toggler)
}

#[track_caller]
fn build_knob<'a, Field, Message, OnChange, OnRelease, Output>(
    field: Field,
    name: Option<&'a str>,
    on_change: OnChange,
    on_release: Option<OnRelease>,
) -> Column<'a, Message>
where
    Field: DiscreteParameter + fmt::Display + fmt::Debug,
    Message: 'a,
    Output: Into<Message>,
    OnChange: 'a + Fn(Normal) -> Output,
    OnRelease: 'a + Fn() -> Option<Output>,
{
    let mut knob = iced_audio::Knob::new(to_ui_param(field), move |normal| {
        (on_change)(to_jstation_normal(normal)).into()
    })
    .size(Length::Units(35));

    if let Some(on_release) = on_release {
        knob = knob.on_release(move || on_release().map(Into::into));
    }

    column![
        param_label(name.unwrap_or_else(|| field.param_name()))
            .horizontal_alignment(iced::alignment::Horizontal::Center),
        knob,
        value_label(field),
    ]
    .spacing(5)
    .align_items(Alignment::Center)
}

pub fn knob<'a, Field, Message, OnChange, Output>(
    field: Field,
    on_change: OnChange,
) -> KnobBuilder<'a, Field, Message, OnChange, Output>
where
    Field: DiscreteParameter + fmt::Display,
    Message: 'a,
    Output: Into<Message>,
    OnChange: Fn(Normal) -> Output,
{
    KnobBuilder {
        field,
        on_change,
        name: None,
        phantom: PhantomData,
    }
}

pub struct KnobBuilder<'a, Field, Message, OnChange, Output>
where
    Field: DiscreteParameter + fmt::Display,
    Message: 'a,
    Output: Into<Message>,
    OnChange: 'a + Fn(Normal) -> Output,
{
    field: Field,
    on_change: OnChange,
    name: Option<&'a str>,
    phantom: PhantomData<&'a Message>,
}

impl<'a, Field, Message, OnChange, Output> KnobBuilder<'a, Field, Message, OnChange, Output>
where
    Field: DiscreteParameter + fmt::Display + fmt::Debug,
    Message: 'a,
    Output: 'a + Into<Message>,
    OnChange: Fn(Normal) -> Output,
{
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn on_release<OnRelease>(
        self,
        on_release: OnRelease,
    ) -> KnobBuilderOnRelease<'a, Field, Message, OnChange, OnRelease, Output>
    where
        OnRelease: Fn() -> Option<Output>,
    {
        KnobBuilderOnRelease {
            field: self.field,
            on_change: self.on_change,
            name: self.name,
            on_release,
            phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn build(self) -> Column<'a, Message> {
        let on_release_none = Option::<fn() -> Option<Output>>::None;
        build_knob(self.field, self.name, self.on_change, on_release_none)
    }
}

pub struct KnobBuilderOnRelease<'a, Field, Message, OnChange, OnRelease, Output>
where
    Field: DiscreteParameter + fmt::Display,
    Message: 'a,
    Output: Into<Message>,
    OnChange: Fn(Normal) -> Output,
    OnRelease: Fn() -> Option<Output>,
{
    field: Field,
    on_change: OnChange,
    name: Option<&'a str>,
    on_release: OnRelease,
    phantom: PhantomData<&'a Message>,
}

impl<'a, Field, Message, OnChange, OnRelease, Output>
    KnobBuilderOnRelease<'a, Field, Message, OnChange, OnRelease, Output>
where
    Field: DiscreteParameter + fmt::Display + fmt::Debug,
    Message: 'a,
    Output: Into<Message>,
    OnChange: 'a + Fn(Normal) -> Output,
    OnRelease: 'a + Fn() -> Option<Output>,
{
    #[track_caller]
    pub fn build(self) -> Column<'a, Message> {
        build_knob(self.field, self.name, self.on_change, Some(self.on_release))
    }
}

#[inline]
fn to_ui_normal(normal: crate::jstation::data::Normal) -> iced_audio::Normal {
    // Safety: jstation's `Normal` is a newtype on an `f32` in (0.0..=1.0)
    // which is the inner type and invariant for `iced_audio::Normal`.
    unsafe { std::mem::transmute(normal) }
}

#[track_caller]
#[inline]
fn to_ui_param<P>(param: P) -> iced_audio::NormalParam
where
    P: crate::jstation::data::DiscreteParameter + fmt::Debug,
{
    let (Some(normal), Some(default)) = (param.normal(), param.normal_default()) else {
        panic!("Attempt to get a value from an inactive parameter {param:?}");
    };

    let value = to_ui_normal(normal);
    let default = to_ui_normal(default);

    iced_audio::NormalParam { value, default }
}

fn to_jstation_normal(normal: iced_audio::Normal) -> crate::jstation::data::Normal {
    // Safety: jstation's `Normal` is a newtype on an `f32` in (0.0..=1.0)
    // which is the inner type and invariant for `iced_audio::Normal`.
    unsafe { std::mem::transmute(normal) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn to_ui_normal() {
        use super::to_ui_normal;
        use crate::jstation::data::Normal;

        const JS_MIN: Normal = Normal::MIN;
        const JS_CENTER: Normal = Normal::CENTER;
        const JS_MAX: Normal = Normal::MAX;

        assert_eq!(to_ui_normal(JS_MIN).as_f32(), JS_MIN.as_ratio());
        assert_eq!(to_ui_normal(JS_CENTER).as_f32(), JS_CENTER.as_ratio());
        assert_eq!(to_ui_normal(JS_MAX).as_f32(), JS_MAX.as_ratio());

        let less_than_min_res = Normal::try_from(0.0 - f32::EPSILON);
        assert!(less_than_min_res.is_err());

        let more_than_max_res = Normal::try_from(1.0 + f32::EPSILON);
        assert!(more_than_max_res.is_err());
    }

    #[test]
    fn to_jstation_normal() {
        use super::to_jstation_normal;
        use iced_audio::Normal;

        const UI_MIN: Normal = Normal::MIN;
        const UI_CENTER: Normal = Normal::CENTER;
        const UI_MAX: Normal = Normal::MAX;

        assert!((to_jstation_normal(UI_MIN).as_ratio() - UI_MIN.as_f32()).abs() < 0.005);
        assert!((to_jstation_normal(UI_CENTER).as_ratio() - UI_CENTER.as_f32()).abs() < 0.005);
        assert!((to_jstation_normal(UI_MAX).as_ratio() - UI_MAX.as_f32()).abs() < 0.005);

        let clipped_less_than_min = Normal::from_clipped(0.0 - f32::EPSILON);
        assert!(
            (to_jstation_normal(clipped_less_than_min).as_ratio() - UI_MIN.as_f32()).abs() < 0.005
        );

        let clipped_more_than_max = Normal::from_clipped(1.0 + f32::EPSILON);
        assert!(
            (to_jstation_normal(clipped_more_than_max).as_ratio() - UI_MAX.as_f32()).abs() < 0.005
        );
    }
}
