use rl_mapgen::dungeons::{CellularAutomata, DungeonBuilder, DungeonConfigurer};

pub fn main() {
    let map = CellularAutomata::new(150, 60).generate();
    println!("{}", map);
}
