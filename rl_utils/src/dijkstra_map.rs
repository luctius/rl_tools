use min_max_macros::max;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Result};
use std::vec::Vec;
use yansi::Paint;

use crate::{Coord, Map, MapMovement, MapObject, MovementCost};

pub fn rgb(minimum: isize, maximum: isize, value: isize) -> (u8, u8, u8) {
    let min = minimum as f32;
    let max = maximum as f32;
    let v = value as f32;
    let ratio = 3.0 * (v - min) / (max - min);
    let b = max!(0, (255.0 * (1.0 - ratio)) as isize);
    let r = max!(0, (255.0 * (ratio - 1.0)) as isize);
    let g = max!(0, 200 - b - r);
    (r as u8, g as u8, b as u8)
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum DijkstraMapValue {
    Goal,
    NonGoal(isize),
    Default,
    Avoid,
    Impassable,
}
impl DijkstraMapValue {
    pub fn to_value(self) -> isize {
        match self {
            DijkstraMapValue::Goal => 0,
            DijkstraMapValue::Default => isize::max_value() / 3,
            DijkstraMapValue::NonGoal(v) => v,
            DijkstraMapValue::Avoid => (isize::max_value() / 2) - 5,
            DijkstraMapValue::Impassable => isize::max_value() - 2,
        }
    }
}
impl MapObject for DijkstraMapValue {
    fn is_transparent(&self) -> bool {
        false
    }
    fn is_walkable(&self) -> MovementCost {
        match self {
            DijkstraMapValue::Impassable => MovementCost::Impossible,
            _ => MovementCost::Possible(1),
        }
    }
}
impl Display for DijkstraMapValue {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let s = match self {
            DijkstraMapValue::Goal => "*",
            DijkstraMapValue::Default => "D",
            DijkstraMapValue::NonGoal(_) => "#",
            DijkstraMapValue::Avoid => "^",
            DijkstraMapValue::Impassable => " ",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DijkstraMap {
    pub map: Map<DijkstraMapValue>,
    goals: Vec<Coord>,
    avoid: Vec<Coord>,
}
impl DijkstraMap {
    pub fn new(size: Coord) -> Self {
        DijkstraMap {
            map: Map::new(size),
            goals: vec![],
            avoid: vec![],
        }
        .blank()
    }
    pub fn with_offset(mut self, offset: Coord) -> Self {
        self.map.area.position = offset;
        self
    }
    pub fn blank(mut self) -> Self {
        self.map.fill(DijkstraMapValue::Default);
        self
    }
    pub fn seed_map<F>(mut self, func: F) -> Self
    where
        F: Fn(Coord) -> DijkstraMapValue,
    {
        for p in self.map.area.iter() {
            self.map[p] = func(p);
            if let DijkstraMapValue::Goal = self.map[p] {
                self.goals.push(p);
            } else if let DijkstraMapValue::Avoid = self.map[p] {
                self.avoid.push(p);
            }
        }
        self
    }
    pub fn with_goal(mut self, c: Coord) -> Self {
        self.map[c] = DijkstraMapValue::Goal;
        self.goals.push(c);
        self
    }
    pub fn calculate(mut self) -> Self {
        let mut queue = VecDeque::new();

        for g in &self.goals {
            queue.push_back(*g);
        }

        while !queue.is_empty() {
            if let Some(current) = queue.pop_front() {
                let cost = self.map[current].to_value();
                match self.map[current] {
                    DijkstraMapValue::NonGoal(_) | DijkstraMapValue::Goal => {
                        for (neighbour, _) in self.map.walkable_tiles(current, MapMovement::Both) {
                            match self.map[neighbour] {
                                DijkstraMapValue::Default | DijkstraMapValue::NonGoal(_) => {
                                    if cost + 1 < self.map[neighbour].to_value() {
                                        self.map[neighbour] = DijkstraMapValue::NonGoal(cost + 1);
                                        queue.push_back(neighbour);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        self
    }
    pub fn merge(mut self, rel_weight: f32, other: &DijkstraMap) -> Self {
        assert!(self.map.area == other.map.area);

        for y in 0..self.map.area.size.y {
            for x in 0..self.map.area.size.x {
                let p: Coord = (x, y).into();
                match other.map[p] {
                    DijkstraMapValue::Default => {}
                    DijkstraMapValue::Impassable => {}
                    DijkstraMapValue::Goal => {
                        match self.map[p] {
                            DijkstraMapValue::Default => self.map[p] = DijkstraMapValue::Goal,
                            DijkstraMapValue::Impassable => self.map[p] = DijkstraMapValue::Goal,
                            DijkstraMapValue::NonGoal(_) => self.map[p] = DijkstraMapValue::Goal,
                            DijkstraMapValue::Goal => self.map[p] = DijkstraMapValue::Goal,
                            DijkstraMapValue::Avoid => self.map[p] = DijkstraMapValue::Avoid, //TODO
                        }
                    }
                    DijkstraMapValue::Avoid => {
                        match self.map[p] {
                            DijkstraMapValue::Default => self.map[p] = DijkstraMapValue::Avoid,
                            DijkstraMapValue::Impassable => self.map[p] = DijkstraMapValue::Avoid,
                            DijkstraMapValue::NonGoal(_) => self.map[p] = DijkstraMapValue::Avoid,
                            DijkstraMapValue::Avoid => self.map[p] = DijkstraMapValue::Avoid,
                            DijkstraMapValue::Goal => self.map[p] = DijkstraMapValue::Avoid, //TODO
                        }
                    }
                    DijkstraMapValue::NonGoal(ocost) => match self.map[p] {
                        DijkstraMapValue::Impassable => (),
                        DijkstraMapValue::Avoid => (),
                        DijkstraMapValue::Goal => (),
                        DijkstraMapValue::Default => {
                            self.map[p] = DijkstraMapValue::NonGoal(
                                (ocost as f32 * rel_weight).round() as isize,
                            )
                        }
                        DijkstraMapValue::NonGoal(scost) => {
                            self.map[p] = DijkstraMapValue::NonGoal(
                                ((scost as f32 + (ocost as f32 * rel_weight)) / 2f32).round()
                                    as isize,
                            )
                        }
                    },
                }
            }
        }
        self
    }
    pub fn invert_with_marge(mut self, percentage: isize) -> Self {
        let mut max_value = 0;
        for p in self.map.area.iter() {
            if let DijkstraMapValue::NonGoal(cost) = self.map[p] {
                if cost > max_value {
                    max_value = cost;
                }
            }
        }

        let perc_cost = (max_value * (100 - percentage)) / 100;

        //for p in self.map.area.iter() {
        for y in 0..self.map.area.size.y {
            for x in 0..self.map.area.size.x {
                let p: Coord = (x, y).into();
                match self.map[p] {
                    DijkstraMapValue::Default => {}
                    DijkstraMapValue::Impassable => {}
                    DijkstraMapValue::Goal => self.map[p] = DijkstraMapValue::Avoid,
                    DijkstraMapValue::Avoid => self.map[p] = DijkstraMapValue::Goal,
                    DijkstraMapValue::NonGoal(cost) => {
                        if cost > perc_cost {
                            self.map[p] = DijkstraMapValue::Goal;
                        } else {
                            self.map[p] = DijkstraMapValue::NonGoal(max_value - cost);
                        }
                    }
                }
            }
        }
        self
    }
    pub fn invert(self) -> Self {
        self.invert_with_marge(0)
    }
}
impl Display for DijkstraMap {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut max = 0;
        for y in 0..self.map.area.size.y {
            for x in 0..self.map.area.size.x {
                if let DijkstraMapValue::NonGoal(cost) = self.map[(x, y)] {
                    if cost > max {
                        max = cost;
                    }
                }
            }
        }
        for y in 0..self.map.area.size.y {
            for x in 0..self.map.area.size.x {
                let tile_str = self.map[(x, y)].to_string();
                let tile = match self.map[(x, y)] {
                    DijkstraMapValue::Goal => Paint::white(tile_str).to_string(),
                    DijkstraMapValue::Default => Paint::white(tile_str).to_string(),
                    DijkstraMapValue::NonGoal(value) => {
                        let (r, g, b) = rgb(0, max, value);
                        Paint::rgb(r, g, b, tile_str).to_string()
                    }
                    DijkstraMapValue::Avoid => Paint::red(tile_str).to_string(),
                    DijkstraMapValue::Impassable => Paint::black(tile_str).to_string(),
                };
                write!(f, "{}", tile)?;
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
}
