use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, Fields, Ident};

use crate::Param;

pub fn derive_struct(input: &DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let param_group = ParamGroup::from_struct(
                &input.ident,
                fields.named.iter().filter_map(Param::from_param_field),
            );

            param_group.into_token_stream()
        }
        _ => abort_call_site!("`#[derive(ParamGroup)]` only supports structs with named fields"),
    }
}

pub struct ParamGroup<'a> {
    name: &'a Ident,
    params: Vec<Param<'a>>,
}

impl<'a> ParamGroup<'a> {
    fn from_struct(name: &'a Ident, params: impl Iterator<Item = Param<'a>>) -> Self {
        Self {
            name,
            params: Vec::from_iter(params),
        }
    }
}

impl<'a> ToTokens for ParamGroup<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // param struct declarations and traits
        tokens.extend(self.params.iter().map(|p| p.to_token_stream()));

        // ParameterGroup specifics
        let group_name = &self.name;

        tokens.extend({
            let param_enum = self.params.iter().map(Param::typ);
            let param_from = self.params.iter().map(Param::typ);

            quote! {
                #[derive(Clone, Copy, Debug)]
                pub enum Parameter {
                    #( #param_enum(#param_enum), )*
                }

                impl From<Parameter> for crate::jstation::Parameter {
                    fn from(param: Parameter) -> Self {
                        crate::jstation::Parameter::#group_name(param)
                    }
                }

                #(
                    impl From<#param_from> for Parameter {
                        fn from(param: #param_from) -> Self {
                            Parameter::#param_from(param)
                        }
                    }

                    impl From<&#param_from> for Parameter {
                        fn from(param: &#param_from) -> Self {
                            Parameter::#param_from(*param)
                        }
                    }

                    impl From<&mut #param_from> for Parameter {
                        fn from(param: &mut #param_from) -> Self {
                            Parameter::#param_from(*param)
                        }
                    }
                )*
            }
        });

        tokens.extend({
            let variant_set_field = self.params.iter().map(|p| {
                let variant = p.typ();
                let field = p.field();

                quote! {
                    Parameter::#variant(param) => {
                        self.#field.set(param).map(Parameter::#variant)
                    }
                }
            });

            quote! {
                impl crate::jstation::data::ParameterSetter for #group_name {
                    type Parameter = Parameter;

                    fn set(&mut self, param: Self::Parameter) -> Option<Self::Parameter> {
                        use crate::jstation::data::ParameterSetter;
                        match param {
                            #( #variant_set_field )*
                        }
                    }
                }
            }
        });

        // CCParameter specifics

        tokens.extend({
            // Only implement for params with a declared cc_nb
            let variant_from_cc = self.params.iter().filter_map(|p| {
                p.cc_nb().map(|cc_nb| {
                    let param = p.typ();
                    quote! {
                        #cc_nb => #param::from_cc(cc).map(Parameter::#param),
                    }
                })
            });

            let variant_to_cc = self.params.iter().filter_map(|p| {
                p.cc_nb().map(|_| {
                    let variant = p.typ();
                    quote! {
                        Parameter::#variant(param) => param.to_cc(),
                    }
                })
            });

            quote! {
                impl crate::jstation::data::CCParameter for Parameter {
                    fn from_cc(cc: crate::midi::CC) -> Option<Self> {
                        use crate::jstation::data::CCParameter;
                        match cc.nb.as_u8() {
                            #( #variant_from_cc )*
                            _ => None,
                        }
                    }

                    fn to_cc(self) -> Option<crate::midi::CC> {
                        use crate::jstation::data::CCParameter;
                        match self {
                            #( #variant_to_cc )*
                            _ => None,
                        }
                    }
                }
            }
        });
    }
}
