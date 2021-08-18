#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Type};

use std::collections::HashSet;
use std::convert::TryFrom;

pub(crate) type TypeId = usize;

mod parsing;
mod validation;

use parsing::ParseEcs;
use validation::ValidatedEcs;

#[proc_macro]
pub fn create_ecs(input: TokenStream) -> TokenStream {
    let ecs = match ValidatedEcs::try_from(parse_macro_input!(input as ParseEcs)) {
        Ok(ecs) => ecs,
        Err(error) => return error.to_compile_error().into(),
    };

    let temp = quote! {};
    TokenStream::from(temp)
}
