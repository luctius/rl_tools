use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident, Token, Visibility,
};

pub mod component;
pub mod query;
pub mod system;
pub mod task;
pub mod unique;

use component::Component;
use query::Query;
use system::System;
use task::Task;
use unique::Unique;

mod kw {
    syn::custom_keyword!(world);
    syn::custom_keyword!(components);
    syn::custom_keyword!(uniques);
    syn::custom_keyword!(queries);
    syn::custom_keyword!(systems);
    syn::custom_keyword!(tasklists);
}

#[derive(Debug)]
pub struct ParseEcs {
    pub visibility: Visibility,
    pub name: Ident,
    pub components: Vec<Component>,
    pub uniques: Vec<Unique>,
    pub queries: Vec<Query>,
    pub systems: Vec<System>,
    pub tasks: Vec<Task>,
}
impl Parse for ParseEcs {
    #[allow(clippy::too_many_lines)]
    fn parse(input: ParseStream) -> Result<Self> {
        let mut components = Vec::new();
        let mut uniques = Vec::new();
        let mut queries = Vec::new();
        let mut systems = Vec::new();
        let mut tasks = Vec::new();

        // Parse Ecs Info
        let visibility: Visibility = input.parse()?;
        input.parse::<kw::world>()?;
        let name: Ident = input.parse()?;

        let ecs;
        braced!(ecs in input);

        // Parse Components
        ecs.parse::<kw::components>()?;
        ecs.parse::<Token![:]>()?;
        let component_stream;
        braced!(component_stream in ecs);

        let mut id_counter = 0;
        loop {
            let mut comp: Component = component_stream.parse()?;
            comp.id = Some(id_counter);
            components.push(comp);

            id_counter += 1;

            let r = component_stream.parse::<Token![,]>();
            if component_stream.is_empty() {
                break;
            } else if let Err(e) = r {
                return Err(e);
            }
        }
        ecs.parse::<Token![,]>()?;

        // Parse Uniques
        if ecs.lookahead1().peek(kw::uniques) {
            ecs.parse::<kw::uniques>()?;
            ecs.parse::<Token![:]>()?;
            let unique_stream;
            braced!(unique_stream in ecs);

            loop {
                let mut comp: Unique = unique_stream.parse()?;
                comp.id = Some(id_counter);
                uniques.push(comp);

                id_counter += 1;

                let r = unique_stream.parse::<Token![,]>();
                if unique_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            }
            let r = ecs.parse::<Token![,]>();
            if !ecs.is_empty() {
                if let Err(e) = r {
                    return Err(e);
                }
            }
        }

        // Parse Queries
        if ecs.lookahead1().peek(kw::queries) {
            ecs.parse::<kw::queries>()?;
            ecs.parse::<Token![:]>()?;
            let query_stream;
            braced!(query_stream in ecs);

            loop {
                let mut query: Query = query_stream.parse()?;
                query.id = Some(id_counter);
                queries.push(query);

                id_counter += 1;

                let r = query_stream.parse::<Token![,]>();
                if query_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            }
            let r = ecs.parse::<Token![,]>();
            if !ecs.is_empty() {
                if let Err(e) = r {
                    return Err(e);
                }
            }
        }

        if ecs.lookahead1().peek(kw::systems) {
            ecs.parse::<kw::systems>()?;
            ecs.parse::<Token![:]>()?;
            let system_stream;
            braced!(system_stream in ecs);

            loop {
                let system: System = system_stream.parse()?;
                systems.push(system);

                let r = system_stream.parse::<Token![,]>();
                if system_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            }
            let r = ecs.parse::<Token![,]>();
            if !ecs.is_empty() {
                if let Err(e) = r {
                    return Err(e);
                }
            }
        }

        if ecs.lookahead1().peek(kw::tasklists) {
            ecs.parse::<kw::tasklists>()?;
            ecs.parse::<Token![:]>()?;
            let task_stream;
            braced!(task_stream in ecs);

            loop {
                let task: Task = task_stream.parse()?;
                tasks.push(task);

                let r = task_stream.parse::<Token![,]>();
                if task_stream.is_empty() {
                    break;
                } else if let Err(e) = r {
                    return Err(e);
                }
            }
            let r = ecs.parse::<Token![,]>();
            if !ecs.is_empty() {
                if let Err(e) = r {
                    return Err(e);
                }
            }
        }

        input.parse::<Token![;]>()?;

        Ok(Self {
            visibility,
            name,
            components,
            uniques,
            queries,
            systems,
            tasks,
        })
    }
}
