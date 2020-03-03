use crate::step::Step;
use grid_2d::Coord;

pub trait CanEnter {
    fn can_enter(&self, coord: Coord) -> bool;
    fn can_step(&self, step: Step) -> bool {
        self.can_enter(step.to_coord)
    }
}
