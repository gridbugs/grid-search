use direction::CardinalDirection;
use grid_2d::Coord;

#[derive(Clone, Copy, Debug)]
pub struct UnitCoord(Coord);
pub const UNIT_COORDS: [UnitCoord; 4] = [
    UnitCoord(Coord::new(0, 1)),
    UnitCoord(Coord::new(1, 0)),
    UnitCoord(Coord::new(0, -1)),
    UnitCoord(Coord::new(-1, 0)),
];

impl UnitCoord {
    pub fn coord(self) -> Coord {
        self.0
    }
    pub fn left90(self) -> UnitCoord {
        UnitCoord(self.0.left90())
    }
    pub fn right90(self) -> UnitCoord {
        UnitCoord(self.0.right90())
    }
    pub fn to_cardinal_direction(self) -> CardinalDirection {
        CardinalDirection::from_unit_coord(self.0)
    }
}
