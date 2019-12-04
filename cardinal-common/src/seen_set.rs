use crate::coord::CardinalCoord;
use crate::path::Path;
use crate::step::{Jump, Step};
use direction::CardinalDirection;
use grid_2d::{Coord, Grid, Size};

#[derive(Debug, Clone)]
struct SeenCell {
    count: u64,
    cost: u32,
    in_direction: Option<CardinalCoord>,
}

#[derive(Debug, Clone)]
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
                cost: 0,
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
            let mut step = Step {
                to_coord: coord,
                in_direction: in_direction.to_unit_coord(),
            };
            for _ in 0..in_direction.magnitude() {
                path.prepend(step);
                step.to_coord -= step.in_direction.to_coord();
            }
            coord = step.to_coord;
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
            coord = coord - in_direction.to_coord();
            cell = self.grid.get_checked(coord);
            debug_assert_eq!(
                cell.count, self.count,
                "path includes cell not visited in latest search"
            );
            ret = Some(in_direction);
        }
        ret.map(|in_direction| in_direction.to_cardinal_direction())
    }

    pub fn init(&mut self, start: Coord) {
        self.count += 1;
        let cell = self.grid.get_checked_mut(start);
        cell.count = self.count;
        cell.in_direction = None;
    }

    fn try_visit(&mut self, to_coord: Coord, in_direction: CardinalCoord, cost: u32) -> Option<Visit> {
        if let Some(cell) = self.grid.get_mut(to_coord) {
            if cell.count != self.count || cost < cell.cost {
                cell.count = self.count;
                cell.cost = cost;
                cell.in_direction = Some(in_direction);
                return Some(Visit);
            }
        }
        None
    }

    pub fn try_visit_step(&mut self, step: Step, cost: u32) -> Option<Visit> {
        self.try_visit(step.to_coord, step.in_direction.to_cardinal_coord(), cost)
    }

    pub fn try_visit_jump(&mut self, jump: Jump, cost: u32) -> Option<Visit> {
        self.try_visit(jump.to_coord, jump.in_direction, cost)
    }
}
