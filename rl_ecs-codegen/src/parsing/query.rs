use proc_macro2::Span;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    Ident, Token, TypePath, Visibility,
};

#[derive(Debug)]
pub struct Query {
    cached: bool,
    name: Ident,
    atoms: Vec<TypePath>,
}
impl Parse for Query {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut atoms = Vec::new();

        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let query_stream;
        braced!(query_stream in input);

        loop {
            let atom = query_stream.parse()?;
            atoms.push(atom);

            let r = query_stream.parse::<Token![,]>();
            if query_stream.is_empty() {
                break;
            } else if let Err(e) = r {
                return Err(e);
            }
        }

        Ok(Self {
            cached: true,
            name,
            atoms,
        })
    }
}
