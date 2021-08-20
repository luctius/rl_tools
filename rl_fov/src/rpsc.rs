// modified from my own c-code at: https://github.com/luctius/heresyrl/blob/master/src/fov/rpsc_fov.c

use rl_utils::{Area, Coord};
use std::vec::Vec;

use crate::{utils::Octant, Fov, FovCallbackEnum, FovConfig, Los, VisionShape};

type Angle = usize;
static ANGLE_PERIOD_SHIFT: usize = 0;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct AngleSet(usize, usize, usize);
impl AngleSet {
    fn max() -> Angle {
        // std::usize::MAX
        1000
    }

    // calculate the angles for this cell
    fn from_offset(row: usize, cell: usize) -> AngleSet {
        let range = Self::max() / (row + 1);
        let near = range * cell;
        let far = near + range;
        let middle = near + (range / 2);

        AngleSet(near >> ANGLE_PERIOD_SHIFT, middle >> ANGLE_PERIOD_SHIFT, far >> ANGLE_PERIOD_SHIFT)
    }

    // convert a given center angle to a cell in the given row
    fn to_cell(&self, row: usize) -> Angle {
        self.1 / ((AngleSet::max() / (row + 1)) >> ANGLE_PERIOD_SHIFT)
    }

    fn blocks(&self, set: &AngleSet) -> bool {
        if set.2 < self.0 {
            return false;
        }
        if set.0 > self.2 {
            return false;
        }

        // let ret = if (!blocker) && (set.1 > self.0 && set.1 < self.2) { true }
        // else { set.0 >= self.0 && set.2 <= self.2 };
        let ret = set.1 >= self.0 && set.1 <= self.2;

        // if !ret { println!("-> does {:?} blocks {:?} [blocker: {}] == {}", self, set, blocker, ret); }
        ret
    }

    fn contains(&self, set: &AngleSet) -> bool {
        (self.0 >= set.0 && self.0 <= set.2) || (self.2 <= set.2 && self.2 >= set.0)
    }
}

struct AngleSetList {
    data: Vec<AngleSet>,
}
impl AngleSetList {
    fn new() -> AngleSetList {
        AngleSetList { data: vec![], }
    }

    fn add(&mut self, set: &AngleSet) {
        self.data.push(*set);
    }

    fn is_blocked(&self, set: &AngleSet) -> bool {
        // println!("----");
        for tas in &self.data {
            if tas.blocks(set) {
                return true;
            }
        }
        false
    }
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Rpsc<'a, T, Func>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    pub area:     Area,
    pub radius:   usize,
    pub vision:   VisionShape,
    pub cb_type:  &'a mut T,
    pub callback: Func,
}
impl<'a, T, Func> Rpsc<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn fov_octant(&mut self, src: Coord, octant: Octant) {
        let mut blocked_list = AngleSetList::new();

        for ((row, cell), point_mod) in octant.iter(self.radius).skip(1) {
            let point = src + point_mod;
            if !self.area.point_within(point) {
                continue;
            } else if !self.vision.in_radius(row, cell, self.radius) {
                continue;
            }

            let set = AngleSet::from_offset(row, cell);
            let blocks = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);

            let visible = !blocked_list.is_blocked(&set);

            if blocks {
                blocked_list.add(&set);
            }

            (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(visible));
        }
    }
}
impl<'a, T, Func> FovConfig for Rpsc<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
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
impl<'a, T, Func> Fov for Rpsc<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn fov(&mut self, src: Coord) {
        for octant in Octant::iterator() {
            self.fov_octant(src, *octant);
        }
    }
}
impl<'a, T, Func> Los for Rpsc<'a, T, Func> where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool, {
    fn los(&mut self, src: Coord, dst: Coord) -> bool {
        let mut blocked_list = AngleSetList::new();
        let mut visible = true;

        let delta = src.delta_abs(dst);

        if let Some(octant) = Octant::find_octant(src, dst) {
            let as_dst = AngleSet::from_offset(delta.x as usize, delta.y as usize);

            let distance = src.pyth(dst);

            for row in 1..distance + 1 {
                let mut applied = false;
                let center_cell = as_dst.to_cell(row as usize);
                let cell_select: [isize; 3] = [0, -1, 1];
                let mut row_visible = false;

                for c in 0..=2 {
                    // get the cell we are interrested in.
                    let cell = center_cell as isize + cell_select[c];

                    if cell < 0 {
                        continue;
                    }
                    if cell > row {
                        continue;
                    }

                    let point = octant.calc_point(src, (row, cell).into());
                    if row == distance && point != dst {
                        continue;
                    }
                    if !self.area.point_within(point) {
                        continue;
                    } else if !self.vision.in_radius(row as usize, cell as usize, self.radius) {
                        continue;
                    }

                    let set = AngleSet::from_offset(row as usize, cell as usize);

                    let blocked = blocked_list.is_blocked(&set);
                    if !blocked {
                        row_visible = true;
                        let blocks = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);

                        if blocks {
                            blocked_list.add(&set);

                            if point == dst {
                                visible = false;
                            }
                        } else if point == dst {
                            break;
                        } else if !applied {
                            (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(visible));
                            applied = true;
                        }
                    }
                }

                if !row_visible {
                    visible = false;
                    break;
                }
            }

            visible
        } else {
            false
        }
    }
}
