use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::param::{Arg, ParamBase};

pub struct Boolean<'a> {
    pub base: ParamBase<'a>,
}

impl<'a> Boolean<'a> {
    pub fn new(mut base: ParamBase<'a>, args: impl Iterator<Item = Arg>) -> Self {
        for arg in args {
            base.have_arg(arg);
        }

        Boolean { base }
    }
}

impl<'a> ToTokens for Boolean<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = &self.base.field.ty;
        let param_name = self.base.name();

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            pub struct #param(bool);

            impl crate::jstation::data::ParameterSetter for #param {
                type Parameter = Self;

                fn set(&mut self, new: Self) -> Option<Self> {
                    if *self == new {
                        return None;
                    }

                    *self = new;

                    Some(new)
                }
            }

            impl crate::jstation::data::BoolParameter for #param {
                const TRUE: Self = #param(true);
                const FALSE: Self = #param(false);

                fn param_name(self) -> &'static str {
                    #param_name
                }
            }

            impl From<bool> for #param {
                fn from(value: bool) -> Self {
                    #param(value)
                }
            }

            impl From<#param> for bool {
                fn from(param: #param) -> Self {
                    param.0
                }
            }
        });

        if let Some(param_nb) = &self.base.param_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::ProgramParameter for #param {
                    fn set_from(
                        &mut self,
                        data: &crate::jstation::ProgramData,
                    ) -> Result<(), crate::jstation::Error> {
                        use crate::jstation::data::ParameterNumber;
                        const PARAM_NB: ParameterNumber = ParameterNumber::new(#param_nb);

                        // `PARAM_NB` is guaranteed to be in the range of the constant sized
                        // array returned by `data.buf()`, bound checking should get optimized out.
                        self.0 = data.buf()[PARAM_NB.as_usize()].as_u8() != 0;

                        Ok(())
                    }

                    #[inline]
                    fn has_changed(&self, data: &crate::jstation::ProgramData) -> bool {
                        use crate::jstation::data::ParameterNumber;
                        const PARAM_NB: ParameterNumber = ParameterNumber::new(#param_nb);

                        // `PARAM_NB` is guaranteed to be in the range of the constant sized
                        // array returned by `data.buf()`, bound checking should get optimized out.
                        let data_bool = data.buf()[PARAM_NB.as_usize()].as_u8() != 0;
                        data_bool != self.0
                    }

                    #[inline]
                    fn store(&mut self, data: &mut crate::jstation::ProgramData) {
                        use crate::jstation::data::BoolParameter;
                        use crate::jstation::data::ParameterNumber;
                        const PARAM_NB: ParameterNumber = ParameterNumber::new(#param_nb);

                        // `PARAM_NB` is guaranteed to be in the range of the constant sized
                        // array returned by `data.buf_mut()`, bound checking should get optimized out.
                        data.buf_mut()[PARAM_NB.as_usize()] = self.raw_value();
                    }
                }
            });
        }

        if let Some(cc_nb) = &self.base.cc_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::CCParameter for #param {
                    fn to_cc(self) -> Option<crate::midi::CC> {
                        use crate::midi;

                        let value = if self.into() {
                            midi::CCValue::MAX
                        } else {
                            midi::CCValue::ZERO
                        };

                        Some(midi::CC::new(midi::CCNumber::new(#cc_nb), value))
                    }
                }

                impl crate::jstation::data::CCParameterSetter for #param {
                    type Parameter = Self;

                    fn set_cc(
                        &mut self,
                        cc: crate::midi::CC,
                    ) -> Result<Option<Self>, crate::jstation::Error>
                    {
                        const CC_TRUE_THRESHOLD: u8 = 0x40;

                        assert_eq!(cc.nb.as_u8(), #cc_nb);

                        let value = cc.value.as_u8() >= CC_TRUE_THRESHOLD;
                        if self.0 == value {
                            return Ok(None);
                        }

                        *self = #param(value);

                        Ok(Some(*self))
                    }
                }
            });
        }
    }
}
