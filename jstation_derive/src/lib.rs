use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod boolean;
pub(crate) use boolean::Boolean;

mod const_range;
pub(crate) use const_range::ConstRange;

mod variable_range;
pub(crate) use variable_range::VariableRange;

mod param;
pub(crate) use param::Param;

mod param_group;

#[proc_macro_derive(ParameterSetter, attributes(boolean, const_range, variable_range))]
#[proc_macro_error]
pub fn param_group(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    param_group::derive_struct(&input).into()
}
