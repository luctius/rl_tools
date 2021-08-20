use std::fmt;

use crate::Coord;

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct Area {
    pub position: Coord,
    pub size: Coord,
}
impl Area {
    pub fn new(position: Coord, size: Coord) -> Area {
        Area { position, size }
    }

    pub fn center(&self) -> Coord {
        (
            self.position.x + (self.size.x / 2),
            self.position.y + (self.size.y / 2),
        )
            .into()
    }

    pub fn point_within(&self, point: Coord) -> bool {
        self.position.x <= point.x
            && point.x < self.position.x + self.size.x
            && self.position.y <= point.y
            && point.y < self.position.y + self.size.y
    }

    pub fn area_within(&self, area: Area) -> bool {
        self.position.x <= area.position.x
            && self.position.y <= area.position.y
            && self.position.x + self.size.x >= area.position.x + area.size.x
            && self.position.y + self.size.y >= area.position.y + area.size.y
    }

    pub fn overlaps(&self, other: Area) -> bool {
        self.point_within(other.position)
            || self.point_within(other.position + (other.size.x, 0).into())
            || self.point_within(other.position + (0, other.size.y).into())
            || self.point_within(other.position + other.size)
            || self.point_within(other.center())
            || self.position.x >= other.position.x
                && self.position.y >= other.position.y
                && self.position.x <= other.position.x + other.size.y
                && self.position.y <= other.position.y + other.size.x
            || self.position.x + self.size.y >= other.position.x
                && self.position.y + self.size.x >= other.position.y
                && self.position.x + self.size.y <= other.position.x + other.size.y
                && self.position.y + self.size.x <= other.position.y + other.size.x
    }

    pub fn iter(&self) -> AreaIter {
        AreaIter {
            pos: (0, 0).into(),
            size: self.size,
            position: self.position,
        }
    }
}
impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} -> {}]", self.position, self.position + self.size)
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct AreaIter {
    pub pos: Coord,
    pub size: Coord,
    pub position: Coord,
}
impl Iterator for AreaIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.x <= self.size.x && self.pos.y <= self.size.y {
            if self.pos.x > self.size.x {
                self.pos.x = 0;
                self.pos.y += 1;
            }
            let ret = Some(self.pos + self.position);
            self.pos.x += 1;
            if self.pos.x > self.size.x {
                self.pos.x = 0;
                self.pos.y += 1;
            }
            ret
        } else {
            None
        }
    }
}
