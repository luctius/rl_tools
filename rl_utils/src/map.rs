use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{Index, IndexMut};

use crate::{Area, Coord};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum MovementCost {
    Possible(usize),
    Impossible,
}

pub trait MapObject: PartialEq + Clone + Debug {
    fn is_transparent(&self) -> bool;
    fn is_walkable(&self) -> MovementCost;
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum MapMovement {
    Orthogonal,
    Diagonal,
    Both,
}
impl MapMovement {
    pub fn get_reachable_tiles(self) -> Vec<Coord> {
        let movement_mod: [Coord; 8] = [
            (-1, 0).into(),
            (0, -1).into(),
            (1, 0).into(),
            (0, 1).into(), /* ORTHOGANAL */
            (-1, -1).into(),
            (1, -1).into(),
            (-1, 1).into(),
            (1, 1).into(), /* DIAGONAL   */
        ];
        match self {
            MapMovement::Orthogonal => movement_mod[0..4].to_vec(),
            MapMovement::Diagonal => movement_mod[4..8].to_vec(),
            MapMovement::Both => movement_mod.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map<T>
where
    T: MapObject,
{
    pub area: Area,
    pub data: Vec<T>,
}
impl<T> Map<T>
where
    T: MapObject,
{
    pub fn new(size: Coord) -> Self {
        Map { area: Area::new((0, 0).into(), size), data: Vec::with_capacity((size.x * size.y) as usize) }
    }

    pub fn with_offset(mut self, offset: Coord) -> Self {
        self.area.position = offset;
        self
    }

    pub fn fill(&mut self, filler: T) {
        self.data.truncate(0);
        self.area.iter().for_each(|_p| {
            self.data.push(filler.clone());
        });
    }

    pub fn fill_each<F>(&mut self, mut func: F)
    where
        F: FnMut(Coord) -> T,
    {
        self.data.truncate(0);
        self.area.iter().for_each(|p| {
            self.data.push(func(p));
        });
    }

    pub fn get(&self, pos: Coord) -> Option<&T> {
        self.data.get((pos.y as usize * self.area.size.x as usize) + pos.x as usize)
    }

    pub fn get_mut(&mut self, pos: Coord) -> Option<&mut T> {
        self.data.get_mut((pos.y as usize * self.area.size.x as usize) + pos.x as usize)
    }

    pub fn walkable_tiles(&self, pos: Coord, movement: MapMovement) -> Vec<(Coord, usize)> {
        let mut retvec = vec![];
        for i in movement.get_reachable_tiles() {
            if self.area.point_within(pos + i) {
                if let Some(c) = self.get(pos + i) {
                    match c.is_walkable() {
                        MovementCost::Possible(cost) => {
                            retvec.push((pos + i, cost));
                        }
                        MovementCost::Impossible => (),
                    }
                }
            }
        }
        retvec
    }

    pub fn iter(&'_ self) -> MapIterator<'_, T> {
        MapIterator { pos: 0, size: self.area.size, start: self.area.position, map: self.data.as_slice() }
    }

    pub fn import_from_iter(&mut self, iter: MapIterator<T>) {
        for (xy, t) in iter {
            if !t.is_transparent() {
                if let Some(e) = self.get_mut(xy) {
                    *e = t;
                }
            }
        }
    }
}
impl<T> Display for Map<T>
where
    T: MapObject + Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        for y in 0..self.area.size.y {
            for x in 0..self.area.size.x {
                write!(f, "{}", self[(x, y)].to_string())?;
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
}
impl<T> Index<usize> for Map<T>
where
    T: MapObject,
{
    type Output = T;

    fn index(&self, pos: usize) -> &Self::Output {
        &self.data[pos]
    }
}
impl<T> Index<isize> for Map<T>
where
    T: MapObject,
{
    type Output = T;

    fn index(&self, pos: isize) -> &Self::Output {
        &self.data[pos as usize]
    }
}
impl<T> Index<Coord> for Map<T>
where
    T: MapObject,
{
    type Output = T;

    fn index(&self, pos: Coord) -> &Self::Output {
        assert!(self.area.point_within(pos));
        &self.data[(pos.y as usize * self.area.size.x as usize) + pos.x as usize]
    }
}
impl<T> Index<&Coord> for Map<T>
where
    T: MapObject,
{
    type Output = T;

    fn index(&self, pos: &Coord) -> &Self::Output {
        assert!(self.area.point_within(*pos));
        &self.data[(pos.y as usize * self.area.size.x as usize) + pos.x as usize]
    }
}
impl<T> IndexMut<Coord> for Map<T>
where
    T: MapObject,
{
    fn index_mut(&'_ mut self, pos: Coord) -> &'_ mut Self::Output {
        assert!(self.area.point_within(pos));
        &mut self.data[(pos.y as usize * self.area.size.x as usize) + pos.x as usize]
    }
}
impl<T> IndexMut<&Coord> for Map<T>
where
    T: MapObject,
{
    fn index_mut(&'_ mut self, pos: &Coord) -> &'_ mut Self::Output {
        assert!(self.area.point_within(*pos));
        &mut self.data[(pos.y as usize * self.area.size.x as usize) + pos.x as usize]
    }
}
impl<T> Index<(usize, usize)> for Map<T>
where
    T: MapObject,
{
    type Output = T;

    fn index(&self, pos: (usize, usize)) -> &Self::Output {
        assert!(self.area.point_within(pos.into()));
        &self.data[(pos.1 * self.area.size.x as usize) + pos.0]
    }
}
impl<T> IndexMut<(usize, usize)> for Map<T>
where
    T: MapObject,
{
    fn index_mut(&'_ mut self, pos: (usize, usize)) -> &'_ mut Self::Output {
        assert!(self.area.point_within(pos.into()));
        &mut self.data[(pos.1 * self.area.size.x as usize) + pos.0]
    }
}

impl<T> Index<(isize, isize)> for Map<T>
where
    T: MapObject,
{
    type Output = T;

    fn index(&self, pos: (isize, isize)) -> &Self::Output {
        assert!(self.area.point_within(pos.into()));
        &self.data[(pos.1 as usize * self.area.size.x as usize) + pos.0 as usize]
    }
}
impl<T> IndexMut<(isize, isize)> for Map<T>
where
    T: MapObject,
{
    fn index_mut(&'_ mut self, pos: (isize, isize)) -> &'_ mut Self::Output {
        assert!(self.area.point_within(pos.into()));
        &mut self.data[(pos.1 as usize * self.area.size.x as usize) + pos.0 as usize]
    }
}

/// Iterator which returns a Tuple containing a (Coord, T)
///
/// This is used to retrieve map contents, for example from a [Dungeon].
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct MapIterator<'a, T>
where
    T: Clone,
{
    pub pos: isize,
    pub size: Coord,
    pub start: Coord,
    pub map: &'a [T],
}
impl<'a, T> Iterator for MapIterator<'a, T>
where
    T: Clone,
{
    type Item = (Coord, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.size.x * self.size.y {
            self.pos += 1;
            Some((
                ((((self.pos - 1) % self.size.x) + self.start.x), (((self.pos - 1) / self.size.x) + self.start.y))
                    .into(),
                self.map[(self.pos - 1) as usize].clone(),
            ))
        } else {
            None
        }
    }
}
