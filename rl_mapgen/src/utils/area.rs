use min_max_macros::{max, min};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rl_utils::Area;

pub trait AreaGenerator {
    fn generate_room(&self, offset: (isize, isize), size: (isize, isize), max_ratio: f32, seed: u64) -> Option<Area>;
}

impl AreaGenerator for Area {
    fn generate_room(&self, offset: (isize, isize), size: (isize, isize), max_ratio: f32, seed: u64) -> Option<Area> {
        let mut rng = SmallRng::seed_from_u64(seed);

        let min_sz = 5;

        let limit_x = self.position.x + self.size.x - 1;
        let limit_y = self.position.y + self.size.y - 1;

        let pos_x_mod = rng.gen_range(offset.0, offset.1);
        let pos_y_mod = rng.gen_range(offset.0, offset.1);
        let size_x_mod = rng.gen_range(size.0, size.1);
        let size_y_mod = rng.gen_range(size.0, size.1);

        let mut pos_x_mod = ((self.position.x * pos_x_mod) / 100) + 1;
        let mut pos_y_mod = ((self.position.y * pos_y_mod) / 100) + 1;

        let min_room_sz = min!(limit_x - pos_x_mod, limit_y - pos_y_mod);
        let size_x = ((self.size.x - min!(self.size.x, pos_x_mod)) * size_x_mod) / 100;
        let size_y = ((self.size.y - min!(self.size.y, pos_y_mod)) * size_y_mod) / 100;

        let size_x = min!(size_x, min_room_sz);
        let size_y = min!(size_y, min_room_sz);
        let mut size_x = max!(size_x, min_sz);
        let mut size_y = max!(size_y, min_sz);

        if self.position.x + pos_x_mod + size_x > limit_x {
            if size_x > min_sz {
                size_x -= (self.position.x + pos_x_mod + size_x) - limit_x;
            }
            if self.position.x + pos_x_mod + size_x > limit_x {
                pos_x_mod -= (self.position.x + pos_x_mod + size_x) - limit_x;
            }
        }
        if self.position.y + pos_y_mod + size_y > limit_y {
            if size_y > min_sz {
                size_y -= (self.position.y + pos_y_mod + size_y) - limit_y;
            }
            if self.position.y + pos_y_mod + size_y > limit_y {
                pos_y_mod -= (self.position.y + pos_y_mod + size_y) - limit_y;
            }
        }

        if size_x > size_y {
            let ratio = size_x as f32 / size_y as f32;
            if ratio > max_ratio {
                size_x = (max_ratio * size_y as f32) as isize;
            }
        } else {
            let ratio = size_y as f32 / size_x as f32;
            if ratio > max_ratio {
                size_y = (max_ratio * size_x as f32) as isize;
            }
        };

        if pos_x_mod < 0 || pos_y_mod < 0 {
            None
        } else {
            Some(Area { position: self.position + (pos_x_mod, pos_y_mod).into(), size: (size_x, size_y).into() })
        }
    }
}
