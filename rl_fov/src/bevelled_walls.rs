// Modified from: http://www.adammil.net/blog/v125_roguelike_vision_algorithms.html#mycode

use rl_utils::{Area, Coord};

use crate::{utils::Octant, Fov, FovCallbackEnum, FovConfig, Los, VisionShape};

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
    }

    pub fn greater_or_equal(&self, s: Slope) -> bool {
        self.y * s.x >= self.x * s.y
    }

    pub fn less_or_equal(&self, s: Slope) -> bool {
        self.y * s.x <= self.x * s.y
    }

    pub fn less(&self, s: Slope) -> bool {
        self.y * s.x < self.x * s.y
    }
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
pub struct BevelledWalls<'a, T, Func>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    pub symmetric: bool,
    pub area:      Area,
    pub radius:    usize,
    pub vision:    VisionShape,
    pub cb_type:   &'a mut T,
    pub callback:  Func,
}
impl<'a, T, Func> BevelledWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn compute(&mut self, src: Coord, octant: Octant, row: isize, mut top: Slope, bottom: Slope) {
        for x in row..=self.radius as isize {
            let mut was_opaque = Opaque::Uninitialised;

            let top_y = if top.x == 1 {
                x as isize
            } else {
                let top_y = ((x * 2 - 1) * top.y + top.x) / (top.x * 2);
                if (self.callback)(self.cb_type, (x, top_y).into(), FovCallbackEnum::IsBlocked) {
                    if top.greater_or_equal((x * 2, top_y * 2 + 1).into())
                       && !(self.callback)(self.cb_type, (x, top_y + 1).into(), FovCallbackEnum::IsBlocked)
                    {
                        top_y + 1
                    } else {
                        top_y
                    }
                } else {
                    top_y
                }
            };

            let bottom_y = if bottom.y == 0 {
                0
            } else {
                let bottom_y = ((x * 2 - 1) * bottom.y + bottom.x) / (bottom.x * 2);
                if bottom.greater_or_equal((bottom_y * 2 + 1, x * 2).into())
                   && (self.callback)(self.cb_type, (x, bottom_y).into(), FovCallbackEnum::IsBlocked)
                   && !(self.callback)(self.cb_type, (x, bottom_y + 1).into(), FovCallbackEnum::IsBlocked)
                {
                    bottom_y + 1
                } else {
                    bottom_y
                }
            };

            for y in (bottom_y..=top_y).rev() {
                let point = octant.calc_point(src, (x, y).into());
                if !self.area.point_within(point) {
                    continue;
                } else if !self.vision.in_radius(x as usize, y as usize, self.radius) {
                    continue;
                }

                let is_opaque = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);

                let is_visible = if !self.symmetric {
                    is_opaque
                    || ((y != top_y || top.greater_or_equal((x, y).into()))
                        && (y != bottom_y || bottom.less_or_equal((x, y).into())))
                } else {
                    (y != top_y || top.greater((x * 4 + 1, y * 4 - 1).into()))
                    && (y != bottom_y || bottom.less((x * 4 - 1, y * 4 + 1).into()))
                };
                (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(is_visible));

                if x != self.radius as isize {
                    if is_opaque {
                        if was_opaque == Opaque::Transparent {
                            let mut nx = x * 2;
                            let ny = y * 2 + 1;

                            if (self.callback)(self.cb_type, (x, y + 1).into(), FovCallbackEnum::IsBlocked) {
                                nx -= 1;
                            }

                            if top.greater((nx, ny).into()) {
                                let bottom = (x * 2 - 1, y * 2 + 1).into();
                                if y == bottom_y {
                                    break;
                                } else {
                                    self.compute(src, octant, x + 1, top, bottom);
                                }
                            } else if y == bottom_y {
                                return;
                            }
                        }

                        was_opaque = Opaque::Opaque;
                    } else {
                        if was_opaque == Opaque::Opaque {
                            let nx =
                                if !self.symmetric
                                   && (self.callback)(self.cb_type, (x + 1, y + 1).into(), FovCallbackEnum::IsBlocked)
                                {
                                    (x * 2) + 1
                                } else {
                                    x * 2
                                };
                            let ny = y * 2 + 1;

                            if bottom.greater_or_equal((nx, ny).into()) {
                                return;
                            }
                            top = (nx, ny).into();
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
impl<'a, T, Func> FovConfig for BevelledWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
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
impl<'a, T, Func> Fov for BevelledWalls<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn fov(&mut self, src: Coord) {
        for octant in Octant::iterator() {
            self.compute(src, *octant, 1, Slope::new(1, 1), Slope::new(1, 0));
        }
    }
}
