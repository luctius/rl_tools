use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, Room};

pub fn main() {
    let map = Room::new(30, 30).generate();
    println!("{}", map);
}
