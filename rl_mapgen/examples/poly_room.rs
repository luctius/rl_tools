use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, PolyRoom};

pub fn main() {
    let map = PolyRoom::new(30, 30).generate();
    println!("{}", map);
}
