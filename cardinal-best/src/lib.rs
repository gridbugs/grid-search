use direction::CardinalDirection;
use grid_2d::{Coord, Grid, Size};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::vec_deque;
use std::collections::VecDeque;

const DIRECTIONS: [Direction; 4] = [
    Direction(Coord::new(0, 1)),
    Direction(Coord::new(1, 0)),
    Direction(Coord::new(0, -1)),
    Direction(Coord::new(-1, 0)),
];

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
struct Direction(Coord);

struct SeenCell {
    count: u64,
    in_direction: Option<Direction>,
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
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

pub type Depth = u64;

pub struct Context {
    count: u64,
    seen_set: Grid<SeenCell>,
    queue: VecDeque<(Step, Depth)>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathNode {
    pub to_coord: Coord,
    pub in_direction: CardinalDirection,
}

pub struct PathIter<'a> {
    iter: vec_deque::Iter<'a, Step>,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathNode;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|step| PathNode {
            to_coord: step.to_coord,
            in_direction: CardinalDirection::from_unit_coord(step.in_direction.0),
        })
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Debug)]
pub struct Path {
    steps: VecDeque<Step>,
}

impl Path {
    pub fn iter(&self) -> PathIter {
        PathIter {
            iter: self.steps.iter(),
        }
    }
    pub fn len(&self) -> usize {
        self.steps.len()
    }
}

pub trait BestSearch {
    fn is_at_max_depth(&self, depth: Depth) -> bool;
    fn can_enter_updating_best(&mut self, coord: Coord) -> bool;
    fn best_coord(&self) -> Option<Coord>;
}

impl Context {
    pub fn new(size: Size) -> Self {
        Self {
            count: 1,
            seen_set: Grid::new_fn(size, |_| SeenCell {
                count: 0,
                in_direction: None,
            }),
            queue: VecDeque::new(),
        }
    }

    fn build_path_to(&self, end: Coord, path: &mut Path) {
        let mut cell = self.seen_set.get(end).expect("path end out of bounds");
        debug_assert_eq!(
            cell.count, self.count,
            "path end not visited in latest search"
        );
        let mut coord = end;
        path.steps.clear();
        while let Some(in_direction) = cell.in_direction {
            let step = Step {
                to_coord: coord,
                in_direction,
            };
            path.steps.push_back(step);
            coord = coord - in_direction.0;
            cell = self.seen_set.get_checked(coord);
            debug_assert_eq!(
                cell.count, self.count,
                "path includes cell not visited in latest search"
            );
        }
    }

    fn first_step_towards(&self, end: Coord) -> Option<Step> {
        let mut cell = self.seen_set.get(end).expect("path end out of bounds");
        debug_assert_eq!(
            cell.count, self.count,
            "path end not visited in latest search"
        );
        let mut coord = end;
        let mut ret = None;
        while let Some(in_direction) = cell.in_direction {
            let step = Step {
                to_coord: coord,
                in_direction,
            };
            coord = coord - in_direction.0;
            cell = self.seen_set.get_checked(coord);
            debug_assert_eq!(
                cell.count, self.count,
                "path includes cell not visited in latest search"
            );
            ret = Some(step);
        }
        ret
    }

    fn consider_best<B: BestSearch>(&mut self, best_search: &mut B, step: Step, depth: Depth) {
        if let Some(seen_cell) = self.seen_set.get_mut(step.to_coord) {
            if seen_cell.count != self.count {
                if best_search.can_enter_updating_best(step.to_coord) {
                    seen_cell.count = self.count;
                    seen_cell.in_direction = Some(step.in_direction);
                    if !best_search.is_at_max_depth(depth) {
                        self.queue.push_back((step, depth));
                    }
                }
            }
        }
    }

    fn best_search_core<B: BestSearch>(&mut self, best_search: &mut B, start: Coord) {
        self.count += 1;
        self.queue.clear();
        let start_cell = self.seen_set.get_checked_mut(start);
        start_cell.count = self.count;
        start_cell.in_direction = None;
        if !best_search.can_enter_updating_best(start) {
            return;
        }
        if best_search.is_at_max_depth(0) {
            return;
        }
        for &in_direction in &DIRECTIONS {
            let step = Step {
                to_coord: start + in_direction.0,
                in_direction,
            };
            self.consider_best(best_search, step, 1);
        }
        if best_search.is_at_max_depth(1) {
            return;
        }
        while let Some((step, depth)) = self.queue.pop_front() {
            let next_depth = depth + 1;
            self.consider_best(best_search, step.forward(), next_depth);
            self.consider_best(best_search, step.left(), next_depth);
            self.consider_best(best_search, step.right(), next_depth);
        }
    }

    pub fn best_search_path<B: BestSearch>(
        &mut self,
        mut best_search: B,
        start: Coord,
        path: &mut Path,
    ) {
        self.best_search_core(&mut best_search, start);
        let end = best_search.best_coord().unwrap_or(start);
        self.build_path_to(end, path);
    }

    pub fn best_search_first<B: BestSearch>(
        &mut self,
        mut best_search: B,
        start: Coord,
    ) -> Option<CardinalDirection> {
        self.best_search_core(&mut best_search, start);
        let end = best_search.best_coord().unwrap_or(start);
        self.first_step_towards(end)
            .map(|step| CardinalDirection::from_unit_coord(step.in_direction.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    enum Cell {
        Solid,
        Traversable(u8),
    }

    struct Test {
        grid: Grid<Cell>,
        start: Coord,
    }

    fn str_slice_to_test(str_slice: &[&str]) -> Test {
        let width = str_slice[0].len() as u32;
        let height = str_slice.len() as u32;
        let size = Size::new(width, height);
        let mut grid = Grid::new_clone(size, Cell::Solid);
        let mut start = None;
        for (y, line) in str_slice.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let cell = match ch {
                    '.' => Cell::Traversable(0),
                    '@' => {
                        start = Some(coord);
                        Cell::Traversable(0)
                    }
                    '1' => Cell::Traversable(1),
                    '2' => Cell::Traversable(2),
                    '#' => Cell::Solid,
                    _ => panic!(),
                };
                *grid.get_checked_mut(coord) = cell;
            }
        }
        Test {
            grid,
            start: start.unwrap(),
        }
    }

    struct ConstrainedSearch<'a> {
        max_depth: Depth,
        world: &'a Grid<Cell>,
        best_coord: Option<Coord>,
        best_score: u8,
    }
    impl<'a> ConstrainedSearch<'a> {
        fn new(max_depth: Depth, world: &'a Grid<Cell>) -> Self {
            Self {
                max_depth,
                world,
                best_coord: None,
                best_score: 0,
            }
        }
    }
    impl<'a> BestSearch for ConstrainedSearch<'a> {
        fn is_at_max_depth(&self, depth: Depth) -> bool {
            depth >= self.max_depth
        }
        fn can_enter_updating_best(&mut self, coord: Coord) -> bool {
            if let Some(&Cell::Traversable(score)) = self.world.get(coord) {
                if self.best_coord.is_none() || score > self.best_score {
                    self.best_score = score;
                    self.best_coord = Some(coord);
                }
                true
            } else {
                false
            }
        }
        fn best_coord(&self) -> Option<Coord> {
            self.best_coord
        }
    }

    const GRID_A: &[&str] = &[
        "..........",
        ".1.....2..",
        "..........",
        "..........",
        "..........",
        "..........",
        "...1......",
        "..........",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_a() {
        let Test { grid, start } = str_slice_to_test(GRID_A);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.best_search_path(&mut ConstrainedSearch::new(100, &grid), start, &mut path);
        assert_eq!(path.len(), 13);
        ctx.best_search_path(&mut ConstrainedSearch::new(10, &grid), start, &mut path);
        assert_eq!(path.len(), 4);
        ctx.best_search_path(&mut ConstrainedSearch::new(3, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
    }

    const GRID_B: &[&str] = &[
        "....#.....",
        ".@........",
        "....#.....",
        "########.#",
        "1......#.#",
        ".....#...#",
        "..########",
        "...#2.....",
        "##.###....",
        "..........",
    ];

    #[test]
    fn grid_b() {
        let Test { grid, start } = str_slice_to_test(GRID_B);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.best_search_path(&mut ConstrainedSearch::new(100, &grid), start, &mut path);
        assert_eq!(path.len(), 33);
        ctx.best_search_path(&mut ConstrainedSearch::new(30, &grid), start, &mut path);
        assert_eq!(path.len(), 20);
        ctx.best_search_path(&mut ConstrainedSearch::new(3, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
    }
}
