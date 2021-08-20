use std::vec::Vec;

use rand::{rngs::SmallRng, Rng, SeedableRng};
use triangulation::{Delaunay, Point};

use rl_utils::{Area, Coord};

/// Generates a multi-room dungeon via the
/// [BSP](http://www.roguebasin.com/index.php?title=Basic_BSP_Dungeon_generation) Algorithm. Then
/// a Delaunay triangulation is generated, of with a percentage of the
/// vertices are used to create corridors.
use crate::corridors::{create_corridor, walker_corridor, CorridorFunction};
use crate::{
    dungeons::{Dungeon, DungeonBuilder, DungeonCombiner, DungeonConfigurer, DungeonParams, Room},
    utils::{
        bsp_tree::{BspTree, NodeDir, Split},
        dirs::Dir,
        AreaGenerator, Tile,
    },
};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
enum SplitDir {
    Horizontal,
    Vertical,
}
impl SplitDir {
    fn get_directions(self) -> (Dir, Dir) {
        match self {
            SplitDir::Horizontal => (Dir::South, Dir::North),
            SplitDir::Vertical => (Dir::West, Dir::East),
        }
    }
}

trait BspSplit {
    fn split(&self, dir: SplitDir, percentage: isize) -> (Area, Area);
}
impl BspSplit for Area {
    // Too tired to fix this abomination for now...
    fn split(&self, dir: SplitDir, percentage: isize) -> (Area, Area) {
        assert!(percentage < 100);

        match dir {
            SplitDir::Horizontal => {
                let yd = (self.size.y * percentage) / 100;
                (Area { position: self.position, size: Coord { x: self.size.x, y: yd, }, },
                 Area { position: Coord { x: self.position.x, y: self.position.y + yd, },
                        size:     Coord { x: self.size.x, y: self.size.y - yd, }, })
            },
            SplitDir::Vertical => {
                let xd = (self.size.x * percentage) / 100;
                (Area { position: self.position, size: Coord { x: xd, y: self.size.y, }, },
                 Area { position: Coord { x: self.position.x + xd, y: self.position.y, },
                        size:     Coord { x: self.size.x - xd, y: self.size.y, }, })
            },
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
struct BspData {
    pub area:       Area,
    pub build_area: Option<Area>,
    pub room:       Option<Dungeon>,
    pub corridor:   Option<Vec<Coord>>,
    pub split_dir:  Option<SplitDir>,
}
impl BspData {
    fn new(area: Area) -> BspData {
        BspData { area, build_area: None, room: None, split_dir: None, corridor: None }
    }

    fn create_room(&mut self,
                   seed: u64,
                   room_offset: (isize, isize),
                   room_size: (isize, isize),
                   max_ratio: f32)
                   -> bool {
        if let Some(room_area) = self.area.generate_room(room_offset, room_size, max_ratio, seed) {
            self.build_area = Some(room_area);
            true
        } else {
            false
        }
    }

    fn can_build_room(&self, builder: &dyn DungeonBuilder) -> bool {
        if let Some(room_area) = self.build_area {
            room_area.size.x >= builder.minimum_size().x && room_area.size.y >= builder.minimum_size().y
        } else {
            false
        }
    }

    fn build_room(&mut self, builder: &dyn DungeonBuilder, seed: u64) -> bool {
        if let Some(room_area) = self.build_area {
            let params = DungeonParams { area: room_area, seed };
            if params.area.size.x >= builder.minimum_size().x && params.area.size.y >= builder.minimum_size().y {
                self.room = Some(builder.generate_with_params(params));
                return true;
            }
        }
        false
    }
}
impl Split for BspData {
    type Context = u64;

    fn split(&mut self, context: &Self::Context) -> Option<(Self, Self)> {
        let mut rng = SmallRng::seed_from_u64(*context);
        let perc = rng.gen_range(30, 70);
        let min_sz = 15;

        if let Some(split_dir) = if self.area.size.x >= min_sz && self.area.size.y >= min_sz {
            if rng.gen_range(0, 100) < 60 {
                Some(SplitDir::Horizontal)
            } else {
                Some(SplitDir::Vertical)
            }
        } else if self.area.size.x >= min_sz {
            Some(SplitDir::Vertical)
        } else if self.area.size.y >= min_sz {
            Some(SplitDir::Horizontal)
        } else {
            None
        } {
            self.split_dir = Some(split_dir);
            let (a1, a2) = self.area.split(split_dir, perc);
            Some((BspData::new(a1), BspData::new(a2)))
        } else {
            None
        }
    }
}

#[allow(missing_debug_implementations)]
#[allow(missing_copy_implementations)]
pub struct Bsp {
    params: DungeonParams,
    depth: usize,
    default_builder: Box<dyn DungeonBuilder>,
    builders: Vec<(isize, Box<dyn DungeonBuilder>)>,
    default_corridor: Box<CorridorFunction>,
    room_offset: (isize, isize),
    room_size: (isize, isize),
    extra_corridor_chance: isize,
    max_ratio: f32,
}
impl DungeonConfigurer for Bsp {
    fn new(size_x: isize, size_y: isize) -> Self {
        Bsp { params: DungeonParams::new(size_x, size_y),
              default_builder: Box::new(Room::new(size_x, size_y)),
              default_corridor: Box::new(walker_corridor),
              builders: vec![],
              depth: 5,
              room_offset: (0, 30),
              room_size: (90, 120),
              extra_corridor_chance: 20,
              max_ratio: 1.7, }
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
impl Bsp {
    pub fn with_depth(mut self, depth: usize) -> Bsp {
        self.depth = depth;
        self
    }

    pub fn with_extra_corridor_chance(mut self, percentage: isize) -> Bsp {
        self.extra_corridor_chance = percentage.abs() % 140;
        self
    }

    pub fn with_room_offset(mut self, minimum: isize, maximum: isize) -> Bsp {
        self.room_offset = (minimum, maximum);
        self
    }

    pub fn with_room_size(mut self, minimum: isize, maximum: isize) -> Bsp {
        self.room_size = (minimum, maximum);
        self
    }

    pub fn with_maximum_room_size_ratio(mut self, max_ratio: f32) -> Bsp {
        self.max_ratio = max_ratio;
        self
    }

    fn generate_nodes(&self, mut bsp: &mut BspTree<BspData>, seed: u64) {
        let mut rng = SmallRng::seed_from_u64(seed);

        for d in 0..self.depth {
            for mut node in bsp.iter(d).collect::<Vec<_>>() {
                node.split(&rng.gen(), &mut bsp);
            }
        }
    }

    fn create_leaf_rooms(&self, bsp: &mut BspTree<BspData>, seed: u64) {
        let mut rng = SmallRng::seed_from_u64(seed);

        for node in bsp.leaf_iter().collect::<Vec<_>>() {
            if let Some(data) = bsp.get_data_mut(node) {
                if data.create_room(rng.gen(), self.room_offset, self.room_size, self.max_ratio) {
                    let mut build = false;
                    let mut perc = rng.gen_range(0, 99);

                    for (percentage, builder) in &self.builders {
                        if data.can_build_room(builder.as_ref()) {
                            perc -= percentage;
                            if perc <= 0 && !build {
                                build = data.build_room(builder.as_ref(), rng.gen());
                                if !build {
                                    perc += percentage;
                                }
                            }
                        }
                    }
                    if !build {
                        data.build_room(self.default_builder.as_ref(), rng.gen());
                    }
                }
            }
        }
    }

    fn connect_leaf_rooms(&self, bsp: &mut BspTree<BspData>) {
        for node in bsp.iter(self.depth - 1).collect::<Vec<_>>() {
            if let Some((dir1, dir2)) = bsp.get_data(node).and_then(|d| d.split_dir.map(|d| d.get_directions())) {
                let ap1 = node.get_child(NodeDir::Left, &bsp).and_then(|child| {
                                                                 bsp.get_data_mut(child)
                        .and_then(|data| data.room.as_mut().and_then(|r| r.create_access_point(dir1)))
                                                             });
                let ap2 = node.get_child(NodeDir::Right, &bsp).and_then(|child| {
                                                                  bsp.get_data_mut(child)
                        .and_then(|data| data.room.as_mut().and_then(|r| r.create_access_point(dir2)))
                                                              });
                if let Some(a) = ap1 {
                    if let Some(b) = ap2 {
                        if let Some(data) = bsp.get_data_mut(node) {
                            data.corridor = Some((self.default_corridor)(a, b));
                        }
                    }
                }
            }
        }
    }

    fn connect_leaf_rooms_with_nephews(&self, bsp: &mut BspTree<BspData>) {
        for d in 0..self.depth - 1 {
            for node in bsp.iter(d).collect::<Vec<_>>() {
                if let Some((dir1, dir2)) = bsp.get_data(node).and_then(|d| d.split_dir.map(|dir| dir.get_directions()))
                {
                    let child1_area = bsp.get_data(node.get_child(NodeDir::Left, &bsp).unwrap()).unwrap().area;
                    let child2_area = bsp.get_data(node.get_child(NodeDir::Right, &bsp).unwrap()).unwrap().area;
                    let mut ap1 = vec![];
                    let mut ap2 = vec![];

                    for n in bsp.leaf_iter().collect::<Vec<_>>() {
                        let area = bsp.get_data(n).unwrap().area;
                        if child1_area.area_within(area) && !child2_area.area_within(area) {
                            if let Some(room) = &mut bsp.get_data_mut(n).as_mut().unwrap().room {
                                ap1.push(room.create_access_point(dir1));
                            }
                        } else if child2_area.area_within(area) && !child1_area.area_within(area) {
                            if let Some(room) = &mut bsp.get_data_mut(n).as_mut().unwrap().room {
                                ap2.push(room.create_access_point(dir2));
                            }
                        }
                    }

                    let mut best = std::f64::MAX;
                    let mut best_ap1 = None;
                    let mut best_ap2 = None;
                    for a1 in &ap1 {
                        for a2 in &ap2 {
                            if let Some(a1) = a1 {
                                if let Some(a2) = a2 {
                                    let pyth = a1.real_pyth(*a2);
                                    if pyth < best {
                                        best = pyth;
                                        best_ap1 = Some(a1);
                                        best_ap2 = Some(a2);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(ap1) = best_ap1 {
                        if let Some(ap2) = best_ap2 {
                            if let Some(data) = bsp.get_data_mut(node) {
                                data.corridor = Some((self.default_corridor)(*ap1, *ap2));
                            }
                        }
                    }
                }
            }
        }
    }
}
impl DungeonCombiner for Bsp {
    fn with_default_builder(mut self, builder: Box<dyn DungeonBuilder>) -> Self
        where Self: Sized, {
        self.default_builder = builder;
        self
    }

    fn with_additional_builder(mut self, percentage: isize, builder: Box<dyn DungeonBuilder>) -> Self
        where Self: Sized, {
        self.builders.push((percentage.abs(), builder));
        self
    }

    fn with_default_corridor(mut self, corridor: Box<CorridorFunction>) -> Self
        where Self: Sized, {
        self.default_corridor = corridor;
        self
    }
}
impl DungeonBuilder for Bsp {
    fn minimum_size(&self) -> Coord {
        Coord::new(30, 30)
    }

    fn get_params(&self) -> DungeonParams {
        self.params
    }

    fn get_name(&self) -> String {
        "Bsp".to_string()
    }

    fn generate_with_params(&self, params: DungeonParams) -> Dungeon {
        let mut rng = SmallRng::seed_from_u64(params.seed);
        let mut output = Dungeon::new(self.get_name(), params);

        let mut bsp = BspTree::new(BspData::new(Area::new((0, 1).into(), params.area.size - (0, 1).into())));

        // Generate BSP Areas
        self.generate_nodes(&mut bsp, rng.gen());

        // For the Leaf (end) nodes, create rooms
        self.create_leaf_rooms(&mut bsp, rng.gen());

        // Connect leaf rooms with sibblings.
        self.connect_leaf_rooms(&mut bsp);

        // Connect leaf rooms with nephews.
        self.connect_leaf_rooms_with_nephews(&mut bsp);

        // Commit rooms to map
        for node in bsp.leaf_iter().collect::<Vec<_>>() {
            if let Some(data) = bsp.get_data(node) {
                if let Some(ref room) = data.room {
                    output.rooms.push(
                        data.build_area
                            .expect("This should not happen, an area with a builder cannot have an empty build_area."),
                    );
                    output.map.import_from_iter(room.iter());
                }
            }
        }

        // Replace transparent areas with walls
        for c in output.map.area.iter() {
            if output.map[c] == Tile::Transparent {
                output.map[c] = Tile::Wall;
            }
        }

        // Create Corridors
        for d in (0..=self.depth).rev() {
            for node in bsp.iter(d).collect::<Vec<_>>() {
                if let Some(data) = bsp.get_data(node) {
                    if let Some(ref corridor) = data.corridor {
                        output.add_corridor(corridor.clone());
                    }
                }
            }
        }

        // we now have an minimal connected dungeon, so we redo the the rtiangulation, bu now we
        // include the minor rooms.
        let mut points: Vec<Point> = vec![];
        let mut ids = vec![];
        for node in bsp.leaf_iter().collect::<Vec<_>>() {
            if let Some(data) = bsp.get_data(node) {
                if let Some(ref room) = data.room {
                    points.push((room.map.area.center() + room.area.position).into());
                    ids.push(node.id);
                }
            }
        }
        let result = Delaunay::new(&points).expect("No triangulation exists.");

        // for a set percentage of the edges, create corridors
        let mut triangles2 = result.dcel.vertices.clone().into_iter();
        triangles2.next();
        for e1 in result.dcel.vertices {
            if let Some(e2) = triangles2.next() {
                if rng.gen_range(0, 100) < self.extra_corridor_chance {
                    let e1_leaf = ids[e1];
                    let e2_leaf = ids[e2];

                    let e1_center = output.rooms[e1].center();
                    let e2_center = output.rooms[e2].center();

                    let mut ap1 = None;
                    let mut ap2 = None;
                    if let Some(room) = &mut bsp.data[e1_leaf].room {
                        ap1 = room.create_access_point_from(e2_center);
                    }
                    if let Some(room) = &mut bsp.data[e2_leaf].room {
                        ap2 = room.create_access_point_from(e1_center);
                    }

                    if let Some(a1) = ap1 {
                        if let Some(a2) = ap2 {
                            output.add_corridor((self.default_corridor)(a1, a2));
                        }
                    }
                }
            }
        }

        for corridor in &output.corridors {
            create_corridor(&corridor, &mut output.map, false);
        }

        for corridor in &output.corridors {
            create_corridor(&corridor, &mut output.map, true);
        }

        output.seed = rng.gen();
        output
    }
}
