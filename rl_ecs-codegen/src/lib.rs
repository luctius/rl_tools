#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

use std::convert::TryFrom;

pub(crate) type TypeId = usize;

mod codegen;
mod parsing;
mod validation;

use parsing::ParseEcs;
use validation::ValidatedEcs;

#[proc_macro]
pub fn create_ecs(input: TokenStream) -> TokenStream {
    let pecs = parse_macro_input!(input as ParseEcs);
    // dbg!(&pecs);

    let ecs = match ValidatedEcs::try_from(pecs) {
        Ok(ecs) => ecs,
        Err(error) => return error.to_compile_error().into(),
    };

    TokenStream::from(ecs)
}
