use crate::Coord;
use std::vec::Vec;

fn difference(start: isize, end: isize) -> (isize, isize) {
    if end >= start {
        (end - start, 1)
    } else {
        (start - end, -1)
    }
}

pub fn tranthong(start: Coord, end: Coord) -> Vec<Coord> {
    let mut retvec = Vec::with_capacity((start.pyth(end) * 2) as usize);

    tranthong_func(start, end, |c| retvec.push(c) );
    retvec
}

pub fn tranthong_func<Func>(start: Coord, end: Coord, mut func: Func) where Func: FnMut(Coord) {
    let (deltax, signdx) = difference(start.x, end.x);
    let (deltay, signdy) = difference(start.y, end.y);

    let mut current = start;
    let mut test: isize = if signdy == -1 { -1 } else { 0 };

    func(current);

    if deltax >= deltay {
        test = (deltax + test) >> 1;
        for _ in 1..deltax {
            test -= deltay;
            current.x += signdx;
            if test < 0 {
                current.y += signdy;
                test += deltax;
            }
            func(current);
        }
    } else {
        test = (deltay + test) >> 1;
        for _ in 1..deltay {
            test -= deltax;
            current.y += signdy;
            if test < 0 {
                current.x += signdx;
                test += deltay;
            }
            func(current);
        }
    }
    func(current);
}
