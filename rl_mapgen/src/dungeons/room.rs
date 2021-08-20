use std::{fmt::Display, fs::File, io::Read, path::Path};

use rl_utils::Coord;

use crate::{
    dungeons::{Dungeon, DungeonBuilder, DungeonConfigurer, DungeonParams},
    utils::Tile,
};

/// Generate a simple singular room [Dungeon](dungeons/struct.Dungeon.html)
/// Afterwards it goes over the map to connect any disconnected areas and then applies a heatmap to
/// locate large open areas which it classifies as rooms. No corridors are created.
///
/// In addition to the usual creation interface, Room also implements from_file() to load an prefab
/// dungeon from a file.
///
/// ```rust
///     use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, Room};
///
///     let map = Room::from_file("./examples/test_dungeon.dng").generate();
///     println!("{}", map);
/// ```
///
/// For most [DungeonCombiner](dungeon/trait.DungeonCombiner.html) this is the default room.
///
///  [from_file()]: dungeons/trait.DungeonBuilder.html#method.finalize
#[derive(PartialEq, Hash, Debug, Clone)]
pub struct Room {
    params:      DungeonParams,
    str_dungeon: Option<String>,
}
impl Room {
    pub fn from_file<P>(path: P) -> Self
        where P: AsRef<Path> + Display, {
        let mut dfile = File::open(&path).unwrap_or_else(|_| panic!("Unable to open {}!", path));
        let mut contents = String::new();
        dfile.read_to_string(&mut contents).unwrap_or_else(|_| panic!("Unable to read from {}!", path));

        let size = Room::dungeon_size(&contents);

        Room { params: DungeonParams::new(size.x, size.y), str_dungeon: Some(contents), }
    }

    fn dungeon_size(s: &str) -> Coord {
        let mut xend = -1;
        let mut len = 0;
        for (i, c) in s.chars().enumerate() {
            if xend == -1 && c == '\n' {
                xend = i as isize;
            }

            len = i as isize;
        }

        let xsize = xend + 1;
        Coord::new(xsize, len / xsize)
    }
}
impl DungeonBuilder for Room {
    fn minimum_size(&self) -> Coord {
        if self.str_dungeon.is_some() {
            self.params.area.size
        } else {
            Coord::new(4, 4)
        }
    }

    fn get_params(&self) -> DungeonParams {
        self.params
    }

    fn get_name(&self) -> String {
        "Room".to_string()
    }

    fn generate_with_params(&self, params: DungeonParams) -> Dungeon {
        let mut output = Dungeon::new(self.get_name(), params);
        output.map.fill(Tile::Floor);

        if let Some(ref dungeon) = self.str_dungeon {
            for (i, c) in dungeon.chars().enumerate() {
                let point = Coord::new(i as isize % self.params.area.size.x, i as isize / self.params.area.size.x);

                output.map[point] = c.into();
            }
        } else {
            for y in 0..output.area.size.y {
                output.map[(0, y)] = Tile::Wall;
                output.map[(output.area.size.x - 1, y)] = Tile::Wall;
            }
            for x in 0..output.area.size.x {
                output.map[(x, 0)] = Tile::Wall;
                output.map[(x, output.area.size.y - 1)] = Tile::Wall;
            }
        }

        output.rooms.push(params.area);
        output
    }
}
impl DungeonConfigurer for Room {
    fn new(size_x: isize, size_y: isize) -> Self {
        Room { params: DungeonParams::new(size_x, size_y), str_dungeon: None, }
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
