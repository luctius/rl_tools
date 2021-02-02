use rl_utils::{Area, Coord};

use crate::{Cone, Fov, FovCallbackEnum, FovConfig, Los, VisionShape};

#[derive(Clone, PartialEq, Hash, PartialOrd, Debug)]
pub struct TestMap {
    string: String,
    sz:     Coord,
}
impl TestMap {
    pub fn new(s: String) -> Self {
        let mut xend = -1;
        let mut len = 0;
        for (i, c) in s.chars().enumerate() {
            if xend == -1 && c == '\n' {
                xend = i as isize;
            }

            len = i as isize;
        }

        let xsize = xend + 1;
        let size = Coord::new(xsize, len / xsize);

        TestMap { string: s, sz: size, }
    }

    pub fn size(&self) -> Coord {
        self.sz
    }

    pub fn player_pos(&self) -> Option<Coord> {
        let size = self.size();
        for (i, c) in self.string.chars().enumerate() {
            if c == '@' {
                return Some(Coord::new(i as isize % size.x, i as isize / size.x));
            }
        }
        None
    }

    pub fn destination(&self) -> Option<Coord> {
        let size = self.size();
        let mut dst = None;
        for (i, c) in self.string.chars().enumerate() {
            if c == 'x' {
                dst = Some(Coord::new(i as isize % size.x, i as isize / size.x));
            }
        }
        dst
    }

    pub fn area(&self) -> Area {
        Area::new((0, 0).into(), self.size())
    }

    pub fn get(&self, c: Coord) -> Option<&str> {
        let size = self.size();
        let idx = (c.y as usize * size.x as usize) + c.x as usize;
        self.string.get(idx..=idx)
    }

    pub fn is_touched(&self, c: Coord) -> bool {
        let tile = self.get(c);
        if tile == Some(".") {
            true
        } else {
            false
        }
    }

    pub fn fov_func(&mut self, c: Coord, fe: FovCallbackEnum) -> bool {
        let size = self.size();

        let idx = (c.y as usize * size.x as usize) + c.x as usize;
        let tile = self.string.get(idx..=idx);

        if tile == Some("\n") {
            return false;
        }

        let blocking = if tile == Some("#") { true } else { tile == Some("*") };

        match fe {
            FovCallbackEnum::IsBlocked => blocking,
            FovCallbackEnum::SetVisible(visible) => {
                if visible && !blocking {
                    self.string.replace_range(idx..=idx, ".");
                } else if visible && blocking {
                    self.string.replace_range(idx..=idx, "*");
                }
                true
            },
        }
    }

    pub fn los_func(&mut self, c: Coord, fe: FovCallbackEnum) -> bool {
        let size = self.size();

        let idx = (c.y as usize * size.x as usize) + c.x as usize;
        let tile = self.string.get(idx..=idx);

        if tile == Some("\n") {
            return false;
        }

        let blocking = if tile == Some("#") { true } else { tile == Some("*") };

        match fe {
            FovCallbackEnum::IsBlocked => blocking,
            FovCallbackEnum::SetVisible(visible) => {
                if visible && !blocking {
                    self.string.replace_range(idx..=idx, ".");
                } else if visible && blocking {
                    self.string.replace_range(idx..=idx, "*");
                }
                true
            },
        }
    }

    pub fn los_test_func(&mut self, c: Coord, fe: FovCallbackEnum) -> bool {
        let size = self.size();

        if c.x >= size.x - 1 || c.y >= size.y - 1 {
            return true;
        }

        let idx = (c.y as usize * size.x as usize) + c.x as usize;
        let tile = self.string.get(idx..=idx);

        let blocking = if tile == Some("#") { true } else { tile == Some("*") };

        match fe {
            FovCallbackEnum::IsBlocked => blocking,
            FovCallbackEnum::SetVisible(visible) => true,
        }
    }

    pub fn print(&self) {
        println!("map size: {}, player pos: {:?}", self.size(), self.player_pos());
        println!("{}", &self.string);
    }
}
