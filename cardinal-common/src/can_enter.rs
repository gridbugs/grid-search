use crate::step::Step;
use grid_2d::ICoord;

pub trait CanEnter {
    fn can_enter(&self, coord: ICoord) -> bool;
    fn can_step(&self, step: Step) -> bool {
        self.can_enter(step.to_coord)
    }
}
