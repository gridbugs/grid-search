use grid_2d::Coord;

pub trait CanEnter {
    fn can_enter(&self, coord: Coord) -> bool;
}
