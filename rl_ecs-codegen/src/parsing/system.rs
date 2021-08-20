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
    Iterator {
        add_stores: Option<Vec<TypePath>>,
        remove_stores: Option<Vec<TypePath>>,
    },
}
impl Default for SystemType {
    fn default() -> Self {
        SystemType::Iterator {
            add_stores: None,
            remove_stores: None,
        }
    }
}

#[derive(Debug)]
pub struct System {
    pub func: Ident,
    pub queries: Vec<Ident>,
    pub r#type: SystemType,
    pub state: Option<Type>,
    pub args: Vec<BareFnArg>,
    pub weight: Weight,
}
impl Parse for System {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut queries = Vec::new();
        let mut args = Vec::new();

        let SystemOption {
            r#type,
            state,
            weight,
        } = input.parse::<SystemOption>()?;
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
            weight,
        })
    }
}

mod kw {
    syn::custom_keyword!(add_stores);
    syn::custom_keyword!(remove_stores);
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
        let mut add_stores = None;
        let mut remove_stores = None;
        let mut r#type = SystemType::default();
        let mut state = None;
        let mut weight = Weight::Medium;

        if input.lookahead1().peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let option_stream;
            bracketed!(option_stream in input);

            let mut has_add_store = false;
            let mut has_remove_store = false;
            let mut has_state = false;
            let mut has_weight = false;
            let mut has_for_each = false;

            loop {
                let lookahead = option_stream.lookahead1();
                if !has_add_store && !has_for_each && lookahead.peek(kw::add_stores) {
                    let mut stores = Vec::new();
                    option_stream.parse::<kw::add_stores>()?;
                    option_stream.parse::<Token![=]>()?;

                    let add_stores_stream;
                    bracketed!(add_stores_stream in option_stream);

                    loop {
                        let store: TypePath = add_stores_stream.parse()?;
                        stores.push(store);
                        if add_stores_stream.is_empty() {
                            break;
                        }
                        add_stores_stream.parse::<Token![,]>()?;
                    }
                    add_stores = Some(stores);
                    has_add_store = true;
                } else if !has_remove_store && !has_for_each && lookahead.peek(kw::remove_stores) {
                    let mut stores = Vec::new();
                    option_stream.parse::<kw::remove_stores>()?;
                    option_stream.parse::<Token![=]>()?;

                    let remove_stores_stream;
                    bracketed!(remove_stores_stream in option_stream);

                    loop {
                        let store: TypePath = remove_stores_stream.parse()?;
                        stores.push(store);
                        if remove_stores_stream.is_empty() {
                            break;
                        }
                        remove_stores_stream.parse::<Token![,]>()?;
                    }
                    remove_stores = Some(stores);
                    has_remove_store = true;
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
                    } else if lookahead.peek(kw::medium) {
                        option_stream.parse::<kw::medium>()?;
                        weight = Weight::Medium;
                    } else if lookahead.peek(kw::high) {
                        option_stream.parse::<kw::high>()?;
                        weight = Weight::High;
                    } else {
                        return Err(lookahead.error());
                    }

                    has_weight = true;
                } else if !has_for_each
                    && !(has_add_store || has_remove_store)
                    && lookahead.peek(kw::for_each)
                {
                    option_stream.parse::<kw::for_each>()?;
                    r#type = SystemType::ForEach;
                    has_for_each = true;
                } else if option_stream.is_empty() {
                    break;
                } else {
                    return Err(lookahead.error());
                }

                let r = option_stream.parse::<Token![,]>();
                if option_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            }

            if has_add_store || has_remove_store {
                r#type = SystemType::Iterator {
                    add_stores,
                    remove_stores,
                };
            }
        }

        Ok(Self {
            r#type,
            state,
            weight,
        })
    }
}
