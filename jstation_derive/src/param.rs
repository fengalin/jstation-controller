use heck::ToTitleCase;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::ToTokens;
use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Field, Ident, Token, Type,
};

use crate::{Boolean, ConstRange, VariableRange};

pub(crate) enum Param<'a> {
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

        let attr = &field.attrs[0];
        let args = attr
            .parse_args_with(Punctuated::<Arg, Token![,]>::parse_terminated)
            .unwrap_or_abort()
            .into_iter();

        let param = ParamBase::new(field);

        if attr.path.is_ident("const_range") {
            Some(Param::ConstRange(ConstRange::new(param, args)))
        } else if attr.path.is_ident("boolean") {
            Some(Param::Boolean(Boolean::new(param, args)))
        } else if attr.path.is_ident("variable_range") {
            Some(Param::VariableRange(VariableRange::new(param, args)))
        } else {
            abort!(
                field,
                "Unknown param attribute {} for {}",
                attr.path.to_token_stream(),
                param.field.ty.to_token_stream(),
            );
        }
    }

    fn base(&self) -> &ParamBase {
        match self {
            Param::ConstRange(param) => &param.base,
            Param::Boolean(param) => &param.base,
            Param::VariableRange(param) => &param.base,
        }
    }

    pub fn ty(&self) -> &Type {
        &self.base().field.ty
    }

    pub fn field(&self) -> &Ident {
        self.base().field.ident.as_ref().expect("named field")
    }

    pub fn cc_nb(&self) -> Option<u8> {
        self.base().cc_nb
    }

    pub fn param_nb(&self) -> Option<u8> {
        self.base().param_nb
    }

    pub fn is_discriminant(&self) -> bool {
        match self {
            Param::ConstRange(param) => param.is_discriminant(),
            _ => false,
        }
    }
}

pub struct ParamBase<'a> {
    pub field: &'a Field,
    pub name: String,
    pub param_nb: Option<u8>,
    pub cc_nb: Option<u8>,
}

impl<'a> ParamBase<'a> {
    pub fn new(field: &'a Field) -> Self {
        ParamBase {
            field,
            name: field.ty.to_token_stream().to_string().to_title_case(),
            param_nb: None,
            cc_nb: None,
        }
    }

    pub fn have_arg(&mut self, arg: Arg) {
        let name = arg.name.to_string();
        match name.as_str() {
            "param_nb" => self.param_nb = Some(arg.u8_or_abort(self.field)),
            "cc_nb" => self.cc_nb = Some(arg.u8_or_abort(self.field)),
            other => {
                abort!(
                    self.field,
                    "Incompatible arg `{}` for param {}",
                    other,
                    self.field.ty.to_token_stream(),
                );
            }
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
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

    pub fn u8_or_abort(&self, field: &Field) -> u8 {
        let value = self
            .value
            .as_ref()
            .unwrap_or_else(|| abort!(field, "attribute `{}` requires a value", self.name));

        match value {
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) => lit_int.base10_parse::<u8>().unwrap_or_else(|err| {
                abort!(
                    field,
                    "Expected a literal `u8` for `{}`: {:?}",
                    self.name,
                    err
                );
            }),
            other => {
                abort!(
                    field,
                    "Expected a literal `u8` for `{}` found {}",
                    self.name,
                    other.to_token_stream(),
                )
            }
        }
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
