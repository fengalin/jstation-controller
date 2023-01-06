use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
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
                input,
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
    fn from_struct(input: &'a DeriveInput, params: impl Iterator<Item = Param<'a>>) -> Self {
        Self {
            name: &input.ident,
            params: Vec::from_iter(params),
        }
    }

    fn variable_range_fields(&self) -> impl Iterator<Item = &Param<'a>> {
        self.params
            .iter()
            .filter(|param| matches!(param, Param::VariableRange(_)))
    }

    fn sorted_by_param_nb(&self) -> Option<impl Iterator<Item = &Param<'a>>> {
        let mut params = Vec::from_iter(self.params.iter().filter(|p| p.param_nb().is_some()));
        if params.is_empty() {
            return None;
        }

        params.sort_by(|p1, p2| p1.param_nb().unwrap().cmp(&p2.param_nb().unwrap()));

        Some(params.into_iter())
    }
}

impl<'a> ToTokens for ParamGroup<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // param structs declarations and traits
        tokens.extend(self.params.iter().map(|p| p.to_token_stream()));

        // ParameterSetter specifics
        let group_name = &self.name;

        tokens.extend({
            let param_enum = self.params.iter().map(Param::ty);
            let param_from = self.params.iter().map(Param::ty);

            quote! {
                #[derive(Clone, Copy, Debug)]
                pub enum Parameter {
                    #( #param_enum(#param_enum), )*
                }

                impl From<Parameter> for crate::jstation::dsp::Parameter {
                    fn from(param: Parameter) -> Self {
                        crate::jstation::dsp::Parameter::#group_name(param)
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

        let mut found_discriminant = false;

        tokens.extend({
            let variant_set_field = self.params.iter().map(|p| {
                let variant = p.ty();
                let field = p.field();

                if p.is_discriminant() {
                    if found_discriminant {
                        abort!(field, "Multiple discriminants for {}", group_name);
                    }
                    found_discriminant = true;

                    let variable_range_field =
                        self.variable_range_fields().map(|param| param.field());

                    quote! {
                        Parameter::#variant(param) => {
                            crate::jstation::data::ParameterSetter::set(
                                &mut self.#field,
                                param,
                            ).map(|param| {
                                use crate::jstation::data::VariableRangeParameter;
                                #( self.#variable_range_field.set_discriminant(param.into()); )*

                                Parameter::#variant(param)
                            })
                        }
                    }
                } else {
                    quote! {
                        Parameter::#variant(param) => {
                            crate::jstation::data::ParameterSetter::set(
                                &mut self.#field,
                                param,
                            ).map(Parameter::#variant)
                        }
                    }
                }
            });

            quote! {
                impl crate::jstation::data::ParameterSetter for #group_name {
                    type Parameter = Parameter;

                    fn set(&mut self, param: Parameter) -> Option<Parameter> {
                        use crate::jstation::data::ParameterSetter;
                        match param {
                            #( #variant_set_field )*
                        }
                    }
                }
            }
        });

        // RawParameter specifics

        if let Some(params) = self.sorted_by_param_nb() {
            let set_field = params.map(|p| {
                let field = p.field();

                let mut set_field = quote! {
                    if let Err(err) = self.#field.set_from(data) {
                        if err.is_inactive_param() {
                            // Don't propagate
                            log::trace!("set_from: {}", err);
                        } else {
                            return Err(err);
                        }
                    }
                };

                if p.is_discriminant() {
                    let variable_range_field =
                        self.variable_range_fields().map(|param| param.field());

                    set_field.extend(quote! {
                        #(
                            crate::jstation::data::VariableRangeParameter::set_discriminant(
                                &mut self.#variable_range_field,
                                self.#field.into(),
                            );
                        )*
                    });
                }

                set_field
            });

            let has_changed_field =
                self.sorted_by_param_nb()
                    .unwrap()
                    .enumerate()
                    .map(|(idx, p)| {
                        let field = p.field();

                        if idx == 0 {
                            quote! {
                                self.#field.has_changed(data)
                            }
                        } else {
                            quote! {
                                || self.#field.has_changed(data)
                            }
                        }
                    });

            tokens.extend(quote! {
                impl crate::jstation::data::ProgramParameter for #group_name {
                    fn set_from(
                        &mut self,
                        data: &crate::jstation::ProgramData,
                    ) -> Result<(), crate::jstation::Error> {
                        #( #set_field )*

                        Ok(())
                    }

                    #[inline]
                    fn has_changed(&self, data: &crate::jstation::ProgramData) -> bool {
                        #( #has_changed_field )*
                    }
                }
            });
        }

        // CCParameter specifics

        tokens.extend({
            // Only implement for params with a declared cc_nb
            let variant_to_cc = self.params.iter().filter_map(|p| {
                p.cc_nb().map(|_| {
                    let variant = p.ty();
                    quote! {
                        Parameter::#variant(param) => param.to_cc(),
                    }
                })
            });

            // Note: discriminant unitity checked with ParameterSetter
            let variant_set_cc = self.params.iter().filter_map(|p| {
                p.cc_nb().map(|cc_nb| {
                    let variant = p.ty();
                    let field = p.field();

                    if p.is_discriminant() {
                        let variable_range_field =
                            self.variable_range_fields().map(|param| param.field());

                        quote! {
                            #cc_nb => {
                                Ok(self.#field.set_cc(cc)?.map(|param| {
                                    use crate::jstation::data::VariableRangeParameter;
                                    #( self.#variable_range_field.set_discriminant(param.into()); )*

                                    Parameter::#variant(param)
                                }))
                            }
                        }
                    } else {
                        quote! {
                            #cc_nb => Ok(self.#field.set_cc(cc)?.map(Parameter::#variant)),
                        }
                    }
                })
            });

            quote! {
                impl crate::jstation::data::CCParameter for Parameter {
                    fn to_cc(self) -> Option<crate::midi::CC> {
                        use crate::jstation::data::CCParameter;
                        match self {
                            #( #variant_to_cc )*
                            _ => None,
                        }
                    }
                }

                impl crate::jstation::data::CCParameterSetter for #group_name {
                    type Parameter = Parameter;

                    fn set_cc(
                        &mut self,
                        cc: crate::midi::CC,
                    ) -> Result<Option<Parameter>, crate::jstation::Error>
                    {
                        use crate::jstation::data::CCParameterSetter;
                        match cc.nb.as_u8() {
                            #( #variant_set_cc )*
                            other => Err(crate::jstation::Error::CCNumberUnknown(other)),
                        }
                    }
                }
            }
        });
    }
}
