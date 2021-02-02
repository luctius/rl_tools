// www.adammil.net/blog/v125_roguelike_vision_algorithms.html
// http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds

#![feature(const_fn)]
#![feature(const_generics)]
//#![feature(min_const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(int_bits_const)]
pub mod utils;

pub mod bevelled_walls;
pub mod diamond_walls;
pub mod precalculated_raycasting;
pub mod rpsc;
pub mod shadow_casting;

use rl_utils::{Area, Coord};

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum FovCallbackEnum {
    IsBlocked,
    SetVisible(bool),
}

pub trait FovConfig {
    fn with_area(self, area: Area) -> Self;
    fn with_radius(self, radius: usize) -> Self;
    fn with_vision_shape(self, vision: VisionShape) -> Self;
}

pub trait Fov: FovConfig {
    fn fov(&mut self, src: Coord);
}

pub trait Los: FovConfig {
    fn los(&mut self, src: Coord, dst: Coord) -> bool;
}

pub trait Cone: FovConfig {
    fn cone(&mut self, src: Coord, dst: Coord, angle: usize);
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum VisionShape {
    Octagon,
    Circle,
    CircleStrict,
    Diamond,
    Square,
}
impl VisionShape {
    const fn in_radius(self, row: usize, cell: usize, radius: usize) -> bool {
        match self {
            VisionShape::Octagon => row + (cell / 2) <= radius,
            VisionShape::Circle => (row * row) + (cell * cell) <= (radius * radius) + radius,
            VisionShape::CircleStrict => (row * row) + (cell * cell) <= radius * radius,
            VisionShape::Diamond => row + cell <= radius - row,
            VisionShape::Square => row <= radius,
        }
    }
}
