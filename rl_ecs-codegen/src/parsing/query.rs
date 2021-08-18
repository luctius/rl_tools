use proc_macro2::Span;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    Ident, Token, Type, Visibility,
};

pub struct Query {
    cached: bool,
    name: Ident,
    atoms: Vec<Atom>,
}
impl Parse for Query {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut atoms = Vec::new();

        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let query_stream;
        braced!(query_stream in input);

        loop {
            let atom: Atom = query_stream.parse()?;
            atoms.push(atom);

            let r = query_stream.parse::<Token![,]>();
            if query_stream.is_empty() {
                break;
            } else if let Err(e) = r {
                return Err(e);
            }
        }

        Ok(Self { cached: true, name, atoms, })
    }
}

pub struct Atom {
    pub r#type: Type,
    pub children: Vec<Atom>,
}
impl Parse for Atom {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = Vec::new();

        let ty: Type = input.parse()?;
        
        if input.lookahead1().peek(Token![<]) {
            input.parse::<Token![<]>()?;

            loop {
                let atom: Atom = input.parse()?;
                children.push(atom);

                if input.lookahead1().peek(Token![>]) {
                    input.parse::<Token![>]>()?;
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            r#type: ty,
            children,
        })
    }
}
