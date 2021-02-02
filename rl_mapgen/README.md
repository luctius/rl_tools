# rl_mapgen

 A Map generation library for roguelikes.

 # Usage

 To use this library simply add the following line to your Cargo.toml dependencies section:

 ```toml
 rl_mapgen = "*"
 ```

 Then import your crate by putting the following line in your main.rs:
 ```rust,ignore
 extern crate rl_mapgen;
 ```

 # Introduction

 This library lets you generate a variety roguelike maps, and is able to select spawn points
 for your creatures, treasures, traps or other things which you like to populate your dungeons
 with.

 The general idea of this library is that every room type can become a dungeon, but there are
 special generators (Combiners), which can take other dungeon generators and combine them with
 their special algorithm to create a final dungeon.

 # Examples

 The following code will generate an empty square room of 30 by 30 tiles, and print the result
 to stdout.

 ```rust
 use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, Room};

 let map = Room::new(30, 30).generate();
 println!("{}", map);
 ```

 The same procedure can be used to generate a cave and print it.

 ```rust
 use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, CellularAutomata};

 let map = CellularAutomata::new(30, 30).finalize();
 println!("{}", map);
 ```

 To generate a more complete dungeon, the following can be used:
 ```rust
use rl_mapgen::dungeons::{
    DungeonBuilder, DungeonCombiner, DungeonConfigurer,
    CellularAutomata, PolyRoom, Stack, Walker,
};
use rl_mapgen::spawn_placement::SpawnPlacements;
use rl_mapgen::utils::Tile;

    let wolf = 'w';
    let goblin = 'g';
    let troll = 't';
    let orc = 'o';
    let gold = '$';
    let map = Stack::new(150, 60)
        .with_rng_seed(1096746485607392085u64) // Making the build reproducable.
        .with_additional_builder(50, Box::new(CellularAutomata::new(0, 0)))
        .with_additional_builder(50, Box::new(Walker::new(0, 0)))
        .with_additional_builder(50, Box::new(PolyRoom::new(0, 0)))
        .finalize()
        .with_stairs(Tile::Floor, SpawnPlacements::Near(Tile::BorderWall))
        .with_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar)
        .with_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar)
        .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::RandomRoom)
        .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::ClusterSimilar)
        .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::RandomRoom)
        .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::ClusterSimilar)
        .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::RandomCorridor)
        .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::ClusterSimilar)
        .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::Random)
        .with_secret_room()
        .with_secret_room()
        .with_spawn_point(Tile::Feature(gold), Tile::Floor, SpawnPlacements::RandomSecretRoom)
        .with_spawn_point(Tile::Feature(gold), Tile::Floor, SpawnPlacements::RandomSecretRoom);
    println!("{}", map);
 ```
 generates:
 ![Image](https://raw.githubusercontent.com/luctius/rl_mapgen/master/examples/stack_example.png)

 Let's walk through this code.

 First we use the [Stack] generator to create a map of 150 width and 60 height. Since this is an
 example, a seed is used so that the image below will match this example. [Stack] is a generator
 which is able to use other dungeon generators to generate the individual rooms. For diversity,
 we add 3 additional builders, each with 50% chance. In theory [Stack] could also take another
 [Stack] as a room builder, though that only works with larger maps.

 Then we use [finalize()] to generate the map and to give us the [Dungeon] type. On this type we
 can add features to the map.

 We first add some stairs, the first one is locates at the edge of the map but still on a floor
 tile. The additional stairs are located as far away as possible from the previous stairs.

 Then we add a couple of spawn points, and designate them with a [Tile::Feature(char)]. This is
 mostly for your convenience. The [char] will be displayed when you print the map, and later-on
 when you import the map into your game, it helps you distinguish what the spawnpoint was for.

 We add a 2 secret rooms, which are small rooms attached to a corridor with a secret door.

 Finally we create two gold spawn points in random secret rooms.

 To use the generated map in your own game, the resulting dungeon supports the following
 iterators:

 * **.iter()** -> returns ( (isize,isize), [Tile]). The tuple is (x-coordinate,y-coordinate), and the
 [Tile] can be [Tile::Wall], [Tile::Floor], [Tile::Door], [Tile::SecretDoor] or [Tile::BorderWall]. The
 last one defines the border of the map.
 * **.corridor_iter()** -> returns a [Vec]<[[Coord]]>, thus the list of coordinates of the corridor.
 * **.room_iter()** -> returns an [Area], which denotes the space inhabited by the room.
 * **.secret_room_iter()** -> returns an [Area], which denotes the space inhabited by the secret room.
 * **.stair_iter()** -> returns an [Coord] representing a stair. The direction of the stair (up,
 down, both), is up to the user.
 * **.spawn_iter()** -> returns ([Coord], [Tile]), where [Coord] is the location and [Tile] the spawn
 type as given to [with_spawn_point()];

 Something like:
 ```rust,ignore
 for t in map.iter() {
    match t {
        Tile::Floor => { ... },
        Tile::Door => { ... },
        Tile::Wall => { ... },
        Tile::BorderWall => { ... },
        _ => { unimplemented!() },
    }
 }
 ```

 The code then has to deal with the stairs, and spawn-points.

 [DungeonConfigurer]: dungeons/trait.DungeonConfigurer.html
 [DungeonBuilder]: dungeons/trait.DungeonBuilder.html
 [DungeonCombiner]: dungeons/trait.DungeonCombiner.html
 [Dungeon]: dungeons/struct.Dungeon.html
 [Stack]: dungeons/stack/struct.Stack.html
 [finalize()]: dungeons/trait.DungeonBuilder.html#method.finalize
 [with_spawn_point()]: dungeons/struct.Dungeon.html#method.with_spawn_point
 [Tile]: utils/tile/enum.Tile.html
 [Tile::Wall]: utils/tile/enum.Tile.html
 [Tile::Floor]: utils/tile/enum.Tile.html
 [Tile::SecretDoor]: utils/tile/enum.Tile.html
 [Tile::BorderWall]: utils/tile/enum.Tile.html
 [Tile::Feature(char)]: utils/tile/enum.Tile.html
 [Coord]: utils_reexport/struct.Coord.html
 [Area]: utils_rexport/struct.Area.html

Current version: 0.1.0

License: MIT OR Apache-2.0
