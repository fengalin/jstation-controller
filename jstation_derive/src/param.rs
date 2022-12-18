use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::ToTokens;
use syn::{
    self,
    parse::{Parse, ParseStream},
    Expr, Field, Ident, Token, Type,
};

use crate::{Boolean, ConstRange, VariableRange};

pub enum Param<'a> {
    ConstRange(ConstRange<'a>),
    Boolean(Boolean<'a>),
    VariableRange(VariableRange<'a>),
}

impl<'a> Param<'a> {
    pub fn from_param_field(field: &'a Field) -> Option<Self> {
        if field.attrs.is_empty() {
            return None;
        }

        if field.attrs.len() > 1 {
            abort!(
                field,
                "Expected only one param attribute for {}, found {}",
                field.ty.to_token_stream(),
                field.attrs.len()
            );
        }

        let param = if field.attrs[0].path.is_ident("const_range") {
            Param::ConstRange(ConstRange::from_attrs(field, &field.attrs[0]))
        } else if field.attrs[0].path.is_ident("boolean") {
            Param::Boolean(Boolean::from_attrs(field, &field.attrs[0]))
        } else if field.attrs[0].path.is_ident("variable_range") {
            Param::VariableRange(VariableRange::from_attrs(field, &field.attrs[0]))
        } else {
            abort!(
                field,
                "Unknown param attribute {} for {}",
                field.attrs[0].path.to_token_stream(),
                field.ty.to_token_stream(),
            );
        };

        Some(param)
    }

    pub fn typ(&self) -> &Type {
        match self {
            Param::ConstRange(param) => param.typ(),
            Param::Boolean(param) => param.typ(),
            Param::VariableRange(param) => param.typ(),
        }
    }

    pub fn field(&self) -> &Ident {
        match self {
            Param::ConstRange(param) => param.field(),
            Param::Boolean(param) => param.field(),
            Param::VariableRange(param) => param.field(),
        }
    }

    pub fn cc_nb(&self) -> Option<u8> {
        match self {
            Param::ConstRange(param) => param.cc_nb(),
            Param::Boolean(param) => param.cc_nb(),
            Param::VariableRange(param) => param.cc_nb(),
        }
    }

    pub fn is_discriminant(&self) -> bool {
        match self {
            Param::ConstRange(param) => param.is_discriminant(),
            _ => false,
        }
    }

    pub fn is_variable_range(&self) -> bool {
        matches!(self, Param::VariableRange(_))
    }
}

impl<'a> ToTokens for Param<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Param::ConstRange(param) => param.to_tokens(tokens),
            Param::Boolean(param) => param.to_tokens(tokens),
            Param::VariableRange(param) => param.to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
pub struct Arg {
    pub name: Ident,
    pub value: Option<Expr>,
}

impl Arg {
    pub fn value_or_abort(&self, field: &Field) -> Expr {
        self.value
            .as_ref()
            .cloned()
            .unwrap_or_else(|| abort!(field, "attribute `{}` requires a value", self.name))
    }

    pub fn no_value_or_abort(&self, field: &Field) {
        if self.value.is_some() {
            abort!(field, "attribute `{}` doesn't accept any value", self.name);
        }
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        let value = if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign_token = input.parse::<Token![=]>()?; // skip '='
            let value = input.parse::<Expr>().ok();
            if value.is_none() {
                abort! {
                    assign_token,
                    "expected `string literal` or `expression` after `=`"
                }
            }

            value
        } else {
            None
        };

        Ok(Self { name, value })
    }
}
