use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{self, punctuated::Punctuated, Attribute, Expr, Field, Ident, Token, Type};

use crate::param::Arg;

// FIXME factorize the common fields amongst all parameters
pub struct VariableRange<'a> {
    field: &'a Field,
    name: Option<String>,
    param_nb: Option<u8>,
    cc_nb: Option<u8>,
}

impl<'a> VariableRange<'a> {
    fn new(field: &'a Field) -> Self {
        VariableRange {
            field,
            name: None,
            param_nb: None,
            cc_nb: None,
        }
    }

    pub fn from_attrs(field: &'a Field, attr: &Attribute) -> Self {
        let mut param = VariableRange::new(field);

        let args = attr
            .parse_args_with(Punctuated::<Arg, Token![,]>::parse_terminated)
            .unwrap_or_abort();
        for arg in args {
            let name = arg.name.to_string();
            match name.as_str() {
                "param_nb" => param.param_nb = Some(arg.u8_or_abort(field)),
                "cc_nb" => param.cc_nb = Some(arg.u8_or_abort(field)),
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
                        "Incompatible arg `{}` for `variable_range` param {}",
                        other,
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
}

impl<'a> ToTokens for VariableRange<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param = self.typ();
        let param_name = self.name.clone().unwrap_or_else(|| {
            use heck::ToTitleCase;
            param.to_token_stream().to_string().to_title_case()
        });

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Default, PartialEq)]
            pub struct #param {
                discr: <Self as crate::jstation::data::VariableRange>::Discriminant,
                value: crate::jstation::data::DiscreteValue,
            }

            impl crate::jstation::data::VariableRangeParameter for #param {
                const NAME: &'static str = #param_name;

                fn range(self) -> Option<DiscreteRange> {
                    <Self as crate::jstation::data::VariableRange>::range_from(self.discr)
                }

                fn set_discriminant(&mut self, discr: Self::Discriminant) {
                    if self.discr != discr {
                        self.discr = discr;

                        // snap the value to the new range
                        if let Some(range) = <Self as crate::jstation::data::VariableRange>::range_from(discr) {
                            self.value = crate::jstation::data::DiscreteValue::new(
                                self.value.normal(),
                                range,
                            );
                        }
                    }
                }

                fn from_snapped(
                    discr: <Self as crate::jstation::data::VariableRange>::Discriminant,
                    normal: crate::jstation::data::Normal,
                ) -> Option<Self> {
                    let range = <Self as crate::jstation::data::VariableRange>::range_from(discr)?;
                    let value = crate::jstation::data::DiscreteValue::new(normal, range);

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
                    let value =
                        crate::jstation::data::DiscreteValue::try_from_raw(raw, range)
                            .map_err(|err| Error::with_context(Self::NAME, err))?;

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
                        use crate::jstation::data::DiscreteParameter;

                        let normal = self.normal()?;
                        Some(crate::midi::CC::new(
                            crate::midi::CCNumber::new(#cc_nb),
                            normal.into(),
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
                        use crate::jstation::{data::{DiscreteParameter, VariableRangeParameter}, Error};

                        assert_eq!(cc.nb.as_u8(), #cc_nb);

                        let range = self.range()
                            .ok_or_else(|| Error::ParameterInactive(stringify!(#param).into()))?;
                        let value =
                            crate::jstation::data::DiscreteValue::new(cc.value.into(), range);
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
