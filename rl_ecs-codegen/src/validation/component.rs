use proc_macro2::Span;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};
use syn::{spanned::Spanned, Error, Result, Type};

use crate::parsing;

use crate::TypeId;

pub struct Component {
    pub id: TypeId,
    pub r#type: Type,
    pub children: Vec<Child>,
    pub unique: bool,
}
impl Component {
    pub fn try_into_component_list(
        v: Vec<crate::parsing::component::Component>,
    ) -> Result<HashMap<TypeId, Self>> {
        let mut list = HashMap::with_capacity(v.len());
        let mut duplicate_check_list = HashSet::with_capacity(v.len());

        // Find all the listed components
        for c in &v {
            let children = Vec::with_capacity(c.children.len());
            let id = c.id.unwrap(); //Can this fail? if so, it should be an internal error...

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
                    unique: c.unique,
                },
            );
        }

        // Register all child components and find their component ids
        for comp in v {
            for child in &comp.children {
                let type_id = match Component::search_component(&child.r#type, &list) {
                    Some(id) => id,
                    None => {
                        return Err(Error::new(
                            child.r#type.span(),
                            "Child Type is not registered as a Component.",
                        ));
                    }
                };
                let child_type = child.child_type.into();

                list.get_mut(&type_id)
                    .as_mut()
                    .unwrap()
                    .children
                    .push(Child {
                        type_id,
                        child_type,
                        span: child.r#type.span(),
                    });
            }
        }

        // Check for component -> child cycle
        for c in list.values() {
            c.component_child_cycle_check(c.id, &list, 0)?;
        }

        Ok(list)
    }
    pub fn search_component(typ: &Type, list: &HashMap<TypeId, Self>) -> Option<TypeId> {
        list.iter()
            .find_map(|(k, v)| (*typ == v.r#type).then(|| *k))
    }

    fn component_child_cycle_check(
        &self,
        origin: TypeId,
        all: &HashMap<TypeId, Self>,
        count: usize,
    ) -> Result<()> {
        for child in &self.children {
            const MAX: usize = 1_000;

            if count == MAX {
                return Err(Error::new(
                    child.span,
                    &format!("Component tree too deep ({}).", MAX),
                ));
            }
            if origin == child.type_id {
                return Err(Error::new(child.span, "Component Child Cycle Detected!"));
            }

            let c = all.get(&child.type_id).unwrap();
            c.component_child_cycle_check(origin, all, count + 1)?;
        }
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

pub struct Child {
    pub type_id: TypeId,
    pub child_type: ChildType,
    pub span: Span,
}
