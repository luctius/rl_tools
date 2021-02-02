use rl_mapgen::dungeons::{
    Bsp, CellularAutomata, DungeonBuilder, DungeonCombiner, DungeonConfigurer, PolyRoom, Room, Stack, Walker,
};
use rl_mapgen::spawn_placement::SpawnPlacements;
use rl_mapgen::utils::Tile;

pub fn main() {
    let wolf = 'w';
    let goblin = 'g';
    let troll = 't';
    let orc = 'o';
    let gold = '$';
    let mut map = Bsp::new(150, 60)
        .with_room_size(10, 80)
        .with_depth(6)
        .with_maximum_room_size_ratio(1.5)
        .with_extra_corridor_chance(20)
        .finalize()
        .with_stairs(Tile::Floor, SpawnPlacements::Near(Tile::BorderWall))
        .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::RandomRoom)
        .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::RandomRoom)
        .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::RandomRoom)
        .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::RandomCorridor);

    for i in 0..5 {
        map = map
            .with_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar)
            .with_spawn_point(Tile::Feature(gold), Tile::Floor, SpawnPlacements::AvoidCriticalPath)
            .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::ClusterSimilar)
            .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::ClusterSimilar)
            .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::ClusterSimilar)
            .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::AvoidSimilar);
    }

    println!("{}", map);
}
