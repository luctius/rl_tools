use pathfinding::directed::astar::astar;
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};

use rl_utils::{
    dijkstra_map::{DijkstraMap, DijkstraMapValue},
    Area, Coord, Map, MapMovement, MapObject, MovementCost,
};

use crate::dungeons::Dungeon;
use crate::utils::Tile;

type SpawnCallback = dyn Fn(Tile, Tile, &[(Coord, Tile)], &Map<Tile>, u64) -> Option<Coord>;

/* TODO: Avoid placing monsters near start position! */

/// Modifies the Spawning behaviour of
/// [Dungeon::with_spawn_point()](../dungeons/struct.Dungeon.html#method.with_spawn_point),
/// [Dungeon::add_spawn_point()](../dungeons/struct.Dungeon.html#method.with_spawn_point),
/// [Dungeon::with_stairs()](../dungeons/struct.Dungeon.html#method.with_stairs) and
/// [Dungeon::add_stairs()](../dungeons/struct.Dungeon.html#method.add_stairs).
///
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub enum SpawnPlacements<'a> {
    /// If the dungeon contains two or more stairs, an AStar path is generated between each of
    /// them. Then a Heatmap is generated with the path as the coolest points. The spawn-point is
    /// selected from a random high point.
    AvoidCriticalPath,
    /// A Heatmap is generated with each similar [Tile](../utils/tile/enum.Tile.html) as the coolest points. A spawn-point is
    /// selected from a random high point.
    AvoidSimilar,
    /// A Heatmap is generated with each similar [Tile](../utils/tile/enum.Tile.html) as the coolest points. A spawn-point is
    /// selected from a random low point.
    ClusterSimilar,
    /// A random room is selected from all the rooms available, then a random point of the selected
    /// [Tile](../utils/tile/enum.Tile.html) type is selected from that room.
    /// from the room.
    RandomRoom,
    /// A random secret room is selected from all the rooms available, then a random point of the selected
    /// [Tile](../utils/tile/enum.Tile.html) type is selected from that room.
    /// from the room.
    RandomSecretRoom,
    /// A random corridor is selected from all the corridors available, then a random point of the selected
    /// [Tile](../utils/tile/enum.Tile.html) type is selected from that room.
    /// from the room.
    RandomCorridor,
    /// A random point is selected from the map, if the tile type is of the specific [Tile](../utils/tile/enum.Tile.html), it is
    /// returned. If not we try again for a fixed number of times.
    Random,
    /// A Heatmap is generated with each Near [Tile](../utils/tile/enum.Tile.html) as the coolest points. A spawn-point is
    /// selected from a random low point of the specific [Tile](../utils/tile/enum.Tile.html).
    Near(Tile),
    /// A custom spawn-point algorithm taking a closure.
    Custom(&'a SpawnCallback),
}
impl<'a> std::fmt::Display for SpawnPlacements<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SpawnPlacements::AvoidCriticalPath => write!(f, "SpawnPlacements::AvoidCriticalPath"),
            SpawnPlacements::AvoidSimilar => write!(f, "SpawnPlacements::AvoidSimilar"),
            SpawnPlacements::ClusterSimilar => write!(f, "SpawnPlacements::ClusterSimilar"),
            SpawnPlacements::RandomRoom => write!(f, "SpawnPlacements::RandomRoom"),
            SpawnPlacements::RandomSecretRoom => write!(f, "SpawnPlacements::RandomSecretRoom"),
            SpawnPlacements::RandomCorridor => write!(f, "SpawnPlacements::RandomCorridor"),
            SpawnPlacements::Random => write!(f, "SpawnPlacements::Random"),
            SpawnPlacements::Near(tile) => write!(f, "SpawnPlacements::Near: {}", tile.to_string()),
            SpawnPlacements::Custom(_) => write!(f, "SpawnPlacements::Custom Function"),
        }
    }
}
impl<'a> SpawnPlacements<'a> {
    fn avoid_critical_path(
        &self,
        target: Tile,
        features: &[(Coord, Tile)],
        map: &Map<Tile>,
        seed: u64,
    ) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let walkable = if let MovementCost::Possible(_) = target.is_walkable() { true } else { false };

        let mut stairs = vec![];
        for (c, t) in map.iter() {
            if t == Tile::Stairs {
                stairs.push(c);
            }
        }

        let mut critical_path = vec![];
        for s1 in &stairs {
            for s2 in stairs.iter().rev() {
                if *s1 == *s2 {
                    break;
                }

                if let Some((result, _)) = astar(
                    s1,
                    |p| map.walkable_tiles(*p, MapMovement::Orthogonal),
                    |p| p.pyth(*s2) as usize,
                    |p| *p == *s2,
                ) {
                    for c in result {
                        critical_path.push(c);
                    }
                }
            }
        }

        let mut dmap = DijkstraMap::new(map.area.size).seed_map(|c| {
            if walkable {
                if let MovementCost::Possible(_) = map[c].is_walkable() {
                    DijkstraMapValue::Default
                } else {
                    DijkstraMapValue::Impassable
                }
            } else {
                DijkstraMapValue::Impassable
            }
        });
        for c in critical_path {
            dmap = dmap.with_goal(c);
        }
        let dmap = dmap.calculate();

        let mut target_list = vec![];
        for (c, _) in dmap.map.iter() {
            if let DijkstraMapValue::NonGoal(cost) = dmap.map[c] {
                if map[c] == target && !features.iter().any(|(x, _)| *x == c) {
                    target_list.push((c, cost));
                }
            } else if let DijkstraMapValue::Default = dmap.map[c] {
                if map[c] == target && !features.iter().any(|(x, _)| *x == c) {
                    target_list.push((c, 0));
                }
            }
        }

        if !target_list.is_empty() {
            if let Some(max) = target_list.iter().max_by(|(_, p1), (_, p2)| p1.cmp(p2)).map(|t| t.1) {
                let max = (max * 50) / 100;
                let target_list =
                    target_list.iter().filter(|(_, cost)| *cost >= max).map(|(c, _)| *c).collect::<Vec<_>>();
                target_list.choose(&mut rng).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    fn avoid_similar(
        &self,
        spawn_type: Tile,
        target: Tile,
        features: &[(Coord, Tile)],
        map: &Map<Tile>,
        seed: u64,
    ) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let walkable = if let MovementCost::Possible(_) = target.is_walkable() { true } else { false };

        let mut dmap = DijkstraMap::new(map.area.size).seed_map(|c| {
            if walkable {
                if let MovementCost::Possible(_) = map[c].is_walkable() {
                    DijkstraMapValue::Default
                } else {
                    DijkstraMapValue::Impassable
                }
            } else {
                DijkstraMapValue::Impassable
            }
        });
        for (c, tile) in features {
            if *tile == spawn_type {
                dmap = dmap.with_goal(*c);
            }
        }
        let dmap = dmap.calculate();

        let mut target_list = vec![];
        for (c, _) in dmap.map.iter() {
            if let DijkstraMapValue::NonGoal(cost) = dmap.map[c] {
                if map[c] == target && !features.iter().any(|(x, _)| *x == c) {
                    target_list.push((c, cost));
                }
            }
        }

        if !target_list.is_empty() {
            if let Some(max) = target_list.iter().max_by(|(_, p1), (_, p2)| p1.cmp(p2)).map(|t| t.1) {
                let max = (max * 70) / 100;
                let target_list =
                    target_list.iter().filter(|(_, cost)| *cost >= max).map(|(c, _)| *c).collect::<Vec<_>>();
                target_list[..].choose(&mut rng).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    fn cluster_similar(
        &self,
        spawn_type: Tile,
        target: Tile,
        features: &[(Coord, Tile)],
        map: &Map<Tile>,
        seed: u64,
    ) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let walkable = if let MovementCost::Possible(_) = target.is_walkable() { true } else { false };

        let mut dmap = DijkstraMap::new(map.area.size).seed_map(|c| {
            if walkable {
                if let MovementCost::Possible(_) = map[c].is_walkable() {
                    DijkstraMapValue::Default
                } else {
                    DijkstraMapValue::Impassable
                }
            } else {
                DijkstraMapValue::Impassable
            }
        });
        for (c, tile) in features {
            if *tile == spawn_type {
                dmap = dmap.with_goal(*c);
            }
        }
        let dmap = dmap.calculate();

        let mut target_list = vec![];
        for (c, _) in dmap.map.iter() {
            if let DijkstraMapValue::NonGoal(cost) = dmap.map[c] {
                if map[c] == target && !features.iter().any(|(x, _)| *x == c) {
                    target_list.push((c, cost));
                }
            }
        }

        if !target_list.is_empty() {
            if let Some(min) = target_list.iter().min_by(|(_, p1), (_, p2)| p1.cmp(p2)).map(|t| t.1) {
                let min = (min * 100) / 70;
                let target_list =
                    target_list.iter().filter(|(_, cost)| *cost <= min).map(|(c, _)| *c).collect::<Vec<_>>();
                target_list[..].choose(&mut rng).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    /* TODO: select a specific tile type. */
    fn random_room(&self, rooms: &[Area], seed: u64) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);

        if !rooms.is_empty() {
            let area = rooms.choose(&mut rng);
            if let Some(avec) = area.map(|a| a.iter().collect::<Vec<_>>()) {
                avec.choose(&mut rng).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    fn random_secret_room(&self, srooms: &[Area], seed: u64) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);

        if !srooms.is_empty() {
            let area = srooms.choose(&mut rng);
            if let Some(vec) = area.map(|a| a.iter().collect::<Vec<_>>()) {
                vec.choose(&mut rng).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    /* TODO: select a specific tile type. */
    fn random_corridor(&self, corridors: &[Vec<Coord>], seed: u64) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);

        if !corridors.is_empty() {
            let area = corridors.choose(&mut rng);
            if let Some(vec) = area.map(|a| a.iter().collect::<Vec<_>>()) {
                vec.choose(&mut rng).cloned().cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    fn random(&self, target: Tile, map: &Map<Tile>, seed: u64) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut ctr = 0;

        loop {
            ctr += 1;
            let c = Coord::new(rng.gen_range(0, map.area.size.x), rng.gen_range(0, map.area.size.y));
            if map[c] == target {
                break Some(c);
            }
            if ctr == (map.area.size.y * map.area.size.y) / 10 {
                break None;
            }
        }
    }
    fn near(
        &self,
        near_tile: Tile,
        spawn_type: Tile,
        target: Tile,
        features: &[(Coord, Tile)],
        map: &Map<Tile>,
        seed: u64,
    ) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut near_vec = vec![];

        if !features.is_empty() {
            for (c, t) in features {
                if *t == spawn_type {
                    near_vec.push(*c);
                }
            }
        } else {
            for (c, t) in map.iter() {
                if t == near_tile {
                    near_vec.push(c);
                }
            }
        }

        let mut target_list = vec![];
        for y in 0..map.area.size.y {
            for x in 0..map.area.size.x {
                if map[(x, y)] == target {
                    let mut best = (Coord::new(0, 0), std::isize::MAX);

                    for c in &near_vec {
                        let pyth = c.pyth((x, y).into());
                        if pyth < best.1 {
                            best = ((x, y).into(), pyth);
                        }
                    }
                    if best.0 != (0, 0).into() {
                        target_list.push(best);
                    }
                }
            }
        }

        if !target_list.is_empty() {
            if let Some(min) = target_list.iter().min_by(|(_, p1), (_, p2)| p1.cmp(p2)).map(|t| t.1) {
                let target_list =
                    target_list.iter().filter(|(_, cost)| *cost <= min + 1).map(|(c, _)| *c).collect::<Vec<_>>();
                target_list.choose(&mut rng).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn place(
        self,
        spawn_type: Tile,
        target: Tile,
        features: &[(Coord, Tile)],
        dungeon: &Dungeon,
        seed: u64,
    ) -> Option<Coord> {
        match self {
            SpawnPlacements::AvoidCriticalPath => self.avoid_critical_path(target, features, &dungeon.map, seed),
            SpawnPlacements::AvoidSimilar => self.avoid_similar(spawn_type, target, features, &dungeon.map, seed),
            SpawnPlacements::ClusterSimilar => self.cluster_similar(spawn_type, target, features, &dungeon.map, seed),
            SpawnPlacements::RandomRoom => self.random_room(&dungeon.rooms, seed),
            SpawnPlacements::RandomSecretRoom => self.random_secret_room(&dungeon.secret_rooms, seed),
            SpawnPlacements::RandomCorridor => self.random_corridor(&dungeon.corridors, seed),
            SpawnPlacements::Random => self.random(target, &dungeon.map, seed),
            SpawnPlacements::Near(tile) => self.near(tile, spawn_type, target, features, &dungeon.map, seed),
            SpawnPlacements::Custom(func) => func(spawn_type, target, features, &dungeon.map, seed),
        }
    }
}
