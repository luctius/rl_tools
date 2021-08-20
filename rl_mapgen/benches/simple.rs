extern crate criterion;
extern crate rl_mapgen;

use rl_mapgen::{
    dungeons::{
        Bsp, CellularAutomata, DungeonBuilder, DungeonCombiner, DungeonConfigurer, PolyRoom, Room, Stack, Walker,
    },
    spawn_placement::SpawnPlacements,
    utils::Tile,
};

use criterion::{criterion_group, criterion_main, Criterion};

fn cellular_automata_bench(b: &mut Criterion) {
    b.bench_function("cellular_automata", move |b| {
         b.iter(|| {
              let _ = CellularAutomata::new(100, 100).generate();
          })
     });
}

fn walker_bench(b: &mut Criterion) {
    b.bench_function("walker", move |b| {
         b.iter(|| {
              let _ = Walker::new(100, 100).generate();
          })
     });
}

fn bsp_bench(b: &mut Criterion) {
    b.bench_function("bsp", move |b| {
         b.iter(|| {
              let _ = Bsp::new(100, 50).with_additional_builder(50, Box::new(CellularAutomata::new(0, 0)))
                                       .with_additional_builder(50, Box::new(Walker::new(0, 0)))
                                       .with_additional_builder(50, Box::new(PolyRoom::new(0, 0)))
                                       .with_room_size(90, 140)
                                       .with_depth(4)
                                       .with_maximum_room_size_ratio(2.0)
                                       .with_extra_corridor_chance(40)
                                       .finalize()
                                       .with_stairs(Tile::Floor, SpawnPlacements::Near(Tile::BorderWall))
                                       .with_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar);
          })
     });
}

fn stack_bench(b: &mut Criterion) {
    b.bench_function("stack", move |b| {
         b.iter(|| {
              let stair = '<';
              let wolf = 'w';
              let goblin = 'g';
              let troll = 't';

              let mut map =
                  Stack::new(100, 100).with_additional_builder(50, Box::new(CellularAutomata::new(0, 0)))
                                      .with_additional_builder(50, Box::new(Walker::new(0, 0)))
                                      .with_additional_builder(50, Box::new(PolyRoom::new(0, 0)))
                                      .finalize()
                                      .with_stairs(Tile::Floor, SpawnPlacements::Near(Tile::BorderWall))
                                      .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::RandomRoom)
                                      .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::Random)
                                      .with_spawn_point(Tile::Feature(goblin),
                                                        Tile::Floor,
                                                        SpawnPlacements::RandomCorridor);

              for i in 0..5 {
                  map.add_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar);
                  map.add_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::ClusterSimilar);
                  map.add_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::ClusterSimilar);
                  map.add_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::AvoidSimilar);
              }
          })
     });
}

criterion_group!(benches, /* cellular_automata_bench, bsp_bench, */ stack_bench,);
criterion_main!(benches);
