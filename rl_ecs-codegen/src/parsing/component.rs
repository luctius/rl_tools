use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    LitInt, Token, Ident,
};

use crate::TypeId;

#[derive(Debug)]
pub struct Component {
    pub id: Option<TypeId>,
    pub r#type: Ident,
    pub children: Vec<Child>,
}
impl Parse for Component {
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

#[derive(Copy, Clone, Debug)]
pub enum ChildType {
    Single,
    Array(usize),
    Vec,
}

#[derive(Debug)]
pub struct Child {
    pub r#type: Ident,
    pub child_type: ChildType,
}
impl Parse for Child {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(ty) = input.parse() {
            Ok(Self {
                r#type: ty,
                child_type: ChildType::Single,
            })
        } else {
            let child;
            bracketed!(child in input);

            let ty = child.parse()?;

            let child_type = if child.lookahead1().peek(Token![;]) {
                child.parse::<Token![;]>()?;
                let lit: LitInt = child.parse()?;
                let value = lit.base10_parse::<usize>()?;
                ChildType::Array(value)
            } else {
                ChildType::Vec
            };

            Ok(Self {
                r#type: ty,
                child_type,
            })
        }
    }
}
