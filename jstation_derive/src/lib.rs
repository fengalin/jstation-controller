use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod boolean;
pub(crate) use boolean::Boolean;

mod discrete;
pub(crate) use discrete::Discrete;

mod param_group;

mod param;
pub(crate) use param::Param;

#[proc_macro_derive(ParamGroup, attributes(boolean, discrete))]
#[proc_macro_error]
pub fn param_group(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    param_group::derive_struct(&input).into()
}
