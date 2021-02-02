use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, Walker};

pub fn main() {
    let map = Walker::new(150, 60).generate();
    println!("{}", map);
}
