use rl_mapgen::{
    dungeons::{CellularAutomata, DungeonBuilder, DungeonCombiner, DungeonConfigurer, PolyRoom, Room, Stack, Walker},
    spawn_placement::SpawnPlacements,
    utils::Tile,
};

pub fn main() {
    let wolf = 'w';
    let goblin = 'g';
    let troll = 't';
    let orc = 'o';
    let gold = '$';
    let mut map =
        Stack::new(150, 60).with_additional_builder(50, Box::new(CellularAutomata::new(0, 0)))
                           .with_additional_builder(50, Box::new(Walker::new(0, 0)))
                           .with_additional_builder(50, Box::new(PolyRoom::new(0, 0)))
                           .finalize()
                           .with_stairs(Tile::Floor, SpawnPlacements::Near(Tile::BorderWall))
                           .with_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar)
                           .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::RandomRoom)
                           .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::RandomRoom)
                           .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::Random)
                           .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::RandomCorridor);

    for i in 0..4 {
        map = map.with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::ClusterSimilar)
                 .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::ClusterSimilar)
                 .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::ClusterSimilar)
                 .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::Random);
    }
    for i in 0..6 {
        map = map.with_spawn_point(Tile::Feature(gold), Tile::Floor, SpawnPlacements::AvoidCriticalPath);
    }

    map = map.with_secret_room()
             .with_secret_room()
             .with_secret_room()
             .with_secret_room()
             .with_secret_room()
             .with_secret_room()
             .with_secret_room()
             .with_secret_room();
    println!("{}", map);
}
