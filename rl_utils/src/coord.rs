use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use min_max_macros::max;

use crate::tranthong;

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct Coord {
    pub x: isize,
    pub y: isize,
}
impl Coord {
    pub fn new(x: isize, y: isize) -> Self {
        Coord { x, y }
    }
    pub fn delta(self, b: Coord) -> Coord {
        let a = self;
        a.sub(b)
    }
    pub fn delta_abs(self, b: Coord) -> Coord {
        let mut a = self;
        a.x = (self.x - b.x).abs();
        a.y = (self.y - b.y).abs();
        a
    }
    pub fn delta_abs_total(self, b: Coord) -> isize {
        let t = self.delta_abs(b);
        t.x + t.y
    }
    pub fn equals(self, b: Coord) -> bool {
        self.x == b.x && self.y == b.y
    }
    pub fn is_neightbour(self, b: Coord) -> bool {
        (self.x - b.x).abs() <= 1 && (self.y - b.y).abs() <= 1
    }
    pub fn pyth(self, end: Coord) -> isize {
        max!((self.x - end.x).abs(), (self.y - end.y).abs())
    }
    pub fn real_pyth(self, end: Coord) -> f64 {
        (((self.x - end.x).pow(2) + (self.y - end.y).pow(2)) as f64).sqrt()
    }
    pub fn line(self, end: Coord) -> Vec<Coord> {
        tranthong(self, end)
    }
}
impl Add for Coord {
    type Output = Coord;

    fn add(mut self, o: Coord) -> Coord {
        self.x += o.x;
        self.y += o.y;
        self
    }
}
impl AddAssign for Coord {
    fn add_assign(&mut self, o: Coord) {
        *self = self.add(o);
    }
}
impl Sub for Coord {
    type Output = Coord;

    fn sub(mut self, o: Coord) -> Coord {
        self.x -= o.x;
        self.y -= o.y;
        self
    }
}
impl SubAssign for Coord {
    fn sub_assign(&mut self, o: Coord) {
        *self = self.sub(o);
    }
}
impl From<Coord> for (isize, isize) {
    fn from(pos: Coord) -> Self {
        (pos.x, pos.y)
    }
}
impl From<Coord> for (i32, i32) {
    fn from(pos: Coord) -> Self {
        (pos.x as i32, pos.y as i32)
    }
}
impl From<Coord> for (usize, usize) {
    fn from(pos: Coord) -> Self {
        (pos.x as usize, pos.y as usize)
    }
}
impl From<(usize, usize)> for Coord {
    fn from(pos: (usize, usize)) -> Self {
        Coord { x: pos.0 as isize, y: pos.1 as isize }
    }
}
impl From<(isize, isize)> for Coord {
    fn from(pos: (isize, isize)) -> Self {
        Coord { x: pos.0, y: pos.1 }
    }
}
impl From<(i32, i32)> for Coord {
    fn from(pos: (i32, i32)) -> Self {
        Coord { x: pos.0 as isize, y: pos.1 as isize }
    }
}
impl From<triangulation::geom::Point> for Coord {
    fn from(pos: triangulation::geom::Point) -> Self {
        Coord { x: pos.x as isize, y: pos.y as isize }
    }
}
impl From<Coord> for triangulation::geom::Point {
    fn from(pos: Coord) -> Self {
        triangulation::geom::Point { x: pos.x as f32, y: pos.y as f32 }
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}
