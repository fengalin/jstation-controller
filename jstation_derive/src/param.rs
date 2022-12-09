use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::ToTokens;
use syn::{
    self,
    parse::{Parse, ParseStream},
    Expr, Field, Ident, Token, Type,
};

use crate::{Boolean, Discrete};

pub enum Param<'a> {
    Discrete(Discrete<'a>),
    Boolean(Boolean<'a>),
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

        let param = if field.attrs[0].path.is_ident("discrete") {
            Param::Discrete(Discrete::from_attrs(field, &field.attrs[0]))
        } else if field.attrs[0].path.is_ident("boolean") {
            Param::Boolean(Boolean::from_attrs(field, &field.attrs[0]))
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

    pub fn name(&self) -> &Type {
        match self {
            Param::Discrete(param) => param.name(),
            Param::Boolean(param) => param.name(),
        }
    }

    pub fn field(&self) -> &Ident {
        match self {
            Param::Discrete(param) => param.field(),
            Param::Boolean(param) => param.field(),
        }
    }

    pub fn cc_nb(&self) -> Option<&Expr> {
        match self {
            Param::Discrete(param) => param.cc_nb(),
            Param::Boolean(param) => param.cc_nb(),
        }
    }
}

impl<'a> ToTokens for Param<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Param::Discrete(param) => param.to_tokens(tokens),
            Param::Boolean(param) => param.to_tokens(tokens),
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
