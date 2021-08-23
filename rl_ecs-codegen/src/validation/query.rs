use indexmap::IndexMap;
use std::collections::HashSet;
use syn::{Error, Ident, Result};

use crate::{
    validation::{component::Component, AllComponents, AllQueries},
    TypeId,
};

#[derive(Debug)]
pub struct Query {
    pub cached: bool,
    pub name: Ident,
    pub id: TypeId,
    pub children: Vec<Atom>,
}
impl Query {
    pub fn try_into_query_list(
        v: Vec<crate::parsing::query::Query>,
        components: &AllComponents,
    ) -> Result<AllQueries> {
        let mut list = IndexMap::new();

        for q in &v {
            let cached = true;
            let id = q.id.unwrap();
            let name = q.name.clone();
            let mut children = Vec::new();
            let mut duplicate_check_list = HashSet::with_capacity(q.atoms.len());

            for a in &q.atoms {
                let id = match Component::search_component(&a.name, components) {
                    Some(id) => id,
                    None => {
                        return Err(Error::new(
                            name.span(),
                            &format!("{} is not registered as a Component.", a.name),
                        ));
                    }
                };
                let parent = if let Some(p) = &a.parent {
                    match Component::search_component(p, components) {
                        Some(id) => Some(id),
                        None => {
                            return Err(Error::new(
                                name.span(),
                                &format!("{} is not registered as a Component.", p),
                            ));
                        }
                    }
                } else {
                    None
                };

                // Check for duplicates
                if !duplicate_check_list.insert(id) {
                    return Err(Error::new(
                        a.name.span(),
                        "Duplicate Component types are not allowed in a query.",
                    ));
                }

                // Check for child <-> parent validity
                if let Some(parent) = parent {
                    let parent = components.get(&parent).unwrap();
                    if !parent.children.iter().any(|c| c.id == id) {
                        return Err(Error::new(
                            a.name.span(),
                            &format!("{} does not have {} as a child.", parent.name, a.name),
                        ));
                    }
                }

                children.push(Atom { id, parent });
            }

            list.insert(
                id,
                Query {
                    cached,
                    name,
                    id,
                    children,
                },
            );
        }

        Ok(list)
    }
}

#[derive(Debug)]
pub struct Atom {
    pub id: TypeId,
    pub parent: Option<TypeId>,
}
