use std::slice::Iter;

use rl_utils::Coord;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Dir {
    North,
    South,
    East,
    West,
}
impl Dir {
    pub fn get_direction(a: Coord, b: Coord) -> Self {
        let delta = a.delta(b);
        let delta_abs = a.delta_abs(b);

        if delta_abs.x > delta_abs.y {
            if delta.x > 0 {
                Dir::East
            } else {
                Dir::West
            }
        } else if delta.y > 0 {
            Dir::North
        } else {
            Dir::South
        }
    }
    pub fn iter() -> Iter<'static, Dir> {
        static DIR: [Dir; 4] = [Dir::North, Dir::South, Dir::East, Dir::West];
        DIR.iter()
    }
    /*
        pub fn get_random_dir<'a>(seed: u64)-> &'a Dir {
            Dir::iterator().nth(SmallRng::seed_from_u64(seed).gen_range(0, 3) as usize).unwrap()
        }
    */
}
