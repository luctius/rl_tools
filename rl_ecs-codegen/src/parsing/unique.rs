use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident, Token,
};

use super::component::Child;
use crate::TypeId;

#[derive(Debug)]
pub struct Unique {
    pub id: Option<TypeId>,
    pub r#type: Ident,
    pub children: Vec<Child>,
}
impl Parse for Unique {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = Vec::new();

        let ty = input.parse()?;
        if input.parse::<Token![:]>().is_ok() {
            let children_stream;
            braced!(children_stream in input);

            loop {
                let child: Child = children_stream.parse()?;
                children.push(child);

                let r = children_stream.parse::<Token![,]>();
                if children_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            }
        }

        Ok(Self {
            id: None,
            r#type: ty,
            children,
        })
    }
}
