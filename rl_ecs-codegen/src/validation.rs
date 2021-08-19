use indexmap::IndexMap;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    Error, Ident, Token, Type, Visibility,
};

use crate::parsing::{component as parsing_component, ParseEcs};
use crate::TypeId;

pub mod component;
pub mod query;
pub mod system;
pub mod task;

use component::Component;

pub type AllComponents = IndexMap<TypeId, Component>;

#[derive(Debug)]
pub struct ValidatedEcs {
    pub visibility: Visibility,
    pub name: Ident,
    pub components: AllComponents, //Note: we should probably be using a different hash algorithm?
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
