use crate::unit_coord::UnitCoord;
use grid_2d::Coord;

#[derive(Clone, Debug)]
pub struct Step {
    pub to_coord: Coord,
    pub in_direction: UnitCoord,
}

impl Step {
    pub fn forward(&self) -> Self {
        let in_direction = self.in_direction;
        Self {
            to_coord: self.to_coord + in_direction.coord(),
            in_direction,
        }
    }
    pub fn left(&self) -> Self {
        let in_direction = self.in_direction.left90();
        Self {
            to_coord: self.to_coord + in_direction.coord(),
            in_direction,
        }
    }
    pub fn right(&self) -> Self {
        let in_direction = self.in_direction.right90();
        Self {
            to_coord: self.to_coord + in_direction.coord(),
            in_direction,
        }
    }
}
