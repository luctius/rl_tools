use std::vec::Vec;

use rl_utils::Coord;

/// This is the classic roguelike corridor function which seperates any non-straight corridor into
/// a Z corridor.
pub fn walker_corridor(mut start: Coord, mut end: Coord) -> Vec<Coord> {
    let mut retvec = vec![];
    let mut lastvec = vec![];

    let delta_abs = start.delta_abs(end);

    if start.x != end.x && start.y != end.y {
        let (middle1, middle2) = if delta_abs.x > delta_abs.y {
            (
                Coord::new(start.x + (end.delta(start).x / 2), start.y),
                Coord::new(end.x + (start.delta(end).x / 2), end.y),
            )
        } else {
            (
                Coord::new(start.x, start.y + (end.delta(start).y / 2)),
                Coord::new(end.x, end.y + (start.delta(end).y / 2)),
            )
        };

        retvec.append(&mut walker_corridor(start, middle1));
        lastvec.append(&mut walker_corridor(middle2, end));
        start = middle1;
        end = middle2;
    } else {
        retvec.push(start);
    }

    while !start.equals(end) {
        let delta = start.delta(end);
        let delta_abs = start.delta_abs(end);
        let xmod = if delta.x >= 0 { -1 } else { 1 };
        let ymod = if delta.y >= 0 { -1 } else { 1 };

        if delta_abs.x != 0 {
            start.x += xmod;
        } else {
            start.y += ymod;
        }
        retvec.push(start);
    }
    retvec.append(&mut lastvec);

    retvec
}
