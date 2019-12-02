use coord_2d::{Coord, Size};
use direction::CardinalDirection;
pub use grid_search_cardinal_common::path::Path;
use grid_search_cardinal_common::{
    coord::UNIT_COORDS,
    seen_set::{SeenSet, Visit},
    step::Step,
};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub type Depth = u64;

pub struct Context {
    seen_set: SeenSet,
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

pub trait BestSearch {
    fn is_at_max_depth(&self, depth: Depth) -> bool;
    fn can_enter_updating_best(&mut self, coord: Coord) -> bool;
    fn best_coord(&self) -> Option<Coord>;
}

impl Context {
    pub fn new(size: Size) -> Self {
        Self {
            seen_set: SeenSet::new(size),
            queue: VecDeque::new(),
        }
    }

    fn consider<B: BestSearch>(&mut self, best_search: &mut B, step: Step, depth: Depth) {
        if let Some(Visit) = self.seen_set.try_visit_step(step) {
            if best_search.can_enter_updating_best(step.to_coord) {
                if !best_search.is_at_max_depth(depth) {
                    self.queue.push_back((step, depth));
                }
            }
        }
    }

    fn best_search_core<B: BestSearch>(&mut self, best_search: &mut B, start: Coord) {
        self.seen_set.init(start);
        self.queue.clear();
        if !best_search.can_enter_updating_best(start) {
            return;
        }
        if best_search.is_at_max_depth(0) {
            return;
        }
        for &in_direction in &UNIT_COORDS {
            let step = Step {
                to_coord: start + in_direction.to_coord(),
                in_direction,
            };
            self.consider(best_search, step, 1);
        }
        if best_search.is_at_max_depth(1) {
            return;
        }
        while let Some((step, depth)) = self.queue.pop_front() {
            let next_depth = depth + 1;
            self.consider(best_search, step.forward(), next_depth);
            self.consider(best_search, step.left(), next_depth);
            self.consider(best_search, step.right(), next_depth);
        }
    }

    pub fn best_search_path<B: BestSearch>(&mut self, mut best_search: B, start: Coord, path: &mut Path) {
        self.best_search_core(&mut best_search, start);
        let end = best_search.best_coord().unwrap_or(start);
        self.seen_set.build_path_to(end, path);
    }

    pub fn best_search_first<B: BestSearch>(&mut self, mut best_search: B, start: Coord) -> Option<CardinalDirection> {
        self.best_search_core(&mut best_search, start);
        let end = best_search.best_coord().unwrap_or(start);
        self.seen_set.first_direction_towards(end)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use grid_2d::Grid;

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

    fn str_slice_to_test_start_score(str_slice: &[&str], start_score: u8) -> Test {
        let mut test = str_slice_to_test(str_slice);
        *test.grid.get_checked_mut(test.start) = Cell::Traversable(start_score);
        test
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
        ctx.best_search_path(ConstrainedSearch::new(100, &grid), start, &mut path);
        assert_eq!(path.len(), 13);
        ctx.best_search_path(ConstrainedSearch::new(10, &grid), start, &mut path);
        assert_eq!(path.len(), 4);
        ctx.best_search_path(ConstrainedSearch::new(3, &grid), start, &mut path);
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
        ctx.best_search_path(ConstrainedSearch::new(100, &grid), start, &mut path);
        assert_eq!(path.len(), 33);
        ctx.best_search_path(ConstrainedSearch::new(30, &grid), start, &mut path);
        assert_eq!(path.len(), 20);
        ctx.best_search_path(ConstrainedSearch::new(3, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
    }

    const GRID_C: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        ".1........",
        "..........",
        ".@2.......",
        "..........",
    ];

    #[test]
    fn grid_c() {
        let Test { grid, start } = str_slice_to_test(GRID_C);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.best_search_path(ConstrainedSearch::new(100, &grid), start, &mut path);
        assert_eq!(path.len(), 1);
        ctx.best_search_path(ConstrainedSearch::new(2, &grid), start, &mut path);
        assert_eq!(path.len(), 1);
        ctx.best_search_path(ConstrainedSearch::new(0, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
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
        let Test { grid, start } = str_slice_to_test_start_score(GRID_D, 10);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        ctx.best_search_path(ConstrainedSearch::new(100, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
        ctx.best_search_path(ConstrainedSearch::new(2, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
        ctx.best_search_path(ConstrainedSearch::new(0, &grid), start, &mut path);
        assert_eq!(path.len(), 0);
    }
}
