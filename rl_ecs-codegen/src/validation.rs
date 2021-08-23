use indexmap::IndexMap;
use std::{
    convert::TryFrom,
};
use syn::{
    Error, Ident, Visibility,
};

use crate::parsing::ParseEcs;
use crate::TypeId;

pub mod component;
pub mod query;
pub mod system;
pub mod task;
pub mod unique;

use component::Component;
use unique::Unique;

pub type AllComponents = IndexMap<TypeId, Component>;
pub type AllUniques = IndexMap<TypeId, Unique>;

#[derive(Debug)]
pub struct ValidatedEcs {
    pub visibility: Visibility,
    pub name: Ident,
    pub components: AllComponents, //Note: we should probably be using a different hash algorithm?
    pub uniques: AllUniques,
}

impl TryFrom<ParseEcs> for ValidatedEcs {
    type Error = Error;

    fn try_from(pecs: ParseEcs) -> Result<Self, Self::Error> {
        let components = Component::try_into_component_list(pecs.components)?;
        let uniques = Unique::try_into_unique_list(pecs.uniques, &components)?;
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
