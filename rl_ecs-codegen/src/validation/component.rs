use indexmap::IndexMap;
use proc_macro2::Span;
use std::collections::HashSet;
use syn::{Error, Ident, Result};

use super::{AllComponents, TypeId};
use crate::parsing;

#[derive(Debug)]
pub struct Component {
    pub id: TypeId,
    pub name: String,
    pub r#type: Ident,
    pub children: Vec<Child>,
}
impl Component {
    pub fn try_into_component_list(
        v: Vec<crate::parsing::component::Component>,
    ) -> Result<AllComponents> {
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
                    "Duplicate Component types are not allowed.",
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
                let id = match Component::search_component(&child.r#type, &list) {
                    Some(id) => id,
                    None => {
                        return Err(Error::new(
                            child.r#type.span(),
                            &format!("{} is not registered as a Component.", &child.r#type),
                        ));
                    }
                };
                let child_type = child.child_type.into();
                if let ChildType::Array(sz) = child_type {
                    if sz == 0 {
                        return Err(Error::new(
                            child.r#type.span(),
                            "size 0 is unusable, thus not allowed.",
                        ));
                    } else if sz == 1 {
                        let name = &list.get(&id).unwrap().name;
                        child
                            .r#type
                            .span()
                            .unwrap()
                            .warning(&format!(
                                "using [{};1] instead of {} is discouraged.",
                                name, name
                            ))
                            .emit();
                    } else if sz > 10 {
                        let name = &list.get(&id).unwrap().name;
                        child
                            .r#type
                            .span()
                            .unwrap()
                            .note(&format!(
                                "use [{};{}] instead of [{}] is not recommended for large arrays.",
                                name, sz, name
                            ))
                            .emit();
                    }
                }

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

        // Check for component -> child cycle
        let mut name_trace = Vec::new();
        let mut id_trace = Vec::new();
        for c in list.values() {
            name_trace.push(c.name.as_str());
            id_trace.push(c.id);
            c.component_child_cycle_check(&list, &mut name_trace, &mut id_trace)?;
            name_trace.pop();
            id_trace.pop();
        }

        Ok(list)
    }
    pub fn search_component(typ: &Ident, list: &AllComponents) -> Option<TypeId> {
        list.iter()
            .find_map(|(k, v)| (*typ == v.r#type).then(|| *k))
    }

    // Visit all children of self, recusively and bail if we find a duplicate in the trace
    fn component_child_cycle_check<'a>(
        &self,
        all: &'a AllComponents,
        name_trace: &mut Vec<&'a str>,
        id_trace: &mut Vec<TypeId>,
    ) -> Result<()> {
        // Unrealistic amount of tree branches
        const MAX: usize = 1_000;

        for child in &self.children {
            let c = all.get(&child.id).unwrap();
            name_trace.push(&c.name);

            // Realistically, this should not happen,
            // Cycles should have been caught before this,
            // and nobody is crazy enough to create 1000 elements and link them all.
            // .... right??
            if name_trace.len() == MAX {
                return Err(Error::new(
                    child.span,
                    &format!(
                        "Component tree too deep (trace[{}]: {:?}).",
                        name_trace.len(),
                        name_trace
                    ),
                ));
            }

            for tid in id_trace.iter() {
                if *tid == child.id {
                    return Err(Error::new(
                        child.span,
                        &format!("Component Cycle Detected! (trace: {:?})", name_trace),
                    ));
                }
            }

            id_trace.push(c.id);
            c.component_child_cycle_check(all, name_trace, id_trace)?;
        }
        name_trace.pop();
        id_trace.pop();
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ChildType {
    Single,
    Array(usize),
    Vec,
}
impl From<parsing::component::ChildType> for ChildType {
    fn from(ct: parsing::component::ChildType) -> Self {
        match ct {
            parsing::component::ChildType::Single => ChildType::Single,
            parsing::component::ChildType::Array(sz) => ChildType::Array(sz),
            parsing::component::ChildType::Vec => ChildType::Vec,
        }
    }
}

#[derive(Debug)]
pub struct Child {
    pub id: TypeId,
    pub child_type: ChildType,
    pub span: Span,
}
