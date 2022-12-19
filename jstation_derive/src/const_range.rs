use heck::ToTitleCase;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{self, punctuated::Punctuated, Attribute, Expr, Field, Ident, Token, Type};

use crate::param::Arg;

pub struct ConstRange<'a> {
    field: &'a Field,
    is_discr: bool,
    default_center: bool,
    displays: Vec<Display>,
    min: Option<u8>,
    max: Option<u8>,
    param_nb: Option<u8>,
    cc_nb: Option<u8>,
}

impl<'a> ConstRange<'a> {
    fn new(field: &'a Field) -> Self {
        ConstRange {
            field,
            is_discr: false,
            default_center: false,
            displays: Vec::new(),
            min: None,
            max: None,
            param_nb: None,
            cc_nb: None,
        }
    }

    pub fn from_attrs(field: &'a Field, attr: &Attribute) -> Self {
        let mut param = ConstRange::new(field);

        let args = attr
            .parse_args_with(Punctuated::<Arg, Token![,]>::parse_terminated)
            .unwrap_or_abort();
        for arg in args {
            let name = arg.name.to_string();
            match name.as_str() {
                "min" => param.min = Some(arg.u8_or_abort(field)),
                "max" => param.max = Some(arg.u8_or_abort(field)),
                "default_center" => {
                    arg.no_value_or_abort(field);
                    param.default_center = true;
                }
                "display_raw" => {
                    arg.no_value_or_abort(field);
                    param.displays.push(Display::Raw);
                }
                "display_map" => {
                    let list_expr = arg.value_or_abort(field);
                    let path = match list_expr {
                        Expr::Path(expr_path) => expr_path.path,
                        _ => abort!(
                            field,
                            "Unexpected `display_map` expression for {}",
                            field.to_token_stream(),
                        ),
                    };

                    let name = match path.get_ident() {
                        Some(name) => name,
                        None => {
                            abort!(
                                field,
                                "Expecting ident for `display_map`, got {}",
                                field.to_token_stream()
                            )
                        }
                    };

                    param.displays.push(Display::Map(name.clone()));
                }
                "param_nb" => param.param_nb = Some(arg.u8_or_abort(field)),
                "cc_nb" => param.cc_nb = Some(arg.u8_or_abort(field)),
                "discriminant" => {
                    arg.no_value_or_abort(field);
                    param.is_discr = true;
                }
                other => {
                    abort!(
                        field,
                        "Incompatible arg `{other}` for `const_range` param {}",
                        field.ty.to_token_stream(),
                    );
                }
            }
        }

        param
    }

    pub fn typ(&self) -> &Type {
        &self.field.ty
    }

    pub fn field(&self) -> &Ident {
        self.field.ident.as_ref().expect("name field")
    }

    pub fn cc_nb(&self) -> Option<u8> {
        self.cc_nb
    }

    pub fn is_discriminant(&self) -> bool {
        self.is_discr
    }
}

impl<'a> ToTokens for ConstRange<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = self.typ();
        let param_name = param.to_token_stream().to_string().to_title_case();
        let param_default = if self.default_center {
            quote! { CENTER }
        } else {
            quote! { MIN }
        };
        let param_min = self.min.unwrap_or(0);
        let param_max = self.max.unwrap_or_else(|| {
            abort!(
                self.field,
                "Undefined `max` attribute for {}",
                param.to_token_stream()
            )
        });
        if param_max < param_min {
            abort!(
                self.field,
                "`max` is less then `min` for {}",
                param.to_token_stream()
            );
        }

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct #param(crate::jstation::data::DiscreteValue);

            impl crate::jstation::data::ConstRangeParameter for #param {
                const NAME: &'static str = #param_name;
                const DEFAULT: crate::jstation::data::Normal =
                    crate::jstation::data::Normal::#param_default;
                const MIN_RAW: crate::jstation::data::RawValue =
                    crate::jstation::data::RawValue::new(#param_min);
                const MAX_RAW: crate::jstation::data::RawValue =
                    crate::jstation::data::RawValue::new(#param_max);
                const RANGE: crate::jstation::data::DiscreteRange =
                    crate::jstation::data::DiscreteRange::new(Self::MIN_RAW, Self::MAX_RAW);

                fn from_snapped(normal: crate::jstation::data::Normal) -> Self {
                    #param(crate::jstation::data::DiscreteValue::new(normal, Self::RANGE))
                }

                fn try_from_raw(
                    raw: crate::jstation::data::RawValue,
                ) -> Result<Self, crate::jstation::Error> {
                    let value = crate::jstation::data::DiscreteValue::try_from_raw(
                        raw,
                        Self::RANGE,
                    )
                    .map_err(|err| crate::jstation::Error::with_context(Self::NAME, err))?;

                    Ok(#param(value))
                }
            }

            impl crate::jstation::data::DiscreteParameter for #param {
                fn name(self) -> &'static str {
                    use crate::jstation::data::ConstRangeParameter;
                    Self::NAME
                }

                fn normal_default(self) -> Option<crate::jstation::data::Normal> {
                    use crate::jstation::data::ConstRangeParameter;
                    Some(Self::DEFAULT)
                }

                fn normal(self) -> Option<crate::jstation::data::Normal> {
                    Some(self.0.normal())
                }

                fn to_raw_value(self) -> Option<crate::jstation::data::RawValue> {
                    use crate::jstation::data::ConstRangeParameter;
                    Some(self.0.get_raw(Self::RANGE))
                }

                fn reset(&mut self) -> Option<Self> {
                    use crate::jstation::data::{ConstRangeParameter, ParameterSetter};
                    self.set(Self::from_snapped(Self::DEFAULT))
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
                    use crate::jstation::data::ConstRangeParameter;
                    Self::from_snapped(Self::DEFAULT)
                }
            }
        });

        if let Some(param_nb) = &self.param_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::DiscreteRawParameter for #param {
                    const PARAMETER_NB: crate::jstation::data::ParameterNumber =
                        crate::jstation::data::ParameterNumber::new(#param_nb);
                }
            });
        }

        if let Some(cc_nb) = &self.cc_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::CCParameter for #param {
                    fn to_cc(self) -> Option<crate::midi::CC> {
                        Some(crate::midi::CC::new(
                            crate::midi::CCNumber::new(#cc_nb),
                            self.0.normal().into(),
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
                        use crate::jstation::data::ConstRangeParameter;

                        assert_eq!(cc.nb.as_u8(), #cc_nb);

                        let value =
                            crate::jstation::data::DiscreteValue::new(cc.value.into(), Self::RANGE);
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
                Display::Raw => tokens.extend(quote! {
                    impl std::fmt::Display for #param {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            use crate::jstation::data::DiscreteParameter;
                            std::fmt::Display::fmt(&(self.to_raw_value().unwrap().as_u8()), f)
                        }
                    }
                }),
                Display::Map(name) => {
                    use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

                    let name_str = name.to_string();

                    let name_as_type = name_str.to_upper_camel_case();
                    let names_param_str = &format!("{}{}", param.to_token_stream(), name_as_type);
                    let named_param = Ident::new(names_param_str, self.field.span());

                    let named_list = Ident::new(
                        format!("{}S", &names_param_str.to_shouty_snake_case()).as_str(),
                        self.field.span(),
                    );

                    let name_as_field = name_str.to_snake_case();
                    let name_method = Ident::new(&name_as_field, self.field.span());
                    let names_method =
                        Ident::new(format!("{name_as_field}s").as_str(), self.field.span());

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
                                            (<#param>::MAX_RAW.as_u8() - <#param>::MIN_RAW.as_u8()) as usize + 1,
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
                                use crate::jstation::data::DiscreteParameter;
                                Self::#names_method()[self.to_raw_value().unwrap().as_u8() as usize]
                            }
                        }
                    });
                }
            }
        }
    }
}

enum Display {
    Raw,
    Map(Ident),
}
