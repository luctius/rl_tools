use proc_macro2::Span;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    BareFnArg, Ident, LitInt, Token, Type, TypeBareFn,
};

use crate::TypeId;

pub enum SystemType {
    ForEach,
    Iterator { stores: Option<Vec<Type>> },
}
impl Default for SystemType {
    fn default() -> Self {
        SystemType::Iterator { stores: None }
    }
}

pub struct System {
    pub func: Type,
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
}

pub struct SystemOption {
    pub r#type: SystemType,
    pub state: Option<Type>,
}
impl Parse for SystemOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stores = Vec::new();
        let mut r#type = SystemType::default();
        let mut state = None;

        if input.lookahead1().peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let option_stream;
            bracketed!(option_stream in input);
            
            let mut has_store = false;
            let mut has_state = false;
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
        Ok(Self { r#type, state })
    }
}
