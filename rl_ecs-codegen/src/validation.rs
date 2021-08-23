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
use query::Query;

pub type AllComponents = IndexMap<TypeId, Component>;
pub type AllUniques = IndexMap<TypeId, Unique>;
pub type AllQueries = IndexMap<TypeId, Query>;

#[derive(Debug)]
pub struct ValidatedEcs {
    pub visibility: Visibility,
    pub name: Ident,
    pub components: AllComponents, //Note: we should probably be using a different hash algorithm?
    pub uniques: AllUniques,
    pub queries: AllQueries,
}

impl TryFrom<ParseEcs> for ValidatedEcs {
    type Error = Error;

    fn try_from(pecs: ParseEcs) -> Result<Self, Self::Error> {
        let components = Component::try_into_component_list(pecs.components)?;
        let uniques = Unique::try_into_unique_list(pecs.uniques, &components)?;
        let queries = Query::try_into_query_list(pecs.queries, &components)?;
        // let mut systems = Vec::new();
        // let mut tasks = Vec::new();

        Ok(Self {
            visibility: pecs.visibility,
            name: pecs.name,
            components,
            uniques,
            queries,
            // systems,
            // tasks,
        })
    }
}
