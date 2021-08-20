// Modified from: http://www.adammil.net/blog/v125_roguelike_vision_algorithms.html#shadowcode

use rl_utils::{Area, Coord};

use crate::{utils::Octant, Fov, FovCallbackEnum, FovConfig, Los, VisionShape};

type Slope = Coord;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum Opaque {
    Uninitialised,
    Transparent,
    Opaque,
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct ShadowCasting<'a, T, Func>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    pub symmetric: bool,
    pub area:      Area,
    pub radius:    usize,
    pub vision:    VisionShape,
    pub cb_type:   &'a mut T,
    pub callback:  Func,
}
impl<'a, T, Func> ShadowCasting<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn compute(&mut self, src: Coord, octant: Octant, row: isize, mut top: Slope, mut bottom: Slope) {
        for x in row..=self.radius as isize {
            let mut was_opaque = Opaque::Uninitialised;

            // compute the Y coordinates where the top vector leaves the column (on the right) and where the bottom vector
            // enters the column (on the left). this equals (x+0.5)*top+0.5 and (x-0.5)*bottom+0.5 respectively, which can
            // be computed like (x+0.5)*top+0.5 = (2(x+0.5)*top+1)/2 = ((2x+1)*top+1)/2 to avoid floating point math
            // the rounding is a bit tricky, though
            let top_y = if top.x == 1 { x as isize } else { ((x as isize * 2 + 1) * top.y + top.x - 1) / (top.x * 2) };
            let bottom_y = if bottom.y == 0 { 0 } else { ((x * 2 - 1) * bottom.y + bottom.x) / (bottom.x * 2) };

            for y in (bottom_y..=top_y).rev() {
                let point = octant.calc_point(src, (x, y).into());
                if !self.area.point_within(point) {
                    continue;
                } else if !self.vision.in_radius(x as usize, y as usize, self.radius) {
                    continue;
                }

                // NOTE: use the next line instead if you want the algorithm to be symmetrical
                let visible = if !self.symmetric {
                    true
                } else {
                    (y != top.y || top.y * x >= top.x * y) && (y != bottom.y || bottom.y * x <= bottom.x * y)
                };

                (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(visible));
                let is_opaque = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);

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
impl<'a, T, Func> FovConfig for ShadowCasting<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
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

impl<'a, T, Func> Fov for ShadowCasting<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn fov(&mut self, src: Coord) {
        for octant in Octant::iterator() {
            self.compute(src, *octant, 1, Slope::new(1, 1), Slope::new(1, 0));
        }
    }
}
impl<'a, T, Func> Los for ShadowCasting<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn los(&mut self, src: Coord, dst: Coord) -> bool {
        if let Some(octant) = Octant::find_octant(src, dst) {
            let delta = src.delta_abs(dst);
            let mut top = Slope::new(delta.x, delta.y);
            let mut bottom = Slope::new(delta.x, delta.y);

            let distance = src.pyth(dst) - 1;

            for x in 1..=distance as isize {
                let mut was_opaque = Opaque::Uninitialised;

                // compute the Y coordinates where the top vector leaves the column (on the right) and where the bottom vector
                // enters the column (on the left). this equals (x+0.5)*top+0.5 and (x-0.5)*bottom+0.5 respectively, which can
                // be computed like (x+0.5)*top+0.5 = (2(x+0.5)*top+1)/2 = ((2x+1)*top+1)/2 to avoid floating point math
                // the rounding is a bit tricky, though
                let top_y = if top.x == 1 {
                    x as isize
                } else if top.x > 0 {
                    ((x as isize * 2 + 1) * top.y + top.x - 1) / (top.x * 2)
                } else {
                    0
                };
                let bottom_y = if bottom.y == 0 || bottom.x == 0 {
                    0
                } else {
                    ((x * 2 - 1) * bottom.y + bottom.x) / (bottom.x * 2)
                };
                let mut found = false;

                for y in (bottom_y..=top_y).rev() {
                    let point = octant.calc_point(src, (x, y).into());
                    if !self.area.point_within(point) {
                        continue;
                    } else if !self.vision.in_radius(x as usize, y as usize, self.radius) {
                        continue;
                    }

                    // NOTE: use the next line instead if you want the algorithm to be symmetrical
                    let visible = if !self.symmetric {
                        true
                    } else {
                        (y != top.y || top.y * x >= top.x * y) && (y != bottom.y || bottom.y * x <= bottom.x * y)
                    };

                    (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(visible));
                    let is_opaque = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);

                    if !is_opaque {
                        found = true;
                    }

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

                if !found {
                    return false;
                }

                if was_opaque != Opaque::Transparent {
                    break;
                }
            }
            return true;
        }
        false
    }
}
