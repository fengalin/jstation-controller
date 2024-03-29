use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{self, Expr, Ident};

use crate::param::{Arg, ParamBase};

#[derive(Default)]
enum DefaultPos {
    Center,
    Max,
    #[default]
    Min,
}

pub struct ConstRange<'a> {
    pub base: ParamBase<'a>,
    is_discr: bool,
    default_pos: DefaultPos,
    displays: Vec<Display>,
    min: Option<u8>,
    max: Option<u8>,
}

impl<'a> ConstRange<'a> {
    pub fn new(base: ParamBase<'a>, args: impl Iterator<Item = Arg>) -> Self {
        let mut this = ConstRange {
            base,
            is_discr: false,
            default_pos: DefaultPos::default(),
            displays: Vec::new(),
            min: None,
            max: None,
        };

        for arg in args {
            this.have_arg(arg);
        }

        this
    }

    fn have_arg(&mut self, arg: Arg) {
        let name = arg.name.to_string();
        match name.as_str() {
            "min" => self.min = Some(arg.u8_or_abort(self.base.field)),
            "max" => self.max = Some(arg.u8_or_abort(self.base.field)),
            "default_center" => {
                arg.no_value_or_abort(self.base.field);
                self.default_pos = DefaultPos::Center;
            }
            "default_max" => {
                arg.no_value_or_abort(self.base.field);
                self.default_pos = DefaultPos::Max;
            }
            "display_cents" => {
                arg.no_value_or_abort(self.base.field);
                self.displays.push(Display::Cents);
            }
            "display_raw" => {
                arg.no_value_or_abort(self.base.field);
                self.displays.push(Display::Raw);
            }
            "display_map" => {
                let list_expr = arg.value_or_abort(self.base.field);
                let path = match list_expr {
                    Expr::Path(expr_path) => expr_path.path,
                    _ => panic!(
                        "Field {}: unexpected `display_map` expression",
                        self.base.field.to_token_stream(),
                    ),
                };

                let name = match path.get_ident() {
                    Some(name) => name,
                    None => {
                        panic!(
                            "Field {}: expecting ident for `display_map`",
                            self.base.field.to_token_stream(),
                        )
                    }
                };

                self.displays.push(Display::Map(name.clone()));
            }
            "discriminant" => {
                arg.no_value_or_abort(self.base.field);
                self.is_discr = true;
            }
            _ => self.base.have_arg(arg),
        }
    }

    pub fn is_discriminant(&self) -> bool {
        self.is_discr
    }
}

impl<'a> ToTokens for ConstRange<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = &self.base.field.ty;
        let param_name = self.base.name();
        let param_min = self.min.unwrap_or(0);
        let param_max = self.max.unwrap_or_else(|| {
            panic!(
                "Field {}: undefined `max` attribute for {}",
                self.base.field.to_token_stream(),
                param.to_token_stream(),
            )
        });
        if param_max < param_min {
            panic!(
                "Field {}: `max` is less then `min` for {}",
                self.base.field.to_token_stream(),
                param.to_token_stream(),
            );
        }
        let (param_default, normal_default) = match self.default_pos {
            DefaultPos::Center => (
                ((param_max as u16 + param_min as u16) / 2) as u8,
                quote! { CENTER },
            ),
            DefaultPos::Max => (param_max, quote! { MAX }),
            DefaultPos::Min => (param_min, quote! { MIN }),
        };

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct #param(crate::jstation::data::RawValue);

            impl From<#param> for crate::jstation::data::RawValue {
                fn from(param: #param) -> Self {
                    param.0
                }
            }

            impl crate::jstation::data::ConstRangeParameter for #param {
                const RANGE: crate::jstation::data::DiscreteRange =
                    crate::jstation::data::DiscreteRange::new(
                        crate::jstation::data::RawValue::new(#param_min),
                        crate::jstation::data::RawValue::new(#param_max),
                    );

                fn from_normal(normal: crate::jstation::data::Normal) -> Self {
                    use crate::jstation::data::ConstRangeParameter;
                    #param(Self::RANGE.normal_to_raw(normal))
                }

                fn try_from_raw(
                    raw: crate::jstation::data::RawValue,
                ) -> Result<Self, crate::jstation::Error> {
                    use crate::jstation::data::ConstRangeParameter;
                    let value = Self::RANGE
                        .check(raw)
                        .map_err(|err| crate::jstation::Error::with_context(#param_name, err))?;

                    Ok(#param(value))
                }
            }

            impl crate::jstation::data::DiscreteParameter for #param {
                fn param_name(self) -> &'static str {
                    #param_name
                }

                fn normal_default(self) -> Option<crate::jstation::data::Normal> {
                    Some(crate::jstation::data::Normal::#normal_default)
                }

                fn normal(self) -> Option<crate::jstation::data::Normal> {
                    use crate::jstation::data::ConstRangeParameter;
                    Some(Self::RANGE.try_normalize(self.0).unwrap())
                }

                fn reset(&mut self) -> Option<Self> {
                    let default = Self::default();
                    if *self == default {
                        return None;
                    }

                    Some(default)
                }
            }

            impl crate::jstation::data::ParameterSetter for #param {
                type Parameter = Self;

                fn set(&mut self, new: Self) -> Option<Self> {
                    if self.0 == new.0 {
                        return None;
                    }

                    *self = new;

                    Some(new)
                }
            }

            impl Default for #param {
                fn default() -> Self {
                    Self(crate::jstation::data::RawValue::new(#param_default))
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
                        use crate::jstation::data::ConstRangeParameter;
                        use crate::jstation::data::ParameterNumber;
                        const PARAM_NB: ParameterNumber = ParameterNumber::new(#param_nb);

                        // `PARAM_NB` is guaranteed to be in the range of the constant sized
                        // array returned by `data.buf()`, bound checking should get optimized out.
                        *self = Self::try_from_raw(data.buf()[PARAM_NB.as_usize()])?;

                        Ok(())
                    }

                    #[inline]
                    fn has_changed(&self, data: &crate::jstation::ProgramData) -> bool {
                        use crate::jstation::data::ParameterNumber;
                        const PARAM_NB: ParameterNumber = ParameterNumber::new(#param_nb);

                        // `PARAM_NB` is guaranteed to be in the range of the constant sized
                        // array returned by `data.buf()`, bound checking should get optimized out.
                        data.buf()[PARAM_NB.as_usize()] != self.0
                    }

                    #[inline]
                    fn store(&mut self, data: &mut crate::jstation::ProgramData) {
                        use crate::jstation::data::ParameterNumber;
                        const PARAM_NB: ParameterNumber = ParameterNumber::new(#param_nb);

                        // `PARAM_NB` is guaranteed to be in the range of the constant sized
                        // array returned by `data.buf_mut()`, bound checking should get optimized out.
                        data.buf_mut()[PARAM_NB.as_usize()] = self.0;
                    }
                }
            });
        }

        if let Some(cc_nb) = &self.base.cc_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::CCParameter for #param {
                    fn to_cc(self) -> Option<crate::midi::CC> {
                        use crate::jstation::data::ConstRangeParameter;

                        Some(crate::midi::CC::new(
                            crate::midi::CCNumber::new(#cc_nb),
                            Self::RANGE.try_ccize(self.0).unwrap(),
                        ))
                    }
                }

                impl crate::jstation::data::CCParameterSetter for #param {
                    type Parameter = Self;

                    fn set_cc(
                        &mut self,
                        cc: crate::midi::CC,
                    ) -> Result<Option<Self>, crate::jstation::Error> {
                        use crate::jstation::data::ConstRangeParameter;

                        assert_eq!(cc.nb.as_u8(), #cc_nb);

                        let value = Self::RANGE.cc_to_raw(cc.value);

                        if self.0 == value {
                            return Ok(None);
                        }

                        *self = #param(value);

                        Ok(Some(*self))
                    }
                }
            });
        }

        for display in self.displays.iter() {
            match display {
                Display::Cents => tokens.extend(quote! {
                    impl std::fmt::Display for #param {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            use crate::jstation::data::ConstRangeParameter;
                            std::fmt::Display::fmt(&Self::RANGE.to_cents(self.0).unwrap(), f)
                        }
                    }
                }),
                Display::Raw => tokens.extend(quote! {
                    impl std::fmt::Display for #param {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            std::fmt::Display::fmt(&self.0, f)
                        }
                    }
                }),
                Display::Map(name) => {
                    use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

                    let name_str = name.to_string();

                    let name_as_type = name_str.to_upper_camel_case();
                    let names_param_str = &format!("{}{}", param.to_token_stream(), name_as_type);
                    let named_param = Ident::new(names_param_str, self.base.field.span());

                    let named_list = Ident::new(
                        format!("{}S", &names_param_str.to_shouty_snake_case()).as_str(),
                        self.base.field.span(),
                    );
                    let expected_list_len = (param_max - param_min) as usize + 1;

                    let name_as_field = name_str.to_snake_case();
                    let name_method = Ident::new(&name_as_field, self.base.field.span());
                    let names_method =
                        Ident::new(format!("{name_as_field}s").as_str(), self.base.field.span());

                    tokens.extend(quote! {
                        #[derive(Clone, Copy, Debug)]
                        pub struct #named_param {
                            idx: usize,
                            param: #param,
                            name: &'static str,
                        }

                        impl #named_param {
                            pub fn param(self) -> #param {
                                self.param
                            }
                        }

                        impl std::fmt::Display for #named_param {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                std::fmt::Display::fmt(self.name, f)
                            }
                        }

                        impl PartialEq for #named_param {
                            fn eq(&self, other: &Self) -> bool {
                                self.idx == other.idx
                            }
                        }

                        impl Eq for #named_param {}

                        impl #param {
                            pub fn #names_method() -> &'static [#named_param] {
                                static LIST: once_cell::sync::Lazy<Vec<#named_param>> =
                                    once_cell::sync::Lazy::new(|| {
                                        use crate::jstation::data::ConstRangeParameter;

                                        assert_eq!(
                                            #named_list.len(),
                                            #expected_list_len,
                                            concat!(
                                                stringify!(#named_list),
                                                " list len and ",
                                                stringify!(#param),
                                                " range mismatch",
                                            ),
                                        );

                                        Vec::<#named_param>::from_iter(#named_list.into_iter().enumerate().map(
                                            |(idx, name)| {
                                                let param = <#param>::try_from_raw(
                                                    crate::jstation::data::RawValue::new(idx as u8),
                                                )
                                                .expect("Param names and range mismatch");

                                                #named_param { idx, param, name }
                                            },
                                        ))
                                    });

                                &*LIST
                            }

                            pub fn #name_method(self) -> #named_param {
                                Self::#names_method()[self.0.as_u8() as usize]
                            }
                        }
                    });
                }
            }
        }
    }
}

enum Display {
    Cents,
    Raw,
    Map(Ident),
}
