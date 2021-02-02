use core::slice::Iter;
use std::fmt;
use std::iter::Iterator;

use rl_utils::Coord;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Octant {
    NNE,
    NEE,

    SEE,
    SSE,

    SSW,
    SWW,

    NWW,
    NNW,
}
impl Octant {
    pub fn find_octant(src: Coord, dst: Coord) -> Option<Self> {
        let delta_abs = src.delta_abs(dst);
        let tflip = delta_abs.x >= delta_abs.y;

        for o in Octant::iterator() {
            let o_mod = o.coord_mod();

            if tflip == o.flip() &&
                src.x + (delta_abs.x * o_mod.x) == dst.x &&
                src.y + (delta_abs.y * o_mod.y) == dst.y {
                return Some(*o);
            }
        }
        None
    }

    fn coord_mod(self) -> Coord {
        match self {
            Octant::NNE => Coord::new(1, -1),
            Octant::NEE => Coord::new(1, -1),

            Octant::SEE => Coord::new(1, 1),
            Octant::SSE => Coord::new(1, 1),

            Octant::SSW => Coord::new(-1, 1),
            Octant::SWW => Coord::new(-1, 1),

            Octant::NWW => Coord::new(-1, -1),
            Octant::NNW => Coord::new(-1, -1),
        }
    }

    pub fn flip(self) -> bool {
        match self {
            Octant::NNE => true,
            Octant::NEE => false,

            Octant::SEE => false,
            Octant::SSE => true,

            Octant::SSW => true,
            Octant::SWW => false,

            Octant::NWW => false,
            Octant::NNW => true,
        }
    }

    pub fn calc_point(self, src: Coord, delta: Coord) -> Coord {
        let point_mod = if self.flip() {
            (delta.x * self.coord_mod().x, delta.y * self.coord_mod().y).into()
        } else {
            (delta.y * self.coord_mod().y, delta.x * self.coord_mod().x).into()
        };

        src + point_mod
    }

    pub fn iterator() -> Iter<'static, Octant> {
        static OCTANTS: [Octant; 8] =
            [Octant::NNE, Octant::NEE, Octant::SEE, Octant::SSE, Octant::SSW, Octant::SWW, Octant::NWW, Octant::NNW];
        OCTANTS.iter()
    }

    pub fn iter(self, radius: usize) -> OctantIter {
        OctantIter { octant: self, radius, row: 0, cell: 0 }
    }
}
impl fmt::Display for Octant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Octant::NNE => write!(f, "north north east"),
            Octant::NEE => write!(f, "north east  east"),

            Octant::SEE => write!(f, "south east  east"),
            Octant::SSE => write!(f, "south south east"),

            Octant::SSW => write!(f, "south south west"),
            Octant::SWW => write!(f, "south west west"),

            Octant::NWW => write!(f, "north west west"),
            Octant::NNW => write!(f, "north north west"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct OctantIter {
    octant: Octant,
    radius: usize,
    row: usize,
    cell: usize,
}
impl Iterator for OctantIter {
    type Item = ((usize, usize), Coord);

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.radius {
            return None;
        }

        let oct_mod = self.octant.coord_mod();
        let point = if !self.octant.flip() {
            Coord::new(self.row as isize * oct_mod.x, self.cell as isize * oct_mod.y)
        } else {
            Coord::new(self.cell as isize * oct_mod.x, self.row as isize * oct_mod.y)
        };
        let tuple = (self.row, self.cell);

        self.cell += 1;

        if self.cell > self.row {
            self.cell = 0;
            self.row += 1;
        }

        Some((tuple, point))
    }
}
