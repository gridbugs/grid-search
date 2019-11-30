use grid_2d::{Coord, Grid, Size};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const DIRECTIONS: [Direction; 4] = [
    Direction(Coord::new(0, 1)),
    Direction(Coord::new(1, 0)),
    Direction(Coord::new(0, -1)),
    Direction(Coord::new(-1, 0)),
];

#[derive(Clone, Copy, Debug)]
struct Direction(Coord);

#[derive(Debug)]
struct Step {
    to_coord: Coord,
    in_direction: Direction,
}

impl Step {
    fn forward(&self) -> Self {
        let in_direction = self.in_direction;
        Self {
            to_coord: self.to_coord + in_direction.0,
            in_direction,
        }
    }
    fn left(&self) -> Self {
        let in_direction = Direction(self.in_direction.0.left90());
        Self {
            to_coord: self.to_coord + in_direction.0,
            in_direction,
        }
    }
    fn right(&self) -> Self {
        let in_direction = Direction(self.in_direction.0.right90());
        Self {
            to_coord: self.to_coord + in_direction.0,
            in_direction,
        }
    }
}

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

struct SeenCell {
    count: u64,
    in_direction: Option<Direction>,
}

pub struct Context {
    count: u64,
    seen_set: Grid<SeenCell>,
    priority_queue: BinaryHeap<Node>,
}

pub trait PointToPointSearch {
    fn can_enter(&self, coord: Coord) -> bool;
}

struct Stop;

impl Context {
    pub fn new(size: Size) -> Self {
        Self {
            count: 1,
            seen_set: Grid::new_fn(size, |_| SeenCell {
                count: 0,
                in_direction: None,
            }),
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
        if let Some(cell) = self.seen_set.get_mut(step.to_coord) {
            if cell.count != self.count {
                cell.count = self.count;
                if point_to_point_search.can_enter(step.to_coord) {
                    if step.to_coord == goal {
                        return Some(Stop);
                    }
                    cell.in_direction = Some(step.in_direction);
                    let heuristic = step.to_coord.manhattan_distance(goal);
                    let node = Node {
                        cost,
                        cost_plus_heuristic: cost + heuristic,
                        step,
                    };
                    self.priority_queue.push(node);
                }
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
        self.count += 1;
        self.priority_queue.clear();
        let start_cell = self.seen_set.get_checked_mut(start);
        start_cell.count = self.count;
        start_cell.in_direction = None;
        println!("start: {:?}", start);
        if start == goal {
            return;
        }
        for &in_direction in &DIRECTIONS {
            let to_coord = start + in_direction.0;
            if let Some(cell) = self.seen_set.get_mut(to_coord) {
                if cell.count != self.count {
                    cell.count = self.count;
                    if point_to_point_search.can_enter(to_coord) {
                        cell.in_direction = Some(in_direction);
                    }
                }
            }
            if to_coord == goal {
                return;
            }
            let step = Step {
                to_coord,
                in_direction,
            };
            let heuristic = to_coord.manhattan_distance(goal);
            let cost = 1;
            let node = Node {
                cost,
                cost_plus_heuristic: cost + heuristic,
                step,
            };
            self.priority_queue.push(node);
        }
        while let Some(Node {
            cost,
            step,
            cost_plus_heuristic,
        }) = self.priority_queue.pop()
        {
            println!(
                "expanding {:?}, c: {}, c+h: {}",
                step.to_coord, cost, cost_plus_heuristic
            );
            let next_cost = cost + 1;
            if let Some(Stop) =
                self.consider(point_to_point_search, step.forward(), next_cost, goal)
            {
                return;
            }
            if let Some(Stop) = self.consider(point_to_point_search, step.left(), next_cost, goal) {
                return;
            }
            if let Some(Step) = self.consider(point_to_point_search, step.right(), next_cost, goal)
            {
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            goal: goal.unwrap(),
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

    #[test]
    fn grid_a() {
        let Test { grid, start, goal } = str_slice_to_test(GRID_A);
        let mut ctx = Context::new(grid.size());
        let search = Search { grid: &grid };
        ctx.point_to_point_search_core(&search, start, goal);
    }
}
