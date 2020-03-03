use crate::coord::{CardinalCoord, UnitCoord};
use grid_2d::Coord;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub to_coord: Coord,
    pub in_direction: UnitCoord,
}

impl Step {
    pub fn forward(&self) -> Self {
        let in_direction = self.in_direction;
        Self {
            to_coord: self.to_coord + in_direction.to_coord(),
            in_direction,
        }
    }
    pub fn left(&self) -> Self {
        let in_direction = self.in_direction.left90();
        Self {
            to_coord: self.to_coord + in_direction.to_coord(),
            in_direction,
        }
    }
    pub fn right(&self) -> Self {
        let in_direction = self.in_direction.right90();
        Self {
            to_coord: self.to_coord + in_direction.to_coord(),
            in_direction,
        }
    }
    pub fn scale_back(&self, by: u32) -> Jump {
        Jump {
            to_coord: self.to_coord,
            in_direction: self.in_direction.scale(by),
        }
    }
    pub fn to_coord(&self) -> Coord {
        self.to_coord
    }
    pub fn from_coord(&self) -> Coord {
        self.to_coord - self.in_direction.to_coord()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Jump {
    pub to_coord: Coord,
    pub in_direction: CardinalCoord,
}

impl Jump {
    pub fn last_step(&self) -> Step {
        Step {
            to_coord: self.to_coord,
            in_direction: self.in_direction.to_unit_coord(),
        }
    }
}
