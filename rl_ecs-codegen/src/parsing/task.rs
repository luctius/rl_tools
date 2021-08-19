use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    Ident, Token, Type, Visibility,
};

#[derive(Debug)]
pub struct Task {
    name: Ident,
    systems: Vec<Atom>,
}
impl Parse for Task {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut systems = Vec::new();

        let name = input.parse()?;
        input.parse::<Token![=]>()?;

        let task_stream;
        braced!(task_stream in input);

        loop {
            let system = task_stream.parse()?;
            systems.push(system);

            let r = task_stream.parse::<Token![,]>();
            if task_stream.is_empty() {
                break;
            } else if let Err(e) = r {
                return Err(e);
            }
        }

        Ok(Self { name, systems })
    }
}

mod kw {
    syn::custom_keyword!(after);
}

#[derive(Debug)]
pub struct Atom {
    after: Option<Type>,
    system: Type,
}
impl Parse for Atom {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut after = None;
        
        if input.lookahead1().peek(Token![#]) {
            input.parse::<Token![#]>()?;
            
            let option_stream;
            bracketed!(option_stream in input);

            let lookahead = option_stream.lookahead1();
            if lookahead.peek(kw::after) {
                option_stream.parse::<kw::after>()?;
                option_stream.parse::<Token![=]>()?;
                after = Some(option_stream.parse()?);
            }
        }

        let system = input.parse()?;
        Ok(Self {
            after,
            system,
        })
    }
}
