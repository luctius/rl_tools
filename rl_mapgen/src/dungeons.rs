//! This is the main meat of this crate, the dungeon generators.
//!
//! There are two kinds of Dungeon Generators in this crate.
//!
//!The first are the 'simple' Room Generators
//!
//! Both kinds implement [DungeonConfigurer](trait.DungeonConfigurer.html) and
//! [DungeonBuilder].
//! The last trait, [DungeonCombiner](trait.DungeonCombiner.html) is used by the Dungeon Generators
//! that use multiple Dungoen Generators together with an placing algorithm to create the final
//! dungeon.
//!
//! Using the [finalize] method of the [DungeonBuilder], a [Dungeon] struct is generated. Using this
//! struct, the [Dungeon] can be further populated using various methods.
//!
//! [DungeonConfigurer]: trait.DungeonConfigurer.html
//! [DungeonBuilder]: trait.DungeonBuilder.html
//! [DungeonCombiner]: trait.DungeonCombiner.html
//! [Dungeon]: struct.Dungeon.html
//! [finalize]: struct.Dungeon.html#finalize

use core::slice::Iter;
use std::fmt;

use rand::{prelude::SliceRandom, rngs::SmallRng, FromEntropy, Rng, SeedableRng};

use rl_utils::{Area, Coord, Map, MapIterator, MapObject, MovementCost};

use crate::corridors::{walker_corridor, CorridorFunction};
use crate::spawn_placement::SpawnPlacements;
use crate::utils::{Dir, Tile};

/*Room Builders*/
mod bsp;
mod cellular_automata;
mod poly_room;
mod room;
mod stack;
mod walker;

pub use self::bsp::Bsp;
pub use self::cellular_automata::CellularAutomata;
pub use self::poly_room::PolyRoom;
pub use self::room::Room;
pub use self::stack::Stack;
pub use self::walker::Walker;

/// The basic parameters of any Dungeon Generator
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct DungeonParams {
    ///Size and position of the dungeon.
    area: Area,
    ///The random seed to be used while creating the dungeon.
    seed: u64,
}
impl DungeonParams {
    /// Creates a new [DungeonParams](struct.DungeonParams.html)
    pub fn new(size_x: isize, size_y: isize) -> DungeonParams {
        DungeonParams { area: Area::new((0, 0).into(), (size_x, size_y).into()), seed: SmallRng::from_entropy().gen() }
    }
    /// Similar as [new()](struct.DungeonParams.html#new) but also supplies a random seed. Used to avoid initialising a random
    /// generator from entropy.
    pub fn new_with_seed(size_x: isize, size_y: isize, seed: u64) -> Self {
        DungeonParams { area: Area::new((0, 0).into(), (size_x, size_y).into()), seed }
    }
    /// Sets the start of the Dungeon to something else than (0,0). Only usefull if using it within
    /// another dungeon.
    pub fn with_offset(mut self, offset: Coord) -> Self {
        self.area.position = offset;
        self
    }
}

/// The basic functions which any Dungeon Generator implements.
///
///  ```rust
///     use rl_mapgen::dungeons::{DungeonConfigurer, Room};
///
///     let room = Room::new(30, 30)
///         .with_rng_seed(12345)
///         .with_offset(1,1);
///  ```
pub trait DungeonConfigurer {
    /// Creates a new dungeon.
    fn new(size_x: isize, size_y: isize) -> Self
    where
        Self: Sized;

    /// Adds a specific random seed to be used with the generation of this dungeon.
    fn with_rng_seed(self, seed: u64) -> Self
    where
        Self: Sized;

    /// Adds a specific offset to the dungeons position, which is normally (0,0).
    fn with_offset(self, start_x: isize, start_y: isize) -> Self
    where
        Self: Sized;
}

/// Defines the generic actions of every builder.
/// After creating the builder, which is defined by [DungeonConfigurer],
/// these functions are used.
///
/// The split between [DungeonBuilder] and [DungeonConfigurer] is because
/// [DungeonConfigurer] returns Self, which means the trait cannot be cast
/// into a trait object.  Since a [DungeonCombiner] has to store the [DungeonBuilders],
/// they had to be split.
///
///  ```rust
///     use rl_mapgen::dungeons::{DungeonBuilder, DungeonConfigurer, Room};
///
///     // Creating a Room.
///     let mut room = Room::new(30, 30)
///         .with_rng_seed(12345)
///         .with_offset(1,1);
///     // Using the DungeonBuilder Trait to create the dungeon.
///     let dungeon = room.finalize();
///  ```
/// [DungeonConfigurer]: trait.DungeonConfigurer.html
/// [DungeonBuilder]: trait.DungeonBuilder.html
/// [DungeonCombiner]: trait.DungeonCombiner.html
pub trait DungeonBuilder {
    /// This reports the minimum size that the [DungeonBuilder](trait.DungeonBuilder.html) is able to operate in.
    /// Mainly used by a [DungeonCombiner](trait.DungeonCombiner.html).
    fn minimum_size(&self) -> Coord {
        Coord::new(4, 4)
    }
    /// Retrieves set [DungeonParams](struct.DungeonParams.html).
    fn get_params(&self) -> DungeonParams;

    /// Reports the name of the Dungeon Builder, mainly used for debugging DungeonCombiners.
    fn get_name(&self) -> String;

    /// Generates the map of a [DungeonBuilder](trait.DungeonBuilder.html).
    /// This is used by finalize, which calls this function, before adding a border around the map.
    /// As an end-user you probably want [finalize](trait.DungeonBuilder.html#finalize).
    /// The default implementation of this function calls generate_with_params and uses get_params
    /// to supply the parameters of that function.
    fn generate(&mut self) -> Dungeon {
        let p = self.get_params();
        let mut dungeon = self.generate_with_params(p);

        for y in 0..dungeon.area.size.y {
            for x in 0..dungeon.area.size.x {
                if dungeon.map[(x, y)] == Tile::Transparent {
                    dungeon.map[(x, y)] = Tile::Wall;
                }
            }
        }

        dungeon
    }

    /// Creates the map of the dungeon and returns a [Dungeon](struct.Dungeon.html) struct. This function ensures there
    /// is a border wall around the map.
    fn finalize(&mut self) -> Dungeon {
        let mut dungeon = self.generate();

        for y in 0..dungeon.area.size.y {
            dungeon.map[(0, y)] = Tile::BorderWall;
            dungeon.map[(dungeon.area.size.x - 1, y)] = Tile::BorderWall;
        }
        for x in 0..dungeon.area.size.x {
            dungeon.map[(x, 0)] = Tile::BorderWall;
            dungeon.map[(x, dungeon.area.size.y - 1)] = Tile::BorderWall;
        }

        dungeon
    }

    /// This is the function which does all the hard word and the one that should be implemented
    /// by a [DungeonBuilder]. The implementer should do her best to supply a list of rooms and
    /// corridors to the [Dungeon] before returning the Dungeon at the end of this function.
    /// [DungeonConfigurer]: trait.DungeonConfigurer.html
    /// [DungeonBuilder]: trait.DungeonBuilder.html
    /// [DungeonCombiner]: trait.DungeonCombiner.html
    /// [Dungeon]: struct.Dungeon.html
    fn generate_with_params(&self, params: DungeonParams) -> Dungeon;
}

/// A DungeonCombiner uses the basic DungeonBuilders to generate rooms and then places them on the
/// map by copying their Dungeon structure to it's own map.
/// [DungeonConfigurer]: trait.DungeonConfigurer.html
/// [DungeonBuilder]: trait.DungeonBuilder.html
/// [DungeonCombiner]: trait.DungeonCombiner.html
pub trait DungeonCombiner {
    /// Override the basic DungeonBuilder, which is most often the Room builder.
    fn with_default_builder(self, builder: Box<dyn DungeonBuilder>) -> Self
    where
        Self: Sized;
    /// Add additional builders, with a percentage change. A cumulative percentage larger than
    /// 100% makes sense in that not all builders will be called for every room. Some builders
    /// simply may be too large to fit at that spot. Also the order in which the builders are
    /// supplied matters, and it is generally wise to supply larger builders first, to ensure they
    /// have a chance of actually being used.
    fn with_additional_builder(self, percentage: isize, builder: Box<dyn DungeonBuilder>) -> Self
    where
        Self: Sized;
    /// Override the basic DungeonBuilder, which is most often the walker corridor.
    fn with_default_corridor(self, corridor: Box<CorridorFunction>) -> Self
    where
        Self: Sized;
}

/// The main struct of this crate.
/// After a [DungeonBuilder] is done, this struct will be returned. This can then be used to add
/// additional features to the map in a manner that is builder agnostic, and retrieve the
/// information about the map, such as rooms, corridors, spawn-points etc.
///
///  ```rust
/// use rl_mapgen::dungeons::{DungeonBuilder, DungeonCombiner, DungeonConfigurer, Stack};
/// use rl_mapgen::spawn_placement::SpawnPlacements;
/// use rl_mapgen::utils::Tile;
///
///     let wolf = 'w';
///     let goblin = 'g';
///     let troll = 't';
///     let orc = 'o';
///     let gold = '$';
///     let dungeon = Stack::new(150, 60)
///             .finalize()
///
///             // From this point on, it is using the Dungeon Trait.
///             .with_stairs(Tile::Floor, SpawnPlacements::Near(Tile::BorderWall))
///             .with_stairs(Tile::Floor, SpawnPlacements::AvoidSimilar)
///             .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::RandomRoom)
///             .with_spawn_point(Tile::Feature(wolf), Tile::Floor, SpawnPlacements::ClusterSimilar)
///             .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::RandomRoom)
///             .with_spawn_point(Tile::Feature(orc), Tile::Floor, SpawnPlacements::ClusterSimilar)
///             .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::RandomCorridor)
///             .with_spawn_point(Tile::Feature(goblin), Tile::Floor, SpawnPlacements::ClusterSimilar)
///             .with_spawn_point(Tile::Feature(troll), Tile::Floor, SpawnPlacements::Random)
///             .with_secret_room()
///             .with_spawn_point(Tile::Feature(gold), Tile::Floor, SpawnPlacements::RandomSecretRoom)
///             .with_spawn_point(Tile::Feature(gold), Tile::Floor, SpawnPlacements::RandomSecretRoom);
///  ```
/// [DungeonConfigurer]: trait.DungeonConfigurer.html
/// [DungeonBuilder]: trait.DungeonBuilder.html
/// [DungeonCombiner]: trait.DungeonCombiner.html
#[derive(Debug, PartialEq, Clone)]
pub struct Dungeon {
    pub(crate) builder: String,
    pub(crate) area: Area,
    pub(crate) map: Map<Tile>,
    pub(crate) seed: u64,
    pub(crate) rooms: Vec<Area>,
    pub(crate) secret_rooms: Vec<Area>,
    pub(crate) corridors: Vec<Vec<Coord>>,
    pub(crate) stairs: Vec<Coord>,
    pub(crate) spawn_points: Vec<(Coord, Tile)>,
}
impl Dungeon {
    /// This creates this struct, but should not be used by the user.
    /// This should normally happen only within a [DungeonBuilder].
    /// [DungeonBuilder]: trait.DungeonBuilder.html
    pub(crate) fn new(builder: String, params: DungeonParams) -> Dungeon {
        let mut d = Dungeon {
            builder,
            area: params.area,
            map: Map::new(params.area.size),
            seed: params.seed,
            rooms: vec![],
            secret_rooms: vec![],
            corridors: vec![],
            stairs: vec![],
            spawn_points: vec![],
        };
        d.map.fill(Tile::Transparent);
        d
    }
    /// Add stairs to the map.
    ///
    /// The Target [Tile](../utils/tile/enum.Tile.html) represents the type of tile whichshould be replaced.
    /// Normally this should be either Tile::Floor or Tile::Wall.
    ///
    /// Stairs occupy a tile on the map as if they were terrain.
    pub fn add_stairs(&mut self, target: Tile, placement: SpawnPlacements) {
        let mut rng = SmallRng::seed_from_u64(self.seed);
        let mut stairs_features = vec![];

        // Collect the coordinates of all current stairs
        for c in &self.stairs {
            stairs_features.push((*c, Tile::Stairs));
        }

        // And feed them to [SpawnPlacements] so that it can use them to find a suitable spawn
        // spot.
        if let Some(point) = placement.place(Tile::Stairs, target, &stairs_features, &self, rng.gen()) {
            self.map[point] = Tile::Stairs;
            self.stairs.push(point);
        }

        // Update the rng
        self.seed = rng.gen();
    }
    /// A convenience function which calls [add_stairs()](struct.Dungeon.html#method.add_stairs) and returns a
    /// [Dungeon](struct.Dungeon.html).
    pub fn with_stairs(mut self, target: Tile, placement: SpawnPlacements) -> Dungeon {
        self.add_stairs(target, placement);
        self
    }
    /// Add a spawn point to the map
    ///
    /// spawn_tile represents the icon of the spawn-point. It is used for later reference and in
    /// spawn algorithms that avoid or seek similar spawn-points.
    pub fn add_spawn_point(&mut self, spawn_type: Tile, target: Tile, placement: SpawnPlacements) {
        let mut rng = SmallRng::seed_from_u64(self.seed);

        // Ask SpawnPlacements for a new point
        if let Some(point) = placement.place(spawn_type, target, &self.spawn_points, &self, rng.gen()) {
            // Check that we do not use that coordinate allready
            if !self.spawn_points.iter().any(|(x, _)| *x == point) {
                // Make sure the target tile is of the tile we want
                if self.map[point] == target {
                    // Store the coordinate for later reference.
                    self.spawn_points.push((point, spawn_type));
                }
            } else {
                // Debug info that we tried to overwrite an allready existing spawn-point
                let t = self.spawn_points.iter().find(|(x, _)| *x == point).map(|(_, t)| t);
                println!("placement of {} -> {} is allready taken by {:?}", spawn_type, point, t);
            }
        }

        // Update the rng.
        self.seed = rng.gen();
    }

    /// A convenience function which calls [add_spawn_point()](struct.Dungeon.html#method.add_spawn_point) and returns a [Dungeon](struct.Dungeon.html).
    pub fn with_spawn_point(mut self, spawn_type: Tile, target: Tile, placement: SpawnPlacements) -> Dungeon {
        self.add_spawn_point(spawn_type, target, placement);
        self
    }

    /// Adds a small 2x2 room into the map adjacent to a corridor and connects to it with a
    /// [Tile::SecretDoor](../utils/tile/enum.Tile.html#SecretDoor)
    pub fn add_secret_room(&mut self) {
        let mut rng = SmallRng::seed_from_u64(self.seed);

        for _ in 0..1000 {
            let placement = SpawnPlacements::RandomCorridor;
            if let Some(point) = placement.place(Tile::Floor, Tile::Floor, &self.spawn_points, &self, rng.gen()) {
                self.seed = rng.gen();

                let mut search_area: [Coord; 8] = [
                    (-1, -3).into(),
                    (0, -3).into(),
                    (-3, -1).into(),
                    (2, -1).into(),
                    (-3, 0).into(),
                    /*(0, 0)*/ (2, 0).into(),
                    (-1, 2).into(),
                    (0, 2).into(),
                ];
                let box_area: [Coord; 4] = [(0, 0).into(), (1, 0).into(), (0, 1).into(), (1, 1).into()];
                let empty_box_area: [Coord; 12] = [
                    (-1, -1).into(),
                    (0, -1).into(),
                    (1, -1).into(),
                    (2, 1).into(),
                    (-1, 0).into(),
                    (2, 0).into(),
                    (-1, 1).into(),
                    (2, 1).into(),
                    (-1, 2).into(),
                    (0, 2).into(),
                    (1, 2).into(),
                    (2, 2).into(),
                ];

                let mut box_area_walled = true;
                let mut empty_area_walled = true;

                search_area.shuffle(&mut rng);
                for c in &search_area {
                    let search_point = point + *c;
                    if self.map[search_point] == Tile::Wall {
                        for p in &box_area {
                            if self.map[search_point + *p] != Tile::Wall {
                                box_area_walled = false;
                                break;
                            }
                        }

                        if box_area_walled {
                            for p in &empty_box_area {
                                if self.map[search_point + *p] != Tile::Wall {
                                    empty_area_walled = false;
                                }
                            }

                            if empty_area_walled {
                                let secret_area = Area::new(search_point, (1, 1).into());
                                let door_mod: [Coord; 8] = [
                                    (0, -1).into(),
                                    (1, -1).into(),
                                    (-1, 0).into(),
                                    (2, 0).into(),
                                    (-1, 1).into(),
                                    (2, 1).into(),
                                    (0, 2).into(),
                                    (1, 2).into(),
                                ];
                                for c in &door_mod {
                                    if point.is_neightbour(search_point + *c) {
                                        self.map[search_point + *c] = Tile::SecretDoor;
                                        for c in secret_area.iter() {
                                            self.map[c] = Tile::Floor;
                                        }
                                        self.secret_rooms.push(secret_area);

                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// See [add_secret_room()]
    pub fn with_secret_room(mut self) -> Self {
        self.add_secret_room();
        self
    }
    fn create_access_point_from(&mut self, other: Coord) -> Option<Coord> {
        self.create_access_point(Dir::get_direction(self.area.center(), other))
    }
    fn create_access_point(&mut self, dir: Dir) -> Option<Coord> {
        let (ap, c_mod, search_area) = match dir {
            Dir::North => (
                Coord::new(self.area.size.x / 2, 1),
                Coord::new(0, 2),
                Area::new((self.area.size.x / 4, 1).into(), (self.area.size.x / 2, self.area.size.y / 2).into()),
            ),
            Dir::South => (
                Coord::new(self.area.size.x / 2, self.area.size.y - 2),
                Coord::new(0, -2),
                Area::new(
                    (self.area.size.x / 4, self.area.size.y - (self.area.size.y / 2)).into(),
                    (self.area.size.x / 2, self.area.size.y / 2).into(),
                ),
            ),
            Dir::West => (
                Coord::new(self.area.size.x - 2, self.area.size.y / 2),
                Coord::new(-2, 0),
                Area::new(
                    (self.area.size.x / 2, self.area.size.y / 4).into(),
                    (self.area.size.x / 2, self.area.size.y / 2).into(),
                ),
            ),
            Dir::East => (
                Coord::new(1, self.area.size.y / 2),
                Coord::new(2, 0),
                Area::new((1, self.area.size.y / 4).into(), (self.area.size.x / 2, self.area.size.y / 2).into()),
            ),
        };

        let mut best = std::f64::MAX;
        let mut apconnect = None;
        for y in search_area.position.y..search_area.position.y + search_area.size.y {
            for x in search_area.position.x..search_area.position.x + search_area.size.x {
                let c = (x, y).into();
                let t: Tile = self.map[c];
                if let MovementCost::Possible(_) = t.is_walkable() {
                    let pyth = (ap + c_mod).real_pyth(c);
                    if pyth < best {
                        best = pyth;
                        apconnect = Some(c);
                    }
                }
            }
        }
        if let Some(apend) = apconnect {
            let mut v = walker_corridor(ap + c_mod, apend);
            v.append(&mut walker_corridor(ap, ap + c_mod));

            for c in v {
                self.map[c] = Tile::Floor;
            }
            Some(ap + self.area.position)
        } else {
            None
        }
    }

    fn add_room(&mut self, room: Area) {
        self.rooms.push(room);
    }

    fn add_corridor(&mut self, corridor: Vec<Coord>) {
        self.corridors.push(corridor);
    }

    /// Returns an iterator of the base map, returning a tuple of ( (x,y), [Tile](../utils/tile/enum.Tile.html)), where [Tile](../utils/tile/enum.Tile.html) is one of:
    ///   * [Tile::Floor](../utils/tile/enum.Tile.html#Floor)
    ///   * [Tile::Wall](../utils/tile/enum.Tile.html#Wall)
    ///   * [Tile::Door](../utils/tile/enum.Tile.html#Door)
    ///   * [Tile::SecretDoor](../utils/tile/enum.Tile.html#SecretDoor)
    ///
    /// Other iterators give access to other created features.
    pub fn iter(&'_ self) -> MapIterator<'_, Tile> {
        MapIterator { pos: 0, size: self.area.size, start: self.area.position, map: &self.map.data }
    }

    /// Similar to [iter()](struct.Dungeon.html#method.iter), with the difference that this iterates over a subset of the total map.
    pub fn area_iter(&'_ self, area: Area) -> Option<MapIterator<'_, Tile>> {
        if self.area.area_within(area) {
            Some(MapIterator { pos: 0, size: area.size, start: area.position, map: &self.map.data })
        } else {
            None
        }
    }

    /// Returns an Iterator of Coords with the position of each stair generated.
    /// Since the stairsway is generated without direction, it is up to he user to decide if it is
    /// an stair going up, down or both directions.
    pub fn stair_iter(&self) -> Iter<Coord> {
        self.stairs.iter()
    }

    /// Returns an Iterator of the tuple (Coord, Tile), which represents each generated spawn point
    /// as generated with [with_spawn_point()](struct.Dungeon.html#method.with_spawn_point) or
    /// [add_spawn_point()](struct.Dungeon.html#method.add_spawn_point), where the
    /// [Tile](../utils/tile/enum.Tile.html) represents the [Tile](../utils/tile/enum.Tile.html) used as an identifier.
    ///
    /// The order of the list is related to the order with which the user called
    /// [with_spawn_point()](struct.Dungeon.html#method.with_spawn_point) or [add_spawn_point()](struct.Dungeon.html#method.with_spawn_point), however it is possible that certain there are
    /// less points generated than were called for, the algorithm was unable to place more spawn
    /// points.
    pub fn spawn_point_iter(&self) -> Iter<(Coord, Tile)> {
        self.spawn_points.iter()
    }

    /// Returns an Iterator of [Area], which represent each generated room. The Area returns is
    /// he position and maximum size of the room. Certain room generators do not use the full
    /// size of the room, this it is very well possible that only a small portion of the room is
    /// actually accessible. It is possible that there are less entries that were called for, the
    /// algorithm was unable to place more rooms.
    pub fn room_iter(&self) -> Iter<Area> {
        self.rooms.iter()
    }

    /// Returns an Iterator of [Area], representing the position and maximum size of the secret rooms.
    /// For now, each secret room has the exact same size, 2x2.
    /// It is possible that there are less entries that were called for, the algorithm was unable
    /// to place more rooms.
    pub fn secret_room_iter(&self) -> Iter<Area> {
        self.secret_rooms.iter()
    }

    /// Returns an Iterator of [Vec<Coord>], representing the position of each tile within a
    /// corridor.
    pub fn corridor_iter(&self) -> Iter<Vec<Coord>> {
        self.corridors.iter()
    }

    /// Imports another dungeon into this dungeon, copying its map into [position,size]
    /// adding it's stairs, corridors, and spawn_points to it's own list.
    pub fn import_dungeon(&mut self, other: &Dungeon) {
        self.rooms.extend(other.rooms.iter());
        self.stairs.extend(other.stairs.iter());
        self.spawn_points.extend(other.spawn_points.iter());
        self.corridors.extend(other.corridors.clone());
        self.add_room(other.area);
    }
}
impl fmt::Display for Dungeon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.area.size.y {
            for x in 0..self.area.size.x {
                let c = Coord::new(x, y);
                let tile = if let Some(tile) = self.spawn_points.iter().find(|(x, _)| *x == c).map(|(_, tile)| tile) {
                    *tile
                } else {
                    self.map[c]
                };

                write!(f, "{}", tile)?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
