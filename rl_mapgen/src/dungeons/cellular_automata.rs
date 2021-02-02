use std::vec::Vec;

use rand::{rngs::SmallRng, Rng, SeedableRng};

use rl_utils::{
    ca_generate,
    dijkstra_map::{DijkstraMap, DijkstraMapValue},
    Area, CATile, CAparams, Coord, Map, CA,
};

use crate::dungeons::{Dungeon, DungeonBuilder, DungeonConfigurer, DungeonParams};
use crate::utils::{flood_fill::connect_map, Tile};

/// Generate a [Dungeon](dungeons/struct.Dungeon.html) via Cellular Automata
/// Afterwards it goes over the map to connect any disconnected areas and then applies a heatmap to
/// locate large open areas which it classifies as rooms. No corridors are created.
#[derive(PartialEq, Hash, Debug, Clone)]
pub struct CellularAutomata {
    params: DungeonParams,
    wall_prob: usize,
    caparams: Vec<CAparams>,
}
impl CellularAutomata {
    pub fn with_initial_wall_probability(mut self, wall_prob: usize) -> Self {
        self.wall_prob = wall_prob % 100;
        self
    }
    pub fn with_ca_params(mut self, caparams: Vec<CAparams>) -> Self {
        self.caparams = caparams;
        self
    }
}
impl DungeonBuilder for CellularAutomata {
    fn minimum_size(&self) -> Coord {
        Coord::new(8, 8)
    }
    fn get_params(&self) -> DungeonParams {
        self.params
    }
    fn get_name(&self) -> String {
        "CellularAutomata".to_string()
    }
    fn generate_with_params(&self, params: DungeonParams) -> Dungeon {
        let mut rng = SmallRng::seed_from_u64(params.seed);
        let mut output = Dungeon::new(self.get_name(), params);

        if params.area.size.x < self.minimum_size().x || params.area.size.y < self.minimum_size().y {
            return output;
        }
        let mut cmap = Map::new(params.area.size);

        /* Fill cmap randomly with walls */
        cmap.fill_each(|_| {
            let tile = if rng.gen_range(0, 100) < self.wall_prob { CATile::Dead } else { CATile::Alive };

            CA { next: tile, tile }
        });

        /* process params */
        for p in &self.caparams {
            for _ in 0..p.count {
                ca_generate(params.area.size, p, &mut cmap);
            }
        }

        /* Copy CA result into map */
        for y in 0..params.area.size.y {
            for x in 0..params.area.size.x {
                output.map[(x, y)] = cmap[(x, y)].into();
            }
        }

        connect_map(&mut output.map);

        /* Search for room areas
         * Since we have only one room with this map, now we use a dijkstra map to search for open
         * areas and designate them as rooms.
         */
        let dmap = DijkstraMap::new(output.area.size)
            .seed_map(|c| if output.map[c] == Tile::Wall { DijkstraMapValue::Goal } else { DijkstraMapValue::Default })
            .calculate()
            .invert();

        for (c, dv) in dmap.map.iter() {
            let mut found = false;
            match dv {
                DijkstraMapValue::Goal => found = true,
                DijkstraMapValue::NonGoal(ng) if ng >= 3 => found = true,
                _ => (),
            }
            if found {
                let mut used = false;
                for r in &output.rooms {
                    if r.point_within(c) {
                        used = true;
                    }
                }
                if !used {
                    let md = (4, 4).into();
                    let area = Area::new(c - md, md + md);
                    if output.area.area_within(area) {
                        output.rooms.push(area);
                    }
                }
            }
        }

        output
    }
}
impl DungeonConfigurer for CellularAutomata {
    fn new(size_x: isize, size_y: isize) -> Self {
        CellularAutomata {
            params: DungeonParams::new(size_x, size_y),
            wall_prob: 40,
            caparams: vec![CAparams { count: 4, r1: 5, r2: 2 }, CAparams { count: 3, r1: 5, r2: 0 }],
        }
    }

    fn with_rng_seed(mut self, seed: u64) -> Self {
        self.params.seed = seed;
        self
    }

    fn with_offset(mut self, start_x: isize, start_y: isize) -> Self {
        self.params.area.position = (start_x, start_y).into();
        self
    }
}
