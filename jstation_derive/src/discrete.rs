use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{self, punctuated::Punctuated, Attribute, Expr, Field, Ident, Token, Type};

use crate::param::Arg;

pub struct Discrete<'a> {
    field: &'a Field,
    name: Option<String>,
    default: TokenStream,
    displays: Vec<Display>,
    min: TokenStream,
    max: TokenStream,
    param_nb: Option<Expr>,
    cc_nb: Option<Expr>,
}

impl<'a> Discrete<'a> {
    fn new(field: &'a Field) -> Self {
        Discrete {
            field,
            name: None,
            default: quote! { crate::jstation::data::Normal::MIN },
            displays: Vec::new(),
            min: quote! { crate::jstation::data::RawValue::new(0) },
            max: quote! { crate::jstation::data::RawValue::new(0x7f) },
            param_nb: None,
            cc_nb: None,
        }
    }

    pub fn from_attrs(field: &'a Field, attr: &Attribute) -> Self {
        let mut param = Discrete::new(field);

        let args = attr
            .parse_args_with(Punctuated::<Arg, Token![,]>::parse_terminated)
            .unwrap_or_abort();
        for arg in args {
            let name = arg.name.to_string();
            match name.as_str() {
                "min" => {
                    let value = arg.value_or_abort(field);
                    param.min = quote! { crate::jstation::data::RawValue::new(#value) };
                }
                "max" => {
                    let value = arg.value_or_abort(field);
                    param.max = quote! { crate::jstation::data::RawValue::new(#value) };
                }
                "default_min" => {
                    arg.no_value_or_abort(field);
                    param.default = quote! { crate::jstation::data::Normal::MIN };
                }
                "default_center" => {
                    arg.no_value_or_abort(field);
                    param.default = quote! { crate::jstation::data::Normal::CENTER };
                }
                "default_max" => {
                    arg.no_value_or_abort(field);
                    param.default = quote! { crate::jstation::data::Normal::MAX };
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

                    let name = match path.segments.first() {
                        Some(name) => name,
                        None => {
                            abort!(field, "Empty `display_map` for {}", field.to_token_stream())
                        }
                    };

                    param.displays.push(Display::Map(name.ident.clone()));
                }
                "param_nb" => param.param_nb = Some(arg.value_or_abort(field)),
                "cc_nb" => param.cc_nb = Some(arg.value_or_abort(field)),
                "name" => {
                    let name = match arg.value_or_abort(field) {
                        Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) => lit_str.value(),
                        other => other.to_token_stream().to_string(),
                    };
                    param.name = Some(name);
                }
                other => {
                    abort!(
                        field,
                        "Incompatible arg `{other}` for discrete param {}",
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

    pub fn cc_nb(&self) -> Option<&Expr> {
        self.cc_nb.as_ref()
    }
}

impl<'a> ToTokens for Discrete<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = self.typ();
        let param_name = self.name.clone().unwrap_or_else(|| {
            use heck::ToTitleCase;
            param.to_token_stream().to_string().to_title_case()
        });
        let param_default = &self.default;
        let param_min = &self.min;
        let param_max = &self.max;

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct #param(crate::jstation::data::DiscreteValue);

            impl crate::jstation::data::DiscreteParameter for #param {
                const NAME: &'static str = #param_name;
                const DEFAULT: crate::jstation::data::Normal = #param_default;
                const MIN_RAW: crate::jstation::data::RawValue = #param_min;
                const MAX_RAW: crate::jstation::data::RawValue = #param_max;
                const RANGE: crate::jstation::data::DiscreteRange =
                    crate::jstation::data::DiscreteRange::new(Self::MIN_RAW, Self::MAX_RAW);
            }

            impl crate::jstation::data::ParameterSetter for #param {
                type Parameter = Self;

                fn set(&mut self, param: Self) -> Option<Self::Parameter> {
                    crate::jstation::data::DiscreteParameter::set(self, param)
                }
            }

            impl Default for #param {
                fn default() -> Self {
                    use crate::jstation::data::DiscreteParameter;
                    Self::from_snapped(Self::DEFAULT)
                }
            }

            impl From<crate::jstation::data::DiscreteValue> for #param {
                fn from(value: crate::jstation::data::DiscreteValue) -> Self {
                    #param(value)
                }
            }

            impl From<#param> for crate::jstation::data::DiscreteValue {
                fn from(param: #param) -> Self {
                    param.0
                }
            }

            impl From<#param> for crate::jstation::data::Normal {
                fn from(param: #param) -> Self {
                    param.0.normal()
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
                    fn from_cc(cc: crate::midi::CC) -> Option<Self> {
                        use crate::jstation::data::{DiscreteParameter, DiscreteValue};

                        if cc.nb.as_u8() != #cc_nb {
                            return None;
                        }

                        Some(#param(DiscreteValue::new(
                            cc.value.into(),
                            Self::RANGE,
                        )))
                    }

                    fn to_cc(self) -> Option<crate::midi::CC> {
                        Some(crate::midi::CC::new(
                            crate::midi::CCNumber::new(#cc_nb),
                            self.0.normal().into(),
                        ))
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
                            std::fmt::Display::fmt(&(self.to_raw_value().as_u8()), f)
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
                                        use crate::jstation::data::DiscreteParameter;

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
                                Self::#names_method()[self.to_raw_value().as_u8() as usize]
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
