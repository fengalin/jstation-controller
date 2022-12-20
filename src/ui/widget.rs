use std::{fmt, marker::PhantomData};

use iced::{
    widget::{column, text, toggler, vertical_space, Column},
    Alignment, Length,
};

use crate::jstation::data::{BoolParameter, DiscreteParameter, Normal};

pub fn switch<'a, Field, Message, OnChange, Output>(
    name: impl ToString,
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
        text(name),
        vertical_space(iced::Length::Units(10)),
        toggler("".to_string(), field.is_true(), move |is_true| {
            (on_change)(is_true).into()
        })
        .width(Length::Shrink)
    ]
    .align_items(Alignment::Start)
}

fn build_knob<'a, Field, Message, OnChange, OnRelease, Output>(
    field: Field,
    name: Option<String>,
    on_change: OnChange,
    on_release: Option<OnRelease>,
) -> Column<'a, Message>
where
    Field: DiscreteParameter + fmt::Display,
    Message: 'a,
    Output: Into<Message>,
    OnChange: 'a + Fn(Normal) -> Output,
    OnRelease: 'a + Fn() -> Option<Output>,
{
    let mut knob = iced_audio::Knob::new(to_ui_param(field), move |normal| {
        (on_change)(to_jstation_normal(normal)).into()
    })
    .size(crate::ui::KNOB_SIZE);

    if let Some(on_release) = on_release {
        knob = knob.on_release(move || on_release().map(Into::into));
    }

    column![
        text(name.unwrap_or_else(|| field.name().to_string()))
            .size(crate::ui::LABEL_TEXT_SIZE)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(crate::ui::LABEL_WIDTH),
        knob,
        text(field).size(crate::ui::VALUE_TEXT_SIZE),
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
    name: Option<String>,
    phantom: PhantomData<&'a Message>,
}

impl<'a, Field, Message, OnChange, Output> KnobBuilder<'a, Field, Message, OnChange, Output>
where
    Field: DiscreteParameter + fmt::Display,
    Message: 'a,
    Output: 'a + Into<Message>,
    OnChange: Fn(Normal) -> Output,
{
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
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
    name: Option<String>,
    on_release: OnRelease,
    phantom: PhantomData<&'a Message>,
}

impl<'a, Field, Message, OnChange, OnRelease, Output>
    KnobBuilderOnRelease<'a, Field, Message, OnChange, OnRelease, Output>
where
    Field: DiscreteParameter + fmt::Display,
    Message: 'a,
    Output: Into<Message>,
    OnChange: 'a + Fn(Normal) -> Output,
    OnRelease: 'a + Fn() -> Option<Output>,
{
    pub fn build(self) -> Column<'a, Message> {
        build_knob(self.field, self.name, self.on_change, Some(self.on_release))
    }
}

fn to_ui_normal(normal: crate::jstation::data::Normal) -> iced_audio::Normal {
    // Safety: jstation's `Normal::as_ratio` returns an `f32` in (0.0..=1.0)
    // which is the inner type and invariant for `iced_audio::Normal`.
    unsafe { std::mem::transmute(normal.as_ratio()) }
}

#[track_caller]
fn to_ui_param<P>(param: P) -> iced_audio::NormalParam
where
    P: crate::jstation::data::DiscreteParameter,
{
    let (normal, default) = param
        .normal()
        .zip(param.normal_default())
        .expect("Attempt to get a value from an inactive parameter");
    let value = to_ui_normal(normal);
    let default = to_ui_normal(default);

    iced_audio::NormalParam { value, default }
}

#[track_caller]
fn to_jstation_normal(normal: iced_audio::Normal) -> crate::jstation::data::Normal {
    TryFrom::try_from(normal.as_f32()).unwrap()
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
