use std::vec::Vec;

use rl_utils::{Area, Coord};

pub fn separate_areas(bounds: Area, area_list: &[Area]) -> Vec<Area> {
    let mut retvec = area_list.to_vec();
    let mut separated = true;

    for _ in 0..2000 {
        if !separated {
            break;
        }
        separated = false;

        let area_vec = retvec.clone();
        for (id_a, mut a) in &mut retvec.iter_mut().enumerate() {
            let mut velocity = Coord::new(0, 0);
            for (id_b, b) in area_vec.iter().enumerate() {
                if id_a == id_b {
                    continue;
                };
                if a.overlaps(*b) {
                    velocity += ((a.center().x - b.center().x) / 2, (a.center().y - b.center().y) / 2).into();
                    break;
                }
            }

            if velocity.x != 0 || velocity.y != 0 {
                separated = true;

                if bounds.point_within(a.position + velocity) && bounds.point_within(a.position + a.size + velocity) {
                    a.position.x += velocity.x;
                    a.position.y += velocity.y;
                }
            }
        }
    }

    retvec
}
