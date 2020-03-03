use direction::CardinalDirection;
use grid_2d::Coord;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
pub struct CardinalCoord(Coord);

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
pub struct UnitCoord(CardinalCoord);

pub const UNIT_COORDS: [UnitCoord; 4] = [
    UnitCoord(CardinalCoord(Coord::new(1, 0))),
    UnitCoord(CardinalCoord(Coord::new(0, -1))),
    UnitCoord(CardinalCoord(Coord::new(-1, 0))),
    UnitCoord(CardinalCoord(Coord::new(0, 1))),
];

fn is_cardinal(coord: Coord) -> bool {
    (coord.x == 0 && coord.y != 0) || (coord.y == 0 && coord.x != 0)
}

impl CardinalCoord {
    pub const fn to_coord(self) -> Coord {
        self.0
    }
    pub fn from_coord(coord: Coord) -> Option<Self> {
        if is_cardinal(coord) {
            Some(Self(coord))
        } else {
            None
        }
    }
    pub const fn left90(self) -> Self {
        Self(self.0.left90())
    }
    pub const fn right90(self) -> Self {
        Self(self.0.right90())
    }
    pub const fn left135(self) -> Coord {
        self.0.cardinal_left135()
    }
    pub const fn right135(self) -> Coord {
        self.0.cardinal_right135()
    }
    pub fn to_cardinal_direction(self) -> CardinalDirection {
        match self.0.x.cmp(&0) {
            Ordering::Equal => match self.0.y.cmp(&0) {
                Ordering::Equal => unreachable!(),
                Ordering::Less => CardinalDirection::North,
                Ordering::Greater => CardinalDirection::South,
            },
            Ordering::Less => CardinalDirection::West,
            Ordering::Greater => CardinalDirection::East,
        }
    }
    pub const fn magnitude(self) -> u32 {
        (self.0.x + self.0.y).abs() as u32
    }
    pub fn to_unit_coord(self) -> UnitCoord {
        match self.0.x.cmp(&0) {
            Ordering::Equal => match self.0.y.cmp(&0) {
                Ordering::Equal => unreachable!(),
                Ordering::Less => UnitCoord(Self(Coord::new(0, -1))),
                Ordering::Greater => UnitCoord(Self(Coord::new(0, 1))),
            },
            Ordering::Less => UnitCoord(Self(Coord::new(-1, 0))),
            Ordering::Greater => UnitCoord(Self(Coord::new(1, 0))),
        }
    }
}

impl UnitCoord {
    pub const fn left90(self) -> Self {
        Self(self.0.left90())
    }
    pub const fn right90(self) -> Self {
        Self(self.0.right90())
    }
    pub const fn left135(self) -> Coord {
        self.0.left135()
    }
    pub const fn right135(self) -> Coord {
        self.0.right135()
    }
    pub const fn to_coord(self) -> Coord {
        self.0.to_coord()
    }
    pub const fn to_cardinal_coord(self) -> CardinalCoord {
        self.0
    }
    pub fn to_cardinal_direction(self) -> CardinalDirection {
        self.0.to_cardinal_direction()
    }
    pub fn from_cardinal_direction(cardinal_direction: CardinalDirection) -> Self {
        Self(CardinalCoord(cardinal_direction.coord()))
    }
    pub fn scale(self, by: u32) -> CardinalCoord {
        assert_ne!(by, 0);
        CardinalCoord(self.to_coord() * by as i32)
    }
}
