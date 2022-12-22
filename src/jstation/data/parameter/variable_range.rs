use crate::jstation::{
    data::{DiscreteRange, Normal, ParameterSetter, RawValue},
    Error,
};

pub trait VariableRangeParameter:
    VariableRange + ParameterSetter<Parameter = Self> + Clone + Copy
{
    fn range(self) -> Option<DiscreteRange>;
    fn set_discriminant(&mut self, discr: Self::Discriminant);

    fn from_normal(discr: Self::Discriminant, normal: Normal) -> Option<Self>;
    fn try_from_raw(discr: Self::Discriminant, raw: RawValue) -> Result<Self, Error>;
}

pub trait VariableRange: Clone + Copy {
    type Discriminant: Clone + Copy + Default + PartialEq;

    fn range_from(discr: Self::Discriminant) -> Option<DiscreteRange>;
}
