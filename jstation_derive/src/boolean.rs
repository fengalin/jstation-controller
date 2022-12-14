use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{self, punctuated::Punctuated, Attribute, Expr, Field, Ident, Token, Type};

use crate::param::Arg;

pub struct Boolean<'a> {
    field: &'a Field,
    name: Option<String>,
    default: TokenStream,
    display_raw: bool,
    param_nb: Option<Expr>,
    cc_nb: Option<Expr>,
}

impl<'a> Boolean<'a> {
    fn new(field: &'a Field) -> Self {
        Boolean {
            field,
            name: None,
            default: quote! { false },
            display_raw: false,
            param_nb: None,
            cc_nb: None,
        }
    }

    pub fn from_attrs(field: &'a Field, attr: &Attribute) -> Self {
        let mut param = Boolean::new(field);

        let args = attr
            .parse_args_with(Punctuated::<Arg, Token![,]>::parse_terminated)
            .unwrap_or_abort();
        for arg in args {
            let name = arg.name.to_string();
            match name.as_str() {
                "default_inactive" => {
                    arg.no_value_or_abort(field);
                    param.default = quote! { false };
                }
                "default_active" => {
                    arg.no_value_or_abort(field);
                    param.default = quote! { true };
                }
                "display_raw" => {
                    arg.no_value_or_abort(field);
                    param.display_raw = true;
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
                        "Incompatible arg `{other}` for boolean param {}",
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
        self.field.ident.as_ref().expect("named field")
    }

    pub fn cc_nb(&self) -> Option<&Expr> {
        self.cc_nb.as_ref()
    }
}

impl<'a> ToTokens for Boolean<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = self.typ();
        let param_name = self.name.clone().unwrap_or_else(|| {
            use heck::ToTitleCase;
            param.to_token_stream().to_string().to_title_case()
        });
        let param_default = &self.default;

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct #param(bool);

            impl crate::jstation::data::BoolParameter for #param {
                const NAME: &'static str = #param_name;
                const DEFAULT: bool = #param_default;
            }

            impl crate::jstation::data::ParameterSetter for #param {
                type Parameter = Self;

                fn set(&mut self, param: Self) -> Option<Self::Parameter> {
                    crate::jstation::data::BoolParameter::set(self, param)
                }
            }

            impl Default for #param {
                fn default() -> Self {
                    use crate::jstation::data::BoolParameter;
                    #param(Self::DEFAULT)
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

        if let Some(param_nb) = &self.param_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::BoolRawParameter for #param {
                    const PARAMETER_NB: crate::jstation::data::ParameterNumber =
                        crate::jstation::data::ParameterNumber::new(#param_nb);
                }
            });
        }

        if let Some(cc_nb) = &self.cc_nb {
            tokens.extend(quote! {
                impl crate::jstation::data::CCParameter for #param {
                    fn from_cc(cc: crate::midi::CC) -> Option<Self> {
                        const CC_TRUE_THRESHOLD: u8 = 0x40;

                        if cc.nb.as_u8() != #cc_nb {
                            return None;
                        }

                        Some(#param((cc.value.as_u8() >= CC_TRUE_THRESHOLD)))
                    }

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
            });
        }

        if self.display_raw {
            tokens.extend(quote! {
                impl std::fmt::Display for #param {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        use crate::jstation::data::BoolParameter;
                        if self.is_active() {
                            f.write_str("on")
                        } else {
                            f.write_str("off")
                        }
                    }
                }
            });
        }
    }
}
