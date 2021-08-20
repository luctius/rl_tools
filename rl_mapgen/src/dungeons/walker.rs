use min_max_macros::max;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use rl_utils::{Area, Coord};

use crate::{
    dungeons::{Dungeon, DungeonBuilder, DungeonConfigurer, DungeonParams},
    utils::{flood_fill::flood_fill, Tile},
};

/// This generates a single room [Dungeon](../dungeon/struct.Dungeon.html) by creating a walker in
/// the middle of the room which digs into a certain direction. When it has digged a certain
/// numer of tiles, a new walker is created untill a percentage of the
/// [Tile](../utils/tile/enum.Tile.html)s are turned into
/// [Tile::Floor](../utils/tile/enum.Tile.html).
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum WalkerMovement {
    Orthogonal,
    Diagonal,
    Both,
}
impl WalkerMovement {
    pub fn next_coord(self, pos: Coord, seed: u64) -> Option<Coord> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let movement_mod: [Coord; 8] = [(-1, 0).into(),
                                        (0, -1).into(),
                                        (1, 0).into(),
                                        (0, 1).into(), // ORTHOGANAL
                                        (-1, -1).into(),
                                        (1, -1).into(),
                                        (-1, 1).into(),
                                        (1, 1).into() /* DIAGONAL */];
        let max = match self {
            WalkerMovement::Orthogonal | WalkerMovement::Diagonal => 4,
            WalkerMovement::Both => 8,
        };
        let md = match self {
            WalkerMovement::Orthogonal | WalkerMovement::Both => 0,
            WalkerMovement::Diagonal => 4,
        };

        let idx = rng.gen_range(md, max + md);
        let pos = pos + movement_mod[idx];
        if pos.x < 0 || pos.y < 0 {
            None
        } else {
            Some(pos)
        }
    }

    pub fn iter(self, pos: Coord, seed: u64) -> DLAIter {
        DLAIter { movement: self, seed, current: pos }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct DLAIter {
    movement: WalkerMovement,
    seed:     u64,
    current:  Coord,
}
impl Iterator for DLAIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        self.seed = SmallRng::seed_from_u64(self.seed).gen();
        if let Some(cur) = self.movement.next_coord(self.current, self.seed) {
            self.current = cur;
            Some(cur)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Hash, Debug, Clone, Copy)]
pub struct Walker {
    params:     DungeonParams,
    movement:   WalkerMovement,
    floor_perc: isize,
}
impl Walker {
    pub fn with_movement(mut self, movement: WalkerMovement) -> Self {
        self.movement = movement;
        self
    }

    pub fn with_floor_percentage(mut self, percentage: isize) -> Self {
        self.floor_perc = percentage.abs() % 100;
        self
    }
}
impl DungeonBuilder for Walker {
    fn minimum_size(&self) -> Coord {
        Coord::new(10, 10)
    }

    fn get_params(&self) -> DungeonParams {
        self.params
    }

    fn get_name(&self) -> String {
        "Walker".to_string()
    }

    fn generate_with_params(&self, params: DungeonParams) -> Dungeon {
        let mut rng = SmallRng::seed_from_u64(params.seed);
        let mut output = Dungeon::new(self.get_name(), params);
        output.map.fill(Tile::Wall);

        if params.area.size.x < self.minimum_size().x || params.area.size.y < self.minimum_size().y {
            return output;
        }

        let map_area = Area::new((2, 2).into(), params.area.size - (5, 5).into());
        let mut blocks = 0;
        let mut run = 0;
        let target_blocks = (((params.area.size.x - 1) * (params.area.size.y - 1)) * self.floor_perc) / 100;
        let mut new_miner = false;
        let mut iter = self.movement.iter((params.area.size.x / 2, params.area.size.y / 2).into(), rng.gen());

        while blocks < target_blocks {
            while new_miner {
                let tmp_pos =
                    (rng.gen_range(1, params.area.size.x - 2), rng.gen_range(1, params.area.size.y - 2)).into();
                if output.map[tmp_pos] == Tile::Floor {
                    new_miner = false;
                    run = 0;
                    iter = self.movement.iter(tmp_pos, rng.gen());
                }
            }

            if let Some(pos) = iter.next() {
                run += 1;
                if map_area.point_within(pos) {
                    if let Some(tile) = output.map.get_mut(pos) {
                        if *tile != Tile::Floor {
                            *tile = Tile::Floor;
                            blocks += 1;
                        }
                    } else {
                        new_miner = true;
                    }
                } else {
                    new_miner = true;
                }
                if run > max!(params.area.size.x, params.area.size.y) {
                    new_miner = true
                }
            }
        }

        flood_fill(&mut output.map, (0, 0).into(), Tile::Transparent, Tile::Wall);

        output.rooms.push(params.area);
        output
    }
}
impl DungeonConfigurer for Walker {
    fn new(size_x: isize, size_y: isize) -> Self {
        Walker { params:     DungeonParams::new(size_x, size_y),
                 movement:   WalkerMovement::Orthogonal,
                 floor_perc: 40, }
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
