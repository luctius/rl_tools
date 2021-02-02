use rand::{rngs::SmallRng, Rng, SeedableRng};

use rl_utils::{tranthong, Area, Coord};

use crate::dungeons::{Dungeon, DungeonBuilder, DungeonConfigurer, DungeonParams};
use crate::utils::{flood_fill, Tile};

/// Generates a single room via the [Irregular Shaped
/// Room](http://www.roguebasin.com/index.php?title=Irregular_Shaped_Rooms) Algorithm.
/// No corridors or extra rooms are created.
#[derive(PartialEq, Hash, Debug, Clone, Copy)]
pub struct PolyRoom {
    params: DungeonParams,
}
impl DungeonBuilder for PolyRoom {
    fn minimum_size(&self) -> Coord {
        Coord::new(7, 7)
    }
    fn get_params(&self) -> DungeonParams {
        self.params
    }
    fn get_name(&self) -> String {
        "PolyRoom".to_string()
    }
    fn generate_with_params(&self, params: DungeonParams) -> Dungeon {
        let mut rng = SmallRng::seed_from_u64(params.seed);
        let mut output = Dungeon::new(self.get_name(), params);
        if params.area.size.x < self.minimum_size().x || params.area.size.y < self.minimum_size().y {
            return output;
        }

        let depth: Coord = ((params.area.size.x * 30) / 100, (params.area.size.y * 30) / 100).into();

        /* Create rectangles*/
        let rec_up = Area::new((depth.x, 1).into(), (params.area.size.x - (2 * depth.x), depth.y).into());
        let rec_down = Area::new(
            (depth.x, params.area.size.y - depth.y - 1).into(),
            (params.area.size.x - (2 * depth.x), depth.y).into(),
        );
        let rec_right = Area::new(
            (params.area.size.x - depth.x - 1, depth.y).into(),
            (depth.x, params.area.size.y - (2 * depth.y)).into(),
        );
        let rec_left = Area::new((1, depth.y).into(), (depth.x, params.area.size.y - (2 * depth.y)).into());

        let mut up_vec = vec![];
        let mut down_vec = vec![];
        let mut right_vec = vec![];
        let mut left_vec = vec![];

        /* Select points in rectangles*/
        for _ in 0..rng.gen_range(1, depth.x) {
            let x = rng.gen_range(0, rec_up.size.x) + rec_up.position.x;
            let y = rng.gen_range(0, rec_up.size.y) + rec_up.position.y;
            up_vec.push(Coord { x, y });
        }
        for _ in 0..rng.gen_range(1, depth.x) {
            let x = rng.gen_range(0, rec_down.size.x) + rec_down.position.x;
            let y = rng.gen_range(0, rec_down.size.y) + rec_down.position.y;
            down_vec.push(Coord { x, y });
        }
        for _ in 0..rng.gen_range(1, depth.y) {
            let x = rng.gen_range(0, rec_right.size.x) + rec_right.position.x;
            let y = rng.gen_range(0, rec_right.size.y) + rec_right.position.y;
            right_vec.push(Coord { x, y });
        }
        for _ in 0..rng.gen_range(1, depth.y) {
            let x = rng.gen_range(0, rec_left.size.x) + rec_left.position.x;
            let y = rng.gen_range(0, rec_left.size.y) + rec_left.position.y;
            left_vec.push(Coord { x, y });
        }

        /* Sort Vectors*/
        up_vec.sort_unstable_by(|a, b| a.x.cmp(&b.x));
        down_vec.sort_unstable_by(|a, b| b.x.cmp(&a.x));
        right_vec.sort_unstable_by(|a, b| a.y.cmp(&b.y));
        left_vec.sort_unstable_by(|a, b| b.y.cmp(&a.y));

        up_vec.append(&mut right_vec);
        up_vec.append(&mut down_vec);
        up_vec.append(&mut left_vec);
        let mut iter = up_vec.iter();

        if let Some(last) = iter.next() {
            let mut last = *last;
            for next in iter {
                for c in tranthong(last, *next) {
                    output.map[c] = Tile::Wall;
                }
                last = *next;
            }
            for c in tranthong(last, up_vec[0]) {
                output.map[c] = Tile::Wall;
            }
        }
        flood_fill(
            &mut output.map,
            (params.area.size.x / 2, params.area.size.y / 2).into(),
            Tile::Floor,
            Tile::Transparent,
        );

        output.rooms.push(params.area);
        output
    }
}
impl DungeonConfigurer for PolyRoom {
    fn new(size_x: isize, size_y: isize) -> Self {
        PolyRoom { params: DungeonParams::new(size_x, size_y) }
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
