use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::param::{Arg, ParamBase};

pub struct VariableRange<'a> {
    pub base: ParamBase<'a>,
}

impl<'a> VariableRange<'a> {
    pub fn new(mut base: ParamBase<'a>, args: impl Iterator<Item = Arg>) -> Self {
        for arg in args {
            base.have_arg(arg);
        }

        VariableRange { base }
    }
}

impl<'a> ToTokens for VariableRange<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = &self.base.field.ty;
        let param_name = self.base.name();

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            pub struct #param {
                discr: <Self as crate::jstation::data::VariableRange>::Discriminant,
                value: crate::jstation::data::RawValue,
            }

            impl crate::jstation::data::VariableRangeParameter for #param {
                fn range(self) -> Option<DiscreteRange> {
                    <Self as crate::jstation::data::VariableRange>::range_from(self.discr)
                }

                fn set_discriminant(&mut self, discr: Self::Discriminant) {
                    if self.discr != discr {
                        let cur_range = <Self as crate::jstation::data::VariableRange>::range_from(self.discr);
                        self.discr = discr;

                        if let Some(range) = <Self as crate::jstation::data::VariableRange>::range_from(discr) {
                            if cur_range.map_or(true, |cur_range| cur_range != range) {
                                crate::jstation::data::DiscreteParameter::reset(self);
                            }
                        }
                    }
                }

                fn from_normal(
                    discr: <Self as crate::jstation::data::VariableRange>::Discriminant,
                    normal: crate::jstation::data::Normal,
                ) -> Option<Self> {
                    let range = <Self as crate::jstation::data::VariableRange>::range_from(discr)?;
                    let value = range.normal_to_raw(normal);

                    Some(#param {
                        discr,
                        value,
                    })
                }

                fn try_from_raw(
                    discr: <Self as crate::jstation::data::VariableRange>::Discriminant,
                    raw: crate::jstation::data::RawValue,
                ) -> Result<Self, crate::jstation::Error> {
                    use crate::jstation::Error;

                    let range = <Self as crate::jstation::data::VariableRange>::range_from(discr)
                        .ok_or_else(|| Error::ParameterInactive(stringify!(#param).into()))?;
                    let value = range
                        .check(raw)
                        .map_err(|err| crate::jstation::Error::with_context(#param_name, err))?;

                    Ok(#param {
                        discr,
                        value,
                    })
                }
            }

            impl crate::jstation::data::ParameterSetter for #param {
                type Parameter = Self;

                fn set(&mut self, new: Self) -> Option<Self> {
                    use crate::jstation::data::DiscreteParameter;

                    if self.discr != new.discr {
                        *self = new;
                        return Some(new);
                    }

                    if !self.is_active() || self.value == new.value {
                        return None;
                    }

                    self.value = new.value;

                    Some(new)
                }
            }
        });

        if let Some(param_nb) = &self.base.param_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::RawParameter for #param {
                    fn set_raw(
                        &mut self,
                        data: &[crate::jstation::data::RawValue],
                    ) -> Result<(), crate::jstation::Error> {
                        use crate::jstation::data::VariableRangeParameter;
                        let param = Self::try_from_raw(self.discr, data[#param_nb])?;

                        *self = param;

                        Ok(())
                    }
                }
            });
        }

        if let Some(cc_nb) = &self.base.cc_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::CCParameter for #param {
                    fn to_cc(self) -> Option<crate::midi::CC> {
                        use crate::jstation::data::VariableRangeParameter;

                        Some(crate::midi::CC::new(
                            crate::midi::CCNumber::new(#cc_nb),
                            self.range()?.try_ccize(self.value).unwrap(),
                        ))
                    }
                }

                impl crate::jstation::data::CCParameterSetter for #param {
                    type Parameter = Self;

                    fn set_cc(
                        &mut self,
                        cc: crate::midi::CC,
                    ) -> Result<Option<Self>, crate::jstation::Error>
                    {
                        use crate::jstation::{data::VariableRangeParameter, Error};

                        assert_eq!(cc.nb.as_u8(), #cc_nb);

                        let range = self.range()
                            .ok_or_else(|| Error::ParameterInactive(stringify!(#param).into()))?;
                        let value = range.cc_to_raw(cc.value);

                        if self.value == value {
                            return Ok(None);
                        }

                        self.value = value;

                        Ok(Some(*self))
                    }
                }
            });
        }
    }
}
