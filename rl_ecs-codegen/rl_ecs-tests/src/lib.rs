use rl_ecs::create_ecs;
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
pub struct Timer {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Player {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Time {}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Counter {
    pub ctr: usize,
}

create_ecs! {
    pub world Ecs {
        components: {
            Creature: { [Stats;11], ToolUser, Location, Movable, Action },
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
            Player: { Stats, ToolUser, Location, Movable, Action, },
            Time: { Timer, },
            Counter,
        },
        queries: {
            TransferItem: { ToolUser<Inventory<Item>>, Location, Action, },
            // TilePos: { Tile<Location,Inventory<Item>> },
        },
        systems: {
            #[for_each, state: u32, weight = low]
            single = { &mut Timer },
            #[add_stores = [Inventory, Item]]
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
                #[add_stores = [Inventory, Item]]
                pickup_item: <TransferItem, TilePos> = { &mut Inventory, &mut Item, &Location, &mut Action, &Counter},

                // Check that atleast one store is requested per query used.
                // Check for duplicates
                // for_each systems may have either 0 queries and 1 store, or 1 query and X stores whith all non-unique stores matching that of the query.
                // Check that uniques are not listed in remove_stores
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
    use crate::ecs::keys::StatsKey;
    use crate::*;
    use rl_ecs::stores::{
        StoreExBasic, StoreExCreate, StoreExCreateAttach, StoreExGetChild, StoreExGetParent,
        StoreExPurge, UniqueStore, UniqueStoreKey,
    };

    #[test]
    fn it_works() {
        let mut ecs = Ecs::new(Player {}, Time {}, Counter { ctr: 0 });

        let c1 = ecs.create(Creature {});
        ecs.get(c1).unwrap();
        let s1 = ecs.create_and_attach(c1, Stats {}).unwrap();
        let l1 = ecs.create_and_attach(c1, Location {}).unwrap();

        let l1_parent = ecs.get_parent(l1).unwrap();
        let s1_parent = ecs.get_parent(s1).unwrap();
        assert_eq!(c1, l1_parent);
        assert_eq!(c1, s1_parent);

        let t: Option<&Stats> = ecs.get(s1);
        assert!(t.is_some());

        ecs.purge(s1);
        let t: Option<&Stats> = ecs.get(s1);
        assert!(t.is_none());

        ecs.purge(c1);
        let c: Option<&Creature> = ecs.get(c1);
        let l: Option<&Location> = ecs.get(l1);
        assert!(c.is_none());
        assert!(l.is_none());

        let _: &Player = ecs.get_unique();
        let _: &mut Player = ecs.get_unique_mut();
        let _: &Time = ecs.get_unique();
        let _: &mut Time = ecs.get_unique_mut();

        let player_key = Player::unique_key();
        let s2 = ecs.create_and_attach(player_key, Stats {}).unwrap();
        let s2_a: Option<&StatsKey> = ecs.get_children(player_key).unwrap().next();
        assert!(s2_a.is_some());
        assert_eq!(s2, *s2_a.unwrap());
        assert_eq!(player_key, ecs.get_parent(s2).unwrap());
    }
}
