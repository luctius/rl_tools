use crate::utils::Tile;
use rl_utils::{Coord, Map, MapObject, MovementCost};

/* Corridor Builders*/
mod walker_corridor;

pub type CorridorFunction = dyn Fn(Coord, Coord) -> Vec<Coord>;

pub use self::walker_corridor::walker_corridor;

/// This function takes a list of [Coord] and a mutable map and replaces the tiles of the Coord
/// list into floor tiles. Then if add_door is true, it retraces it's steps and adds doors to the
/// corridor.
pub fn create_corridor(corridor: &[Coord], map: &mut Map<Tile>, add_door: bool) {
    let mut first_door = None;
    for c in corridor {
        map[c] = Tile::Floor;
    }

    if add_door {
        for (i, c) in corridor.iter().enumerate() {
            if first_door.is_none() && i < corridor.len() - 1 {
                let delta_abs_next = c.delta_abs(corridor[i + 1]);
                let set_door = if delta_abs_next.x == 1 && delta_abs_next.y == 0 {
                    let above = *c + (0, 1).into();
                    let below = *c + (0, -1).into();
                    map[above].is_walkable() == MovementCost::Impossible
                        && map[below].is_walkable() == MovementCost::Impossible
                } else if delta_abs_next.x == 0 && delta_abs_next.y == 1 {
                    let left = *c + (-1, 0).into();
                    let right = *c + (1, 0).into();
                    map[left].is_walkable() == MovementCost::Impossible
                        && map[right].is_walkable() == MovementCost::Impossible
                } else {
                    false
                };

                if set_door {
                    map[c] = Tile::ClosedDoor;
                    first_door = Some(c);
                    break;
                }
            }
        }
        if corridor.len() > 3 {
            if let Some(fd) = first_door {
                for (i, c) in corridor.iter().rev().enumerate() {
                    if fd.pyth(*c) > 2 && i < corridor.len() - 2 {
                        let delta_abs_next = c.delta_abs(corridor[corridor.len() - i - 2]);
                        let set_door = if delta_abs_next.x == 1 && delta_abs_next.y == 0 {
                            let above = *c + (0, 1).into();
                            let below = *c + (0, -1).into();
                            map[above].is_walkable() == MovementCost::Impossible
                                && map[below].is_walkable() == MovementCost::Impossible
                        } else if delta_abs_next.x == 0 && delta_abs_next.y == 1 {
                            let left = *c + (-1, 0).into();
                            let right = *c + (1, 0).into();
                            map[left].is_walkable() == MovementCost::Impossible
                                && map[right].is_walkable() == MovementCost::Impossible
                        } else {
                            false
                        };

                        if set_door {
                            map[c] = Tile::ClosedDoor;
                            break;
                        }
                    }
                }
            }
        }
    }
}
