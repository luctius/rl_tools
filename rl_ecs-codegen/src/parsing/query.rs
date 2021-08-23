use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident, Token,
};

use crate::TypeId;

#[derive(Debug)]
pub struct Query {
    pub cached: bool,
    pub name: Ident,
    pub id: Option<TypeId>,
    pub atoms: Vec<Atom>,
}
impl Parse for Query {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut atoms = Vec::new();

        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let query_stream;
        braced!(query_stream in input);

        let mut parent = Vec::new();

        loop {
            let ident: Ident = query_stream.parse()?;

            atoms.push(Atom {
                name: ident.clone(),
                parent: parent.last().cloned(),
            });

            let lookahead = query_stream.lookahead1();
            if lookahead.peek(Token![<]) {
                query_stream.parse::<Token![<]>()?;

                //TODO: make this unwrap a nice error
                parent.push(ident);
            } else if !parent.is_empty() && lookahead.peek(Token![>]) {
                while !parent.is_empty() {
                    if lookahead.peek(Token![>]) {
                        query_stream.parse::<Token![>]>()?;
                        parent.pop().unwrap();
                    }
                }
                let r = query_stream.parse::<Token![,]>();
                if query_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            } else if lookahead.peek(Token![,]) {
                query_stream.parse::<Token![,]>()?;
            } else {
                return Err(lookahead.error());
            }

            if query_stream.is_empty() {
                break;
            }
        }

        Ok(Self {
            cached: true,
            name,
            atoms,
            id: None,
        })
    }
}

#[derive(Debug)]
pub struct Atom {
    pub name: Ident,
    pub parent: Option<Ident>,
}
