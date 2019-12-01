use crate::path::Path;
use crate::step::Step;
use crate::unit_coord::UnitCoord;
use direction::CardinalDirection;
use grid_2d::{Coord, Grid, Size};

struct SeenCell {
    count: u64,
    in_direction: Option<UnitCoord>,
}

pub struct SeenSet {
    count: u64,
    grid: Grid<SeenCell>,
}

pub struct Visit;

impl SeenSet {
    pub fn new(size: Size) -> Self {
        Self {
            count: 1,
            grid: Grid::new_fn(size, |_| SeenCell {
                count: 0,
                in_direction: None,
            }),
        }
    }

    pub fn size(&self) -> Size {
        self.grid.size()
    }

    pub fn build_path_to(&self, end: Coord, path: &mut Path) {
        let mut cell = self.grid.get(end).expect("path end out of bounds");
        debug_assert_eq!(cell.count, self.count, "path end not visited in latest search");
        let mut coord = end;
        path.clear();
        while let Some(in_direction) = cell.in_direction {
            let step = Step {
                to_coord: coord,
                in_direction,
            };
            path.prepend(step);
            coord = coord - in_direction.coord();
            cell = self.grid.get_checked(coord);
            debug_assert_eq!(
                cell.count, self.count,
                "path includes cell not visited in latest search"
            );
        }
    }

    pub fn first_direction_towards(&self, end: Coord) -> Option<CardinalDirection> {
        let mut cell = self.grid.get(end).expect("path end out of bounds");
        debug_assert_eq!(cell.count, self.count, "path end not visited in latest search");
        let mut coord = end;
        let mut ret = None;
        while let Some(in_direction) = cell.in_direction {
            let step = Step {
                to_coord: coord,
                in_direction,
            };
            coord = coord - in_direction.coord();
            cell = self.grid.get_checked(coord);
            debug_assert_eq!(
                cell.count, self.count,
                "path includes cell not visited in latest search"
            );
            ret = Some(step);
        }
        ret.map(|step| step.in_direction.to_cardinal_direction())
    }

    pub fn init(&mut self, start: Coord) {
        self.count += 1;
        let cell = self.grid.get_checked_mut(start);
        cell.count = self.count;
        cell.in_direction = None;
    }

    pub fn try_visit(&mut self, step: Step) -> Option<Visit> {
        if let Some(cell) = self.grid.get_mut(step.to_coord) {
            if cell.count != self.count {
                cell.count = self.count;
                cell.in_direction = Some(step.in_direction);
                return Some(Visit);
            }
        }
        None
    }
}
