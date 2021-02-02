use pathfinding::prelude::dijkstra_all;

use rl_utils::{Coord, Map, MapMovement, MapObject, MovementCost};

use crate::corridors::walker_corridor;
use crate::utils::Tile;

pub fn flood_fill<T>(map: &mut Map<T>, fill_point: Coord, fill_char: T, to_fill_char: T)
where
    T: MapObject + Copy,
{
    dijkstra_all(&fill_point, |pos| {
        let retvec = vec![(-1, 0).into(), (0, -1).into(), (1, 0).into(), (0, 1).into()];

        let retvec = retvec
            .iter()
            .map(|p| *pos + *p)
            .filter(|p| map.area.point_within(*p) && map[p] == to_fill_char)
            .collect::<Vec<_>>();

        if map[pos] == to_fill_char {
            map[pos] = fill_char;
        }

        retvec.into_iter().map(|p: Coord| (p, 1))
    });
}

pub fn connect_map(map: &mut Map<Tile>) {
    /* Find a floor tile */
    let p1: Option<Coord> =
        map.iter().find_map(|(c, t)| if let MovementCost::Possible(_) = t.is_walkable() { Some(c) } else { None });

    if let Some(start) = p1 {
        'outer: loop {
            /* Create a Dijkstra Map */
            let result = dijkstra_all(&start, |pos| map.walkable_tiles(*pos, MapMovement::Orthogonal));

            /* Test if all walkable tiles are in the Dijkstra Map*/
            for test in map.area.iter() {
                if let MovementCost::Possible(_) = map[test].is_walkable() {
                    if start == test {
                        continue;
                    }

                    /* If a point is not...*/
                    if result.get(&test).is_none() {
                        /* Search for its nearest companion which IS in the map. */
                        let mut nearest = None;
                        let mut nearest_dist = isize::max_value();
                        for (near, t) in map.iter() {
                            if let MovementCost::Possible(_) = t.is_walkable() {
                                if result.get(&near).is_some() && near.pyth(test) < nearest_dist {
                                    /* And remember it. */
                                    nearest_dist = near.pyth(test);
                                    nearest = Some(near);
                                }
                            }
                        }

                        /* Then, create a passage from it to the tile which was not connected */
                        if let Some(start) = nearest {
                            for c in walker_corridor(start, test) {
                                map[c] = Tile::Floor;
                            }
                            continue 'outer;
                        }
                    }
                }
            }

            break;
        }
    }
}
