//Modified from: http://www.adammil.net/blog/v125_roguelike_vision_algorithms.html#diamondcode

use rl_utils::{Area, Coord};

use crate::utils::Octant;
use crate::{Los, Fov, FovCallbackEnum, FovConfig, VisionShape};

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Slope {
    x: isize,
    y: isize,
}
impl Slope {
    pub fn new(x: isize, y: isize) -> Slope {
        Slope { x, y }
    }
    pub fn greater(&self, s: Slope) -> bool {
        self.y * s.x > self.x * s.y
    } // this > y/x
    pub fn greater_or_equal(&self, s: Slope) -> bool {
        self.y * s.x >= self.x * s.y
    } // this >= y/x
    pub fn less_or_equal(&self, s: Slope) -> bool {
        self.y * s.x <= self.x * s.y
    } // this <= y/x
}
impl From<(isize, isize)> for Slope {
    fn from(t: (isize, isize)) -> Self {
        Self::new(t.0, t.1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum Opaque {
    Uninitialised,
    Transparent,
    Opaque,
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct DiamondWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool {
    pub symmetric: bool,
    pub area: Area,
    pub radius: usize,
    pub vision: VisionShape,
    pub cb_type: &'a mut T,
    pub callback: Func,
}
impl<'a, T, Func> DiamondWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool {
    fn compute(&mut self, src: Coord, octant: Octant, row: isize, mut top: Slope, mut bottom: Slope) {
        for x in row..=self.radius as isize {
            let mut was_opaque = Opaque::Uninitialised;

            let top_y = if top.x == 1 {
                x
            } else {
                let top_y = ((x * 2 - 1) * top.y + top.x) / (top.x * 2);
                let ay = (top.y * 2 + 1) * top.x;

                if (self.callback)(self.cb_type, (x, top_y).into(), FovCallbackEnum::IsBlocked) {
                    if top.greater_or_equal((x * 2, ay).into()) {
                        top.y + 1
                    } else {
                        top.y
                    }
                } else if top.greater((x * 2 + 1, ay).into()) {
                    top.y + 1
                } else {
                    top.y
                }
            };

            let bottom_y = if bottom.y == 0 { 0 } else { ((x * 2 - 1) * bottom.y + bottom.x) / (bottom.x * 2) };

            for y in (bottom_y..=top_y).rev() {
                let point = octant.calc_point(src, (x, y).into());
                if !self.area.point_within(point) { continue; }
                else if !self.vision.in_radius(x as usize,y as usize, self.radius) { continue; }

                // NOTE: use the next line instead if you want the algorithm to be symmetrical
                if !self.symmetric {
                    (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(true));
                } else if (y != top.y || top.greater_or_equal((x, y).into()))
                    && (y != bottom.y || bottom.less_or_equal((x, y).into()))
                {
                    (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(true));
                }

                // if y == top.y or y == bottom.y, make sure the sector actually intersects the wall tile. if not, don't consider
                // it opaque to prevent the code below from moving the top vector up or the bottom vector down
                let is_opaque = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);
                let is_opaque = if is_opaque {
                    if y == top.y
                        && top.less_or_equal((y * 2 - 1, x * 2).into())
                        && !(self.callback)(self.cb_type, (x, y - 1).into(), FovCallbackEnum::IsBlocked)
                        || y == bottom.y
                            && bottom.greater_or_equal((y * 2 + 1, x * 2).into())
                            && !(self.callback)(self.cb_type, (x, y + 1).into(), FovCallbackEnum::IsBlocked)
                    {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };

                if x != self.radius as isize {
                    if is_opaque {
                        if was_opaque == Opaque::Transparent {
                            let new_bottom = (x * 2 - 1, y * 2 + 1).into();
                            if y == bottom_y {
                                bottom = new_bottom;
                                break;
                            } else {
                                self.compute(src, octant, x + 1, top, new_bottom);
                            }
                        }

                        was_opaque = Opaque::Opaque;
                    } else {
                        // adjust top vector downwards and continue if we found a transition from opaque to clear
                        // (x*2+1, y*2+1) is the top-right corner of the clear tile (i.e. the bottom-right of the opaque tile)
                        if was_opaque == Opaque::Opaque {
                            top = (x * 2 + 1, y * 2 + 1).into();
                        }
                        was_opaque = Opaque::Transparent;
                    }
                }
            }

            if was_opaque != Opaque::Transparent {
                break;
            }
        }
    }
}
impl<'a, T, Func> FovConfig for DiamondWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool {
    fn with_area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }
    fn with_radius(mut self, radius: usize) -> Self {
        self.radius = radius;
        self
    }
    fn with_vision_shape(mut self, vision: VisionShape) -> Self {
        self.vision = vision;
        self
    }
}
impl<'a, T, Func> Fov for DiamondWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool {
    fn fov(&mut self, src: Coord) {
        for octant in Octant::iterator() {
            self.compute(src, *octant, 1, Slope::new(1, 1), Slope::new(1, 0) );
        }
    }
}
