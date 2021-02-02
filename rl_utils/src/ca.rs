use std::fmt;
use crate::{Area, Coord, Map, MapObject, MovementCost};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum CATile {
    Dead,
    Alive,
}
impl MapObject for CATile {
    fn is_transparent(&self) -> bool {
        match self {
            CATile::Alive => true,
            _ => false,
        }
    }
    fn is_walkable(&self) -> MovementCost {
        match self {
            CATile::Alive => MovementCost::Possible(1),
            CATile::Dead  => MovementCost::Impossible,
        }
    }
}
impl From<CATile> for char {
    fn from(t: CATile) -> Self {
        match t {
            CATile::Alive => '.',
            CATile::Dead  => '#',
        }
    }
}
impl From<char> for CATile {
    fn from(chr: char) -> Self {
        match chr {
            '.' => CATile::Alive,
            '#' => CATile::Dead,
            _   => CATile::Dead,
        }
    }
}
impl fmt::Display for CATile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self {
            CATile::Alive => ".".to_string(),
            CATile::Dead  => "#".to_string(),
        })
    }
}
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub struct CA {
    pub tile: CATile,
    pub next: CATile,
}
impl CA {
    fn calc_wall_sum(pos: Coord, radius: isize, map: &Map<CA>) -> usize {
        let mut sum = 0;

        let mut min = pos - (radius, radius).into();
        if min.x < 0 {
            min.x = 0;
        }
        if min.y < 0 {
            min.y = 0;
        }

        let area = Area::new(min, (radius * 2, radius * 2).into());
        area.iter().for_each(|p| {
            if let Some(ca) = map.get(p) {
                if ca.tile == CATile::Dead {
                    sum += 1;
                }
            }
        });
        sum
    }
    pub fn check(mut self, pos: Coord, params: &CAparams, map: &Map<CA>) -> Self {
        let sum_r1 = Self::calc_wall_sum(pos, 1, map);
        let sum_r2 = Self::calc_wall_sum(pos, 2, map);

        self.next = if sum_r1 >= params.r1 || sum_r2 <= params.r2 { CATile::Dead } else { CATile::Alive };

        self
    }
    pub fn update(mut self) -> Self {
        self.tile = self.next;
        self
    }
}
impl MapObject for CA {
    fn is_transparent(&self) -> bool {
        false
    }
    fn is_walkable(&self) -> MovementCost {
        self.tile.is_walkable()
    }
}
impl From<CA> for CATile {
    fn from(ca: CA) -> Self {
        ca.tile
    }
}
impl From<CA> for String {
    fn from(ca: CA) -> Self {
        ca.tile.to_string()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct CAparams {
    pub count: usize,
    pub r1: usize,
    pub r2: usize,
}

pub fn ca_generate(size: Coord, param: &CAparams, cmap: &mut Map<CA>) {
    let area = Area::new((0, 0).into(), size);
    area.iter().for_each(|p| {
        let ca = cmap[p];
        cmap[p] = ca.check(p, param, &cmap);
    });
    area.iter().for_each(|p| {
        let ca = cmap[p];
        cmap[p] = ca.update();
    });
}
