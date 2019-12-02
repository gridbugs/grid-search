use crate::coord::{CardinalCoord, UnitCoord};
use grid_2d::Coord;

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
