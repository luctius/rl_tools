use rl_ecs_codegen::create_ecs;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Creature {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Stats {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Inventory {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Item {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Tile {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Weight {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Action {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Damage {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Location {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Movable {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct ToolUser {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Timer {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Player {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Time {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct Counter {
    ctr: usize,
}

create_ecs! {
    pub world Ecs {
        components: {
            Creature: { [Stats;2], ToolUser, Location, Movable, Action },
            Stats,
            Inventory: { [Item] },
            Item,
            Tile: { Location, Inventory, },
            Weight: {Creature},
            Action,
            Damage,
            Location,
            Movable,
            ToolUser: { Inventory, },
            Timer,
        },
        uniques: {
            Player: { Stats, ToolUser, Location, Movable, Action },
            Time: { Timer },
            Counter,
        },
        queries: {
            TransferItem: { ToolUser<Inventory<Item>>, Location, Action },
            TilePos: { Tile<Location,Inventory<Item>> },
        },
        systems: {
            #[for_each, state: u32]
            single = { &mut Timer },
            #[stores = [Inventory, Item]]
            pickup_item: <TransferItem, TilePos> = { &mut Inventory, &mut Item, &Location, &mut Action, &Counter},
        },
        tasklists: {
            work = {
                single,
                #[after = single]
                pickup_item,
            }
        },
    };
}

/*
    Entity is an id to a specify component.

    Components are simple structs, but within the ECS they can have relations to other components; they (can) have a parent, and may have child componens.
    A Component can take:
        - no children
        - multiple children of different components (like a struct)
        - multiple children of the same component (like vec or array)
    Additionally, there are modifiers, like Unique, which means there can onyl be 1 instance of that component in the world.

    Uniques are essentially components of which there will only ever be 1 instance. They can have child components as other components,
        though components cannot have Uniques as children.

    Queries are searches for a combination of components and their relatives. They
        are cached within the world and will be automatically updated when a system
        adds or removes components which affect them.

    Systems are functions which run on one or more components. They use queries to define their scope and define what they affect.
    Systems which share resources are run sequentially in the order given.

    Use cases:
        - Modify single component from an entity (modify timer)
        - Modify a component form an entity based on data from other component of that same entity (position based on velocity and direction)
        - Create and attach a component to an entity (add an status effect to a struck enemy)
        - Attach an existing entity to another entity (Pickup a weapon)
        - Delete / purge a component from an entity

    A workload is a collection of 1 or more systems which will block until all systems are run to the end. It will try to
        parallelise the running of it's systems as much as possible.

    create_ecs! {
        pub world Ecs {
            components: {
                Creature: { Stats, ToolUser, Location, Movable, Action },
                Stats,
                Inventory: { [Item] },
                /*OR*/ Inventory: { [Item;2] },
                Item: { Weight },
                Tile: { Location, Inventory, },
                Weight,
                Action,
                Damage,
                Location,
                Movable,
                ToolUser: { Inventory, },
                Timer,

                // Check for cycles
                // check for duplicates
            },
            uniques: {
                Player: { Stats, ToolUser, Location, Movable, Action },
                Time: { Timer },
                Counter,

                // they are essentially components but with only one instance which is instantiated at world creation
            },
            queries: {
                cached: {
                    TransferItem: { ToolUser<Inventory<Item>>, Location, Action },
                    TilePos: { Tile<Location,Inventory<Item>> },

                    // Check for atleast one instance per query
                    // Check for uniques as possible sources for a query
                },
            },

            systems: {
                #[for_each, state: u32]
                single: = { &mut Timer };
                #[stores = [Inventory, Item]]
                pickup_item: <TransferItem, TilePos> = { &mut Inventory, &mut Item, &Location, &mut Action, &Counter},

                // Check that atleast one store is requested per query used.
                // Check for duplicates
                // for_each systems may have either 0 queries and 1 store, or 1 query and X stores whith all non-unique stores matching that of the query.
            },

            tasklists = {
                work = {
                    #[before = pickup_item]
                    single,
                    #[after = single]
                    pickup_item,

                    // Check for cycles
                    // Check where queries should be updated

                }
            },
        }
    };
*/

#[cfg(test)]
mod tests {
    use crate::*;
    use rl_ecs::stores::{ResourceStore, RlEcsStore};

    #[test]
    fn it_works() {
        // let mut ecs = Ecs::new(0, 0);

        // let p1 = ecs.create(Position {});
        // let v1 = ecs.create_and_attach(p1, Velocity {}).unwrap();
        // let m1 = ecs.create_and_attach(p1, Movement {}).unwrap();

        // let v1_parent = ecs.get_parent(v1).unwrap().unwrap();
        // let m1_parent = ecs.get_parent(m1).unwrap().unwrap();
        // assert_eq!(p1, v1_parent);
        // assert_eq!(p1, m1_parent);

        // let t: Option<&Movement> = ecs.get(m1).unwrap();
        // assert!(t.is_some());

        // ecs.purge(m1).unwrap();
        // let t: Option<&Movement> = ecs.get(m1).unwrap();
        // assert!(t.is_none());

        // ecs.purge(p1).unwrap();
        // let v: Option<&Velocity> = ecs.get(v1).unwrap();
        // let p: Option<&Position> = ecs.get(p1).unwrap();
        // assert!(v.is_none());
        // assert!(p.is_none());

        // let _: &u64 = ecs.get_resource();
        // let _: &mut u64 = ecs.get_resource_mut();
        // let _: &u32 = ecs.get_resource();
        // let _: &mut u32 = ecs.get_resource_mut();
    }
}
