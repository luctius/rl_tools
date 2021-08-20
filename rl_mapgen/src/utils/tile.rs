use std::fmt;
use yansi::Paint;

use rl_utils::{CATile, MapObject, MovementCost, CA};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum Tile {
    Floor,
    Wall,
    BorderWall,
    Corridor,
    ClosedDoor,
    SecretDoor,
    Stairs,
    Feature(char),
    Transparent,
}
impl Tile {}
impl From<CATile> for Tile {
    fn from(t: CATile) -> Self {
        match t {
            CATile::Alive => Tile::Floor,
            CATile::Dead => Tile::Wall,
        }
    }
}
impl From<Tile> for CATile {
    fn from(t: Tile) -> Self {
        match t {
            Tile::Floor => CATile::Alive,
            Tile::Wall => CATile::Dead,
            _ => CATile::Dead,
        }
    }
}
impl From<CA> for Tile {
    fn from(c: CA) -> Self {
        c.tile.into()
    }
}
impl From<char> for Tile {
    fn from(chr: char) -> Self {
        match chr {
            '.' => Tile::Floor,
            '#' => Tile::Wall,
            '*' => Tile::BorderWall,
            '+' => Tile::ClosedDoor,
            '-' => Tile::SecretDoor,
            '>' => Tile::Stairs,
            ',' => Tile::Corridor,
            ' ' => Tile::Transparent,
            '1' => Tile::Feature(chr),
            '2' => Tile::Feature(chr),
            '3' => Tile::Feature(chr),
            '4' => Tile::Feature(chr),
            '5' => Tile::Feature(chr),
            '6' => Tile::Feature(chr),
            '7' => Tile::Feature(chr),
            '8' => Tile::Feature(chr),
            '9' => Tile::Feature(chr),
            '0' => Tile::Feature(chr),
            'a' => Tile::Feature(chr),
            'b' => Tile::Feature(chr),
            'c' => Tile::Feature(chr),
            'd' => Tile::Feature(chr),
            'e' => Tile::Feature(chr),
            'f' => Tile::Feature(chr),
            'g' => Tile::Feature(chr),
            'h' => Tile::Feature(chr),
            'i' => Tile::Feature(chr),
            'j' => Tile::Feature(chr),
            'k' => Tile::Feature(chr),
            'l' => Tile::Feature(chr),
            'm' => Tile::Feature(chr),
            'n' => Tile::Feature(chr),
            'o' => Tile::Feature(chr),
            'p' => Tile::Feature(chr),
            'q' => Tile::Feature(chr),
            'r' => Tile::Feature(chr),
            's' => Tile::Feature(chr),
            't' => Tile::Feature(chr),
            'u' => Tile::Feature(chr),
            'v' => Tile::Feature(chr),
            'w' => Tile::Feature(chr),
            'x' => Tile::Feature(chr),
            'z' => Tile::Feature(chr),
            'A' => Tile::Feature(chr),
            'B' => Tile::Feature(chr),
            'C' => Tile::Feature(chr),
            'D' => Tile::Feature(chr),
            'E' => Tile::Feature(chr),
            'F' => Tile::Feature(chr),
            'G' => Tile::Feature(chr),
            'H' => Tile::Feature(chr),
            'I' => Tile::Feature(chr),
            'J' => Tile::Feature(chr),
            'K' => Tile::Feature(chr),
            'L' => Tile::Feature(chr),
            'M' => Tile::Feature(chr),
            'N' => Tile::Feature(chr),
            'O' => Tile::Feature(chr),
            'P' => Tile::Feature(chr),
            'Q' => Tile::Feature(chr),
            'R' => Tile::Feature(chr),
            'S' => Tile::Feature(chr),
            'T' => Tile::Feature(chr),
            'U' => Tile::Feature(chr),
            'V' => Tile::Feature(chr),
            'W' => Tile::Feature(chr),
            'X' => Tile::Feature(chr),
            'Z' => Tile::Feature(chr),
            '@' => Tile::Feature(chr),
            _ => Tile::Wall,
        }
    }
}
impl MapObject for Tile {
    fn is_transparent(&self) -> bool {
        match self {
            Tile::Transparent => true,
            _ => false,
        }
    }

    fn is_walkable(&self) -> MovementCost {
        match self {
            Tile::Floor => MovementCost::Possible(1),
            Tile::Corridor => MovementCost::Possible(1),
            Tile::ClosedDoor => MovementCost::Possible(1),
            Tile::Stairs => MovementCost::Possible(1),
            Tile::Feature(_) => MovementCost::Possible(1),
            Tile::SecretDoor => MovementCost::Impossible,
            Tile::Wall => MovementCost::Impossible,
            Tile::BorderWall => MovementCost::Impossible,
            Tile::Transparent => MovementCost::Impossible,
        }
    }
}
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self {
                        Tile::Floor => Paint::white(".").to_string(),
                        Tile::Wall => Paint::fixed(173, "#").to_string(),
                        Tile::BorderWall => Paint::fixed(238, "*").to_string(),
                        Tile::Corridor => Paint::white(",").to_string(),
                        Tile::ClosedDoor => Paint::red("+").to_string(),
                        Tile::SecretDoor => Paint::red("S").to_string(),
                        Tile::Stairs => Paint::fixed(28, ">").to_string(),
                        Tile::Feature(chr) => Paint::fixed(33, chr).to_string(),
                        Tile::Transparent => Paint::black(" ").to_string(),
                    })
    }
}
