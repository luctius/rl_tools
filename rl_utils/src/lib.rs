extern crate pathfinding;
extern crate triangulation;
extern crate yansi;

pub mod area;
pub mod ca;
pub mod coord;
pub mod dijkstra_map;
pub mod map;
pub mod tranthong;

pub use self::area::Area;
pub use self::ca::{ca_generate, CATile, CAparams, CA};
pub use self::coord::Coord;
pub use self::dijkstra_map::{DijkstraMap, DijkstraMapValue};
pub use self::map::{Map, MapIterator, MapMovement, MapObject, MovementCost};
pub use self::tranthong::{tranthong, tranthong_func};

#[cfg(test)]
mod tests {}
