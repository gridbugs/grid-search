use coord_2d::{Coord, Size};
use direction::CardinalDirection;
pub use grid_search_cardinal_common::path::Path;
use grid_search_cardinal_common::{
    seen_set::{SeenSet, Visit},
    step::Step,
    unit_coord::UNIT_COORDS,
};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug)]
struct Node {
    cost: u32,
    cost_plus_heuristic: u32,
    step: Step,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost_plus_heuristic.eq(&other.cost_plus_heuristic)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match other
            .cost_plus_heuristic
            .partial_cmp(&self.cost_plus_heuristic)
        {
            Some(Ordering::Equal) => self.cost.partial_cmp(&other.cost),
            other => other,
        }
    }
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.cost_plus_heuristic.cmp(&self.cost_plus_heuristic) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            other => other,
        }
    }
}

pub trait PointToPointSearch {
    fn can_enter(&self, coord: Coord) -> bool;
}

struct Stop;

pub struct Context {
    seen_set: SeenSet,
    priority_queue: BinaryHeap<Node>,
}

#[cfg(feature = "serialize")]
impl Serialize for Context {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.seen_set.size().serialize(s)
    }
}

#[cfg(feature = "serialize")]
impl<'a> Deserialize<'a> for Context {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(d).map(Self::new)
    }
}

impl Context {
    pub fn new(size: Size) -> Self {
        Self {
            seen_set: SeenSet::new(size),
            priority_queue: BinaryHeap::new(),
        }
    }

    fn consider<P: PointToPointSearch>(
        &mut self,
        point_to_point_search: &P,
        step: Step,
        cost: u32,
        goal: Coord,
    ) -> Option<Stop> {
        if let Some(Visit) = self.seen_set.try_visit(step.clone()) {
            if point_to_point_search.can_enter(step.to_coord) {
                if step.to_coord == goal {
                    return Some(Stop);
                }
                let heuristic = step.to_coord.manhattan_distance(goal);
                let node = Node {
                    cost,
                    cost_plus_heuristic: cost + heuristic,
                    step,
                };
                self.priority_queue.push(node);
            }
        }
        None
    }

    fn point_to_point_search_core<P: PointToPointSearch>(
        &mut self,
        point_to_point_search: &P,
        start: Coord,
        goal: Coord,
    ) {
        self.seen_set.init(start);
        self.priority_queue.clear();
        if start == goal {
            return;
        }
        for &in_direction in &UNIT_COORDS {
            let to_coord = start + in_direction.coord();
            let step = Step {
                to_coord,
                in_direction,
            };
            if let Some(Stop) = self.consider(point_to_point_search, step, 1, goal) {
                return;
            }
        }
        while let Some(Node { cost, step, .. }) = self.priority_queue.pop() {
            let next_cost = cost + 1;
            if let Some(Stop) =
                self.consider(point_to_point_search, step.forward(), next_cost, goal)
            {
                return;
            }
            if let Some(Stop) = self.consider(point_to_point_search, step.left(), next_cost, goal) {
                return;
            }
            if let Some(Stop) = self.consider(point_to_point_search, step.right(), next_cost, goal)
            {
                return;
            }
        }
    }

    pub fn point_to_point_search_path<P: PointToPointSearch>(
        &mut self,
        point_to_point_search: P,
        start: Coord,
        goal: Coord,
        path: &mut Path,
    ) {
        self.point_to_point_search_core(&point_to_point_search, start, goal);
        self.seen_set.build_path_to(goal, path);
    }

    pub fn point_to_point_search_first<P: PointToPointSearch>(
        &mut self,
        point_to_point_search: P,
        start: Coord,
        goal: Coord,
    ) -> Option<CardinalDirection> {
        self.point_to_point_search_core(&point_to_point_search, start, goal);
        self.seen_set.first_direction_towards(goal)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use grid_2d::Grid;

    #[derive(Clone)]
    enum Cell {
        Solid,
        Traversable,
    }

    struct Test {
        grid: Grid<Cell>,
        start: Coord,
        goal: Coord,
    }

    fn str_slice_to_test(str_slice: &[&str]) -> Test {
        let width = str_slice[0].len() as u32;
        let height = str_slice.len() as u32;
        let size = Size::new(width, height);
        let mut grid = Grid::new_clone(size, Cell::Solid);
        let mut start = None;
        let mut goal = None;
        for (y, line) in str_slice.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let cell = match ch {
                    '.' => Cell::Traversable,
                    '@' => {
                        start = Some(coord);
                        Cell::Traversable
                    }
                    '*' => {
                        goal = Some(coord);
                        Cell::Traversable
                    }
                    '#' => Cell::Solid,
                    _ => panic!(),
                };
                *grid.get_checked_mut(coord) = cell;
            }
        }
        Test {
            grid,
            start: start.unwrap(),
            goal: goal.unwrap_or(start.unwrap()),
        }
    }

    struct Search<'a> {
        grid: &'a Grid<Cell>,
    }

    impl<'a> PointToPointSearch for Search<'a> {
        fn can_enter(&self, coord: Coord) -> bool {
            if let Some(cell) = self.grid.get(coord) {
                match cell {
                    Cell::Solid => false,
                    Cell::Traversable => true,
                }
            } else {
                false
            }
        }
    }

    const GRID_A: &[&str] = &[
        "..........",
        ".......*..",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_a() {
        let Test { grid, start, goal } = str_slice_to_test(GRID_A);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.point_to_point_search_path(Search { grid: &grid }, start, goal, &mut path);
        assert_eq!(path.len(), 13);
    }

    const GRID_B: &[&str] = &[
        "..........",
        ".......#..",
        ".......#..",
        "....*..#..",
        "########..",
        "..........",
        "..........",
        "..........",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_b() {
        let Test { grid, start, goal } = str_slice_to_test(GRID_B);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.point_to_point_search_path(Search { grid: &grid }, start, goal, &mut path);
        assert_eq!(path.len(), 22);
    }

    const GRID_C: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        ".@*.......",
        "..........",
    ];

    #[test]
    fn grid_c() {
        let Test { grid, start, goal } = str_slice_to_test(GRID_C);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.point_to_point_search_path(Search { grid: &grid }, start, goal, &mut path);
        assert_eq!(path.len(), 1);
    }

    const GRID_D: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_d() {
        let Test { grid, start, goal } = str_slice_to_test(GRID_D);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.point_to_point_search_path(Search { grid: &grid }, start, goal, &mut path);
        assert_eq!(path.len(), 0);
    }
}
