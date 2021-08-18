use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    Error, Ident, Token, Type, Visibility,
};

use crate::TypeId;
use crate::parsing::{component as parsing_component, ParseEcs};

pub mod component;
pub mod query;
pub mod system;
pub mod task;

use component::Component;

pub struct ValidatedEcs {
    pub visibility: Visibility,
    pub name: Ident,
    pub components: HashMap<TypeId, Component>, //Note: we should probably be using a different hash algorithm?
    pub uniques: Vec<TypeId>,
}

impl TryFrom<ParseEcs> for ValidatedEcs {
    type Error = Error;

    fn try_from(pecs: ParseEcs) -> Result<Self, Self::Error> {
        let components = Component::try_into_component_list(pecs.components)?;
        let uniques = pecs.uniques;
        // let mut queries = Vec::new();
        // let mut systems = Vec::new();
        // let mut tasks = Vec::new();

        Ok(Self {
            visibility: pecs.visibility,
            name: pecs.name,
            components,
            uniques,
            // queries,
            // systems,
            // tasks,
        })
    }
}
