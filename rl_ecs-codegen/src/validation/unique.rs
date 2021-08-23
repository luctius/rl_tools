use indexmap::IndexMap;
use std::collections::HashSet;
use syn::{Error, Result, Ident};

use super::component::{Child, Component};
use super::{AllComponents, AllUniques, TypeId};

#[derive(Debug)]
pub struct Unique {
    pub id: TypeId,
    pub name: String,
    pub r#type: Ident,
    pub children: Vec<Child>,
}
impl Unique {
    pub fn try_into_unique_list(
        v: Vec<crate::parsing::unique::Unique>,
        all: &AllComponents,
    ) -> Result<AllUniques> {
        let mut list = IndexMap::with_capacity(v.len());
        let mut duplicate_check_list = HashSet::with_capacity(v.len());

        // Find all the listed components
        for c in &v {
            let children = Vec::with_capacity(c.children.len());
            let id = c.id.unwrap(); //Can this fail? if so, it should be an internal error...
            let name = c.r#type.to_string();

            if !duplicate_check_list.insert(c.r#type.clone()) {
                return Err(Error::new(
                    c.r#type.span(),
                    "Duplicate Unique types are not allowed; use the Newtype pattern instead.",
                ));
            }

            list.insert(
                id,
                Self {
                    id,
                    r#type: c.r#type.clone(),
                    children,
                    name,
                },
            );
        }

        // Register all child components and find their component ids
        for comp in v {
            for child in &comp.children {
                let id = match Component::search_component(&child.r#type, all) {
                    Some(id) => id,
                    None => {
                        return Err(Error::new(
                            child.r#type.span(),
                            &format!("{} is not registered as a Component.", &child.r#type),
                        ));
                    }
                };
                let child_type = child.child_type.into();

                list.get_mut(&comp.id.unwrap())
                    .as_mut()
                    .unwrap()
                    .children
                    .push(Child {
                        id,
                        child_type,
                        span: child.r#type.span(),
                    });
            }
        }

        Ok(list)
    }
    pub fn search_component(typ: &Ident, list: &AllComponents) -> Option<TypeId> {
        list.iter()
            .find_map(|(k, v)| (*typ == v.r#type).then(|| *k))
    }
}
