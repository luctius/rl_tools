// Inspired by: https://github.com/denismr/SymmetricPCVT/blob/master/C%2B%2B/SPCVT.cc

use bitset_core::BitSet;
use std::vec::Vec;

use crate::{utils::Octant, Fov, FovCallbackEnum, FovConfig, Los, VisionShape};
use rl_utils::{tranthong_func, Area, Coord};

const fn nth_triangle_nr(n: usize) -> usize {
    (n * (n + 1)) / 2
}

const fn row_cell_to_index(row: usize, cell: usize) -> usize {
    nth_triangle_nr(row) + cell
}

const fn array_sz(size: usize) -> usize {
    (size + (BITSET_UNIT - 1)) / BITSET_UNIT
}

type BitSetType = u64;
const BITSET_UNIT: usize = BitSetType::BITS as usize;

#[derive(Clone, PartialEq, Hash, PartialOrd, Debug)]
struct PCID<const MAX_RADIUS: usize>
    where [(); array_sz(MAX_RADIUS)]: , {
    ids: [BitSetType; array_sz(MAX_RADIUS)],
}
impl<const MAX_RADIUS: usize> PCID<{ MAX_RADIUS }> where [(); array_sz(MAX_RADIUS)]: , {
    pub const fn new() -> Self {
        Self { ids: [0; array_sz(MAX_RADIUS)], }
    }
}

#[derive(Clone, PartialEq, Hash, PartialOrd, Debug)]
pub struct PCRCbuffer<const MAX_RADIUS: usize>
    where [(); array_sz(MAX_RADIUS)]: , {
    lines:         Vec<PCID<{ MAX_RADIUS }>>,
    default_lines: [BitSetType; array_sz(MAX_RADIUS)],
}
impl<const MAX_RADIUS: usize> PCRCbuffer<{ MAX_RADIUS }> where [(); array_sz(MAX_RADIUS)]: , {
    pub fn generate() -> PCRCbuffer<{ MAX_RADIUS }> {
        Self { lines:         vec![PCID::new(); nth_triangle_nr(MAX_RADIUS)],
               default_lines: [BitSetType::MAX; array_sz(MAX_RADIUS)], }.generate_priv()
    }

    fn generate_priv(mut self) -> PCRCbuffer<{ MAX_RADIUS }> {
        assert!(nth_triangle_nr(MAX_RADIUS - 1) < self.lines.len());

        let start = (0, 0).into();
        for line_id in 0..MAX_RADIUS {
            let current = (MAX_RADIUS - 1, line_id).into();

            tranthong_func(start, current, |c: Coord| {
                let idx = row_cell_to_index(c.x as usize, c.y as usize);

                if let Some(atom) = self.lines.get_mut(idx) {
                    atom.ids.bit_set(line_id);
                }
            });
        }

        self.lines
            .iter()
            .take(MAX_RADIUS - 1)
            .enumerate()
            .for_each(|(i, pcid)| assert_eq!(pcid.ids.bit_none(), false, "failed at {}", i));

        self
    }
}
pub struct PCRC<'a, T, Func, const MAX_RADIUS: usize>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool,
          [(); array_sz(MAX_RADIUS)]: , {
    pub area:     Area,
    pub buffer:   &'a PCRCbuffer<{ MAX_RADIUS }>,
    pub radius:   usize,
    pub vision:   VisionShape,
    pub cb_type:  &'a mut T,
    pub callback: Func,
}
impl<'a, T, Func, const MAX_RADIUS: usize> PCRC<'a, T, Func, MAX_RADIUS>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool,
          [(); array_sz(MAX_RADIUS)]: ,
{
    // pub fn new(buffer: &'a PCRCbuffer, cb_type: &'a mut T, callback: Func) -> PCRC<'a, T, Func> {
    // PCRC {
    // area: Area::new( (0,0).into(), (100,100).into() ),
    // buffer,
    // radius: 20,
    // src: (50,50).into(),
    // vision: VisionShape::Octagon,
    // cb_type,
    // callback,
    // }
    // }
    // pub fn new(area: Area, buffer: &'a PCRCbuffer, radius: usize, src: Coord, vision: VisionShape, cb_type: &'a mut T, callback: Func) -> PCRC<'a, T, Func> {
    // PCRC {
    // area,
    // buffer,
    // radius,
    // src,
    // vision,
    // cb_type,
    // callback,
    // }
    // }
    fn fov_octant(&mut self, src: Coord, octant: Octant) {
        let mut active_lines = self.buffer.default_lines.clone();

        for (((row, cell), point_mod), pcid) in octant.iter(self.radius).zip(&self.buffer.lines).skip(1) {
            let point = src + point_mod;
            if !self.area.point_within(point) {
                continue;
            } else if !self.vision.in_radius(row, cell, self.radius) {
                continue;
            } else if active_lines.bit_none() {
                break;
            }

            let visible = !active_lines.bit_disjoint(&pcid.ids);

            (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(visible));
            if visible && (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked) {
                active_lines.bit_andnot(&pcid.ids);
            }
        }
    }
}
impl<'a, T, Func, const MAX_RADIUS: usize> FovConfig for PCRC<'a, T, Func, MAX_RADIUS>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool,
          [(); array_sz(MAX_RADIUS)]: ,
{
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

impl<'a, T, Func, const MAX_RADIUS: usize> Fov for PCRC<'a, T, Func, MAX_RADIUS>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool,
          [(); array_sz(MAX_RADIUS)]: ,
{
    fn fov(&mut self, src: Coord) {
        assert!(self.radius < MAX_RADIUS);
        for octant in Octant::iterator() {
            self.fov_octant(src, *octant);
        }
    }
}
impl<'a, T, Func, const MAX_RADIUS: usize> Los for PCRC<'a, T, Func, MAX_RADIUS>
    where Func: FnMut(&mut T, Coord, FovCallbackEnum) -> bool,
          [(); array_sz(MAX_RADIUS)]: ,
{
    fn los(&mut self, src: Coord, dst: Coord) -> bool {
        let distance = src.pyth(dst);
        assert!(distance < MAX_RADIUS as isize);

        if let Some(octant) = Octant::find_octant(src, dst) {
            let delta = src.delta_abs(dst);
            let idx = row_cell_to_index(delta.x as usize, delta.y as usize);

            if let Some(dst_pcid) = self.buffer.lines.get(idx) {
                let mut active_lines = dst_pcid.ids.clone();
                let mut cell_start = 0;
                let mut cell_end = 1;
                let mut cell_start_flag = false;

                for row in 0..distance + 1 {
                    let mut found = false;
                    for cell in cell_start..=cell_end {
                        let idx = row_cell_to_index(row as usize, cell as usize);
                        let point = octant.calc_point(src, (row, cell).into());

                        if let Some(pcid) = self.buffer.lines.get(idx) {
                            let visible = !active_lines.bit_disjoint(&pcid.ids);

                            if visible {
                                let blocks = (self.callback)(self.cb_type, point, FovCallbackEnum::IsBlocked);

                                if !blocks {
                                    found = true;
                                    cell_end = cell + 1;
                                    if !cell_start_flag {
                                        cell_start_flag = true;
                                        cell_start = if cell - 1 >= 0 { cell - 1 } else { cell };
                                    }
                                } else {
                                    active_lines.bit_andnot(&pcid.ids);
                                }
                            }
                        }
                    }

                    if active_lines.bit_none() {
                        return false;
                    }
                    assert!(found == true);
                }

                cell_start = 0;
                cell_end = 1;
                for row in 1..distance {
                    let mut found = false;
                    let mut cell_start_flag = false;

                    for cell in cell_start..=cell_end {
                        let point = octant.calc_point(src, (row, cell).into());
                        let idx = row_cell_to_index(row as usize, cell as usize);

                        if let Some(pcid) = self.buffer.lines.get(idx) {
                            let visible = !active_lines.bit_disjoint(&pcid.ids);

                            if visible {
                                found = true;

                                if !cell_start_flag {
                                    cell_start_flag = true;
                                    cell_start = if cell - 1 >= 0 { cell - 1 } else { cell };
                                }

                                cell_end = if cell + 1 <= row + 1 { cell + 1 } else { row };

                                (self.callback)(self.cb_type, point, FovCallbackEnum::SetVisible(true));
                            }
                        }
                    }
                    assert!(found == true);
                }
                return true;
            }
        }
        false
    }
}
