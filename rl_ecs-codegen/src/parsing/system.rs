use proc_macro2::Span;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    BareFnArg, Ident, LitInt, Token, Type, TypeBareFn, TypePath,
};

use crate::TypeId;

#[derive(Debug)]
pub enum SystemType {
    ForEach,
    Iterator { stores: Option<Vec<TypePath>> },
}
impl Default for SystemType {
    fn default() -> Self {
        SystemType::Iterator { stores: None }
    }
}

#[derive(Debug)]
pub struct System {
    pub func: Ident,
    pub queries: Vec<Ident>,
    pub r#type: SystemType,
    pub state: Option<Type>,
    pub args: Vec<BareFnArg>,
}
impl Parse for System {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut queries = Vec::new();
        let mut args = Vec::new();

        let (r#type, state) = if let Ok(options) = input.parse::<SystemOption>() {
            (options.r#type, options.state)
        } else {
            (SystemType::default(), None)
        };
        let func = input.parse()?;

        if input.lookahead1().peek(Token![:]) {
            input.parse::<Token![:]>()?;
            input.parse::<Token![<]>()?;
            loop {
                let ident = input.parse()?;
                queries.push(ident);

                if input.lookahead1().peek(Token![>]) {
                    input.parse::<Token![>]>()?;
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }

        input.parse::<Token![=]>()?;

        let argument_stream;
        braced!(argument_stream in input);
        loop {
            let arg = argument_stream.parse()?;
            args.push(arg);

            if argument_stream.lookahead1().peek(Token![,]) {
                argument_stream.parse::<Token![,]>()?;
                continue;
            }

            break;
        }

        Ok(Self {
            func,
            r#type,
            state,
            queries,
            args,
        })
    }
}

mod kw {
    syn::custom_keyword!(stores);
    syn::custom_keyword!(state);
    syn::custom_keyword!(for_each);
    syn::custom_keyword!(weight);
    syn::custom_keyword!(low);
    syn::custom_keyword!(medium);
    syn::custom_keyword!(high);
}

#[derive(Debug, Copy, Clone)]
pub enum Weight {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
pub struct SystemOption {
    pub r#type: SystemType,
    pub state: Option<Type>,
    pub weight: Weight,
}
impl Parse for SystemOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stores = Vec::new();
        let mut r#type = SystemType::default();
        let mut state = None;
        let mut weight = Weight::Medium;

        if input.lookahead1().peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let option_stream;
            bracketed!(option_stream in input);

            let mut has_store = false;
            let mut has_state = false;
            let mut has_weight = false;
            let mut has_for_each = false;

            loop {
                let lookahead = option_stream.lookahead1();
                if !has_store && !has_for_each && lookahead.peek(kw::stores) {
                    option_stream.parse::<kw::stores>()?;
                    option_stream.parse::<Token![=]>()?;

                    let stores_stream;
                    bracketed!(stores_stream in option_stream);

                    loop {
                        let store: Type = stores_stream.parse()?;
                        stores.push(store);
                        if stores_stream.is_empty() {
                            break;
                        }
                        stores_stream.parse::<Token![,]>()?;
                    }
                    has_store = true;
                } else if !has_state && lookahead.peek(kw::state) {
                    option_stream.parse::<kw::state>()?;
                    option_stream.parse::<Token![:]>()?;
                    state = Some(option_stream.parse()?);
                    has_state = true;
                } else if !has_weight && lookahead.peek(kw::weight) {
                    option_stream.parse::<kw::weight>()?;
                    option_stream.parse::<Token![=]>()?;

                    let lookahead = option_stream.lookahead1();
                    if lookahead.peek(kw::low) {
                        option_stream.parse::<kw::low>()?;
                        weight = Weight::Low;
                    }
                    else if lookahead.peek(kw::medium) {
                        option_stream.parse::<kw::medium>()?;
                        weight = Weight::Medium;
                    }
                    else if lookahead.peek(kw::high) {
                        option_stream.parse::<kw::high>()?;
                        weight = Weight::High;
                    } else {
                        return Err(lookahead.error());
                    }

                    has_weight = true;
                } else if !has_for_each && !has_store && lookahead.peek(kw::for_each) {
                    option_stream.parse::<kw::for_each>()?;
                    r#type = SystemType::ForEach;
                    has_for_each = true;
                } else if option_stream.is_empty() {
                    break;
                } else {
                    return Err(lookahead.error());
                }
                option_stream.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            r#type,
            state,
            weight,
        })
    }
}
