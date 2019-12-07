pub use coord_2d::{Coord, Size};
pub use direction::CardinalDirection;
pub use grid_search_cardinal_common::path::Path;
use grid_search_cardinal_common::{
    coord::UNIT_COORDS,
    seen_set::{SeenSet, Visit},
    step::{Jump, Step},
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
        match other.cost_plus_heuristic.partial_cmp(&self.cost_plus_heuristic) {
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

trait Profiler {
    fn expand(&mut self);
}

impl Profiler for () {
    fn expand(&mut self) {}
}

#[derive(Default, Debug)]
pub struct Profile {
    expand: u64,
}

impl Profiler for Profile {
    fn expand(&mut self) {
        self.expand += 1;
    }
}

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

pub mod expand {
    use super::private_expand::PrivateExpand;
    pub trait Expand: PrivateExpand {}

    #[derive(Debug, Clone, Copy)]
    pub struct JumpPoint;

    #[derive(Debug, Clone, Copy)]
    pub struct Sequential;

    impl Expand for JumpPoint {}
    impl Expand for Sequential {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoPath;

mod private_expand {
    use super::{expand, Context, Coord, PointToPointSearch, Step};
    pub struct Stop;
    pub trait PrivateExpand {
        fn consider<P: PointToPointSearch>(
            context: &mut Context,
            point_to_point_search: &P,
            step: Step,
            cost: u32,
            goal: Coord,
        ) -> Option<Stop>;
        fn expand<P: PointToPointSearch>(
            context: &mut Context,
            point_to_point_search: &P,
            step: Step,
            cost: u32,
            goal: Coord,
        ) -> Option<Stop>;
    }

    impl PrivateExpand for expand::JumpPoint {
        fn consider<P: PointToPointSearch>(
            context: &mut Context,
            point_to_point_search: &P,
            step: Step,
            cost: u32,
            goal: Coord,
        ) -> Option<Stop> {
            context.consider_jps(point_to_point_search, step, cost, goal)
        }

        fn expand<P: PointToPointSearch>(
            context: &mut Context,
            point_to_point_search: &P,
            step: Step,
            cost: u32,
            goal: Coord,
        ) -> Option<Stop> {
            if let Some(Stop) = Self::consider(context, point_to_point_search, step.forward(), cost, goal) {
                return Some(Stop);
            }
            if let Some(Stop) = Self::consider(context, point_to_point_search, step.left(), cost, goal) {
                return Some(Stop);
            }
            if let Some(Stop) = Self::consider(context, point_to_point_search, step.right(), cost, goal) {
                return Some(Stop);
            }
            None
        }
    }

    impl PrivateExpand for expand::Sequential {
        fn consider<P: PointToPointSearch>(
            context: &mut Context,
            point_to_point_search: &P,
            step: Step,
            cost: u32,
            goal: Coord,
        ) -> Option<Stop> {
            context.consider(point_to_point_search, step, cost, goal)
        }

        fn expand<P: PointToPointSearch>(
            context: &mut Context,
            point_to_point_search: &P,
            step: Step,
            cost: u32,
            goal: Coord,
        ) -> Option<Stop> {
            if let Some(Stop) = Self::consider(context, point_to_point_search, step.forward(), cost, goal) {
                return Some(Stop);
            }
            if let Some(Stop) = Self::consider(context, point_to_point_search, step.left(), cost, goal) {
                return Some(Stop);
            }
            if let Some(Stop) = Self::consider(context, point_to_point_search, step.right(), cost, goal) {
                return Some(Stop);
            }
            None
        }
    }
}

use expand::Expand;
use private_expand::Stop;

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
        let cost = cost + 1;
        if let Some(Visit) = self.seen_set.try_visit_step(step, cost) {
            if step.to_coord == goal {
                return Some(Stop);
            }
            if point_to_point_search.can_enter(step.to_coord) {
                let heuristic = step.to_coord.manhattan_distance(goal);
                let cost_plus_heuristic = cost + heuristic;
                let node = Node {
                    cost,
                    cost_plus_heuristic,
                    step,
                };
                self.priority_queue.push(node);
            }
        }
        None
    }

    fn consider_jps<P: PointToPointSearch>(
        &mut self,
        point_to_point_search: &P,
        mut step: Step,
        cost: u32,
        goal: Coord,
    ) -> Option<Stop> {
        let mut jump_cost = 1;
        'outer: loop {
            if step.to_coord == goal {
                let jump = Jump {
                    in_direction: step.in_direction.scale(jump_cost),
                    to_coord: goal,
                };
                self.seen_set.try_visit_jump(jump, cost + jump_cost);
                return Some(Stop);
            }
            if !point_to_point_search.can_enter(step.to_coord) {
                return None;
            }
            if has_forced_neighbour(point_to_point_search, step, goal) {
                break;
            }
            // explore to the left only
            let mut side_step = step.left();
            let mut side_jump_cost = 1;
            'inner: loop {
                if side_step.to_coord == goal {
                    let jump_to_intermediate = Jump {
                        in_direction: step.in_direction.scale(jump_cost),
                        to_coord: step.to_coord,
                    };
                    let jump_to_goal = Jump {
                        in_direction: side_step.in_direction.scale(side_jump_cost),
                        to_coord: goal,
                    };
                    self.seen_set.try_visit_jump(jump_to_intermediate, cost + jump_cost);
                    self.seen_set
                        .try_visit_jump(jump_to_goal, cost + jump_cost + side_jump_cost);
                    return Some(Stop);
                }
                if !point_to_point_search.can_enter(side_step.to_coord) {
                    break 'inner;
                }
                if has_forced_neighbour(point_to_point_search, side_step, goal) {
                    let jump_to_side_jump_point = Jump {
                        in_direction: side_step.in_direction.scale(side_jump_cost),
                        to_coord: side_step.to_coord,
                    };
                    if let Some(Visit) = self
                        .seen_set
                        .try_visit_jump(jump_to_side_jump_point, cost + jump_cost + side_jump_cost)
                    {
                        let heuristic = side_step.to_coord.manhattan_distance(goal);
                        let cost = cost + jump_cost + side_jump_cost;
                        let node = Node {
                            cost,
                            cost_plus_heuristic: cost + heuristic,
                            step: side_step,
                        };
                        self.priority_queue.push(node);
                    }
                    break 'outer;
                }
                side_step = side_step.forward();
                side_jump_cost += 1;
            }
            step = step.forward();
            jump_cost += 1;
        }
        let jump = step.scale_back(jump_cost);
        let cost = cost + jump_cost;
        if let Some(Visit) = self.seen_set.try_visit_jump(jump, cost) {
            let heuristic = step.to_coord.manhattan_distance(goal);
            let node = Node {
                cost,
                cost_plus_heuristic: cost + heuristic,
                step,
            };
            self.priority_queue.push(node);
        }
        None
    }

    fn point_to_point_search_core<S, E, P>(
        &mut self,
        point_to_point_search: &S,
        start: Coord,
        goal: Coord,
        profiler: &mut P,
    ) -> Result<(), NoPath>
    where
        S: PointToPointSearch,
        E: Expand,
        P: Profiler,
    {
        self.seen_set.init(start);
        self.priority_queue.clear();
        if start == goal {
            return Ok(());
        }
        for &in_direction in &UNIT_COORDS {
            let to_coord = start + in_direction.to_coord();
            let step = Step { to_coord, in_direction };
            if let Some(Stop) = E::consider(self, point_to_point_search, step, 1, goal) {
                return Ok(());
            }
        }
        while let Some(Node { cost, step, .. }) = self.priority_queue.pop() {
            profiler.expand();
            if let Some(Stop) = E::expand(self, point_to_point_search, step, cost, goal) {
                return Ok(());
            }
        }
        Err(NoPath)
    }

    pub fn point_to_point_search_path<S, E>(
        &mut self,
        expand: E,
        point_to_point_search: S,
        start: Coord,
        goal: Coord,
        path: &mut Path,
    ) -> Result<(), NoPath>
    where
        S: PointToPointSearch,
        E: Expand,
    {
        let _ = expand;
        self.point_to_point_search_core::<_, E, _>(&point_to_point_search, start, goal, &mut ())?;
        self.seen_set.build_path_to(goal, path);
        Ok(())
    }

    pub fn point_to_point_search_first<S, E>(
        &mut self,
        expand: E,
        point_to_point_search: S,
        start: Coord,
        goal: Coord,
    ) -> Result<Option<CardinalDirection>, NoPath>
    where
        S: PointToPointSearch,
        E: Expand,
    {
        let _ = expand;
        self.point_to_point_search_core::<_, E, _>(&point_to_point_search, start, goal, &mut ())?;
        Ok(self.seen_set.first_direction_towards(goal))
    }

    pub fn point_to_point_search_profile<S, E>(
        &mut self,
        expand: E,
        point_to_point_search: S,
        start: Coord,
        goal: Coord,
    ) -> (Profile, Result<(), NoPath>)
    where
        S: PointToPointSearch,
        E: Expand,
    {
        let _ = expand;
        let mut profile = Profile::default();
        let result = self.point_to_point_search_core::<_, E, _>(&point_to_point_search, start, goal, &mut profile);
        (profile, result)
    }
}

fn has_forced_neighbour<P: PointToPointSearch>(point_to_point_search: &P, step: Step, goal: Coord) -> bool {
    (!point_to_point_search.can_enter(step.to_coord + step.in_direction.left135())
        && (point_to_point_search.can_enter(step.to_coord + step.in_direction.left90().to_coord())
            || step.to_coord + step.in_direction.left90().to_coord() == goal))
        || (!point_to_point_search.can_enter(step.to_coord + step.in_direction.right135())
            && (point_to_point_search.can_enter(step.to_coord + step.in_direction.right90().to_coord())
                || step.to_coord + step.in_direction.right90().to_coord() == goal))
}

#[cfg(test)]
mod test {
    use super::*;
    use grid_2d::Grid;
    use rand::{Rng, SeedableRng};
    use rand_isaac::Isaac64Rng;

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

    fn print_test(test: &Test) {
        for row in test.grid.rows() {
            print!("\"");
            for cell in row {
                let ch = match cell {
                    Cell::Solid => '#',
                    Cell::Traversable => '.',
                };
                print!("{}", ch);
            }
            println!("\",");
        }
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
                    _ => Cell::Traversable,
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

    fn random_test<R: Rng>(size: Size, rng: &mut R) -> Test {
        let mut grid = Grid::new_clone(size, Cell::Traversable);
        let num_solid = size.count() / 4;
        for _ in 0..num_solid {
            let coord = Coord::random_within(size, rng);
            *grid.get_checked_mut(coord) = Cell::Solid;
        }
        let start = Coord::new(0, 0);
        let goal = size.to_coord().unwrap() - Coord::new(1, 1);
        Test { grid, start, goal }
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

    fn test(grid_str_slice: &[&str], len: Option<usize>) {
        let Test { grid, start, goal } = str_slice_to_test(grid_str_slice);
        let mut ctx = Context::new(grid.size());
        let mut path = Path::default();
        match len {
            Some(len) => {
                ctx.point_to_point_search_path(expand::Sequential, Search { grid: &grid }, start, goal, &mut path)
                    .unwrap();
                assert_eq!(path.len(), len);
                ctx.point_to_point_search_path(expand::JumpPoint, Search { grid: &grid }, start, goal, &mut path)
                    .unwrap();
                assert_eq!(path.len(), len);
            }
            None => {
                ctx.point_to_point_search_path(expand::Sequential, Search { grid: &grid }, start, goal, &mut path)
                    .unwrap_err();
                ctx.point_to_point_search_path(expand::JumpPoint, Search { grid: &grid }, start, goal, &mut path)
                    .unwrap_err();
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
        test(GRID_A, Some(13));
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
        test(GRID_B, Some(22));
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
        test(GRID_C, Some(1));
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
        test(GRID_D, Some(0));
    }

    const GRID_E: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..*.......",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_e() {
        test(GRID_E, Some(2));
    }

    const GRID_F: &[&str] = &[
        "..........",
        ".......#.#",
        ".......#.#",
        ".......#.#",
        ".......#*#",
        ".......#.#",
        ".......#.#",
        ".......###",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_f() {
        test(GRID_F, Some(19));
    }

    const GRID_G: &[&str] = &[
        "..........",
        ".......#.#",
        ".......#.#",
        ".......#.#",
        ".......#.#",
        ".......#.#",
        ".......#*#",
        ".......###",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_g() {
        test(GRID_G, Some(21));
    }

    const GRID_H: &[&str] = &[
        "..........",
        "....@.....",
        "..........",
        "...###....",
        "..........",
        "..##......",
        "..........",
        ".....##...",
        "....*.....",
        "..........",
    ];

    #[test]
    fn grid_h() {
        test(GRID_H, Some(11));
    }

    const GRID_I: &[&str] = &[
        "..........",
        "....@.....",
        "..........",
        "..........",
        "...###....",
        "...#*#....",
        "...###....",
        "..........",
        "..........",
        "..........",
    ];

    #[test]
    fn grid_i() {
        test(GRID_I, None);
    }

    const GRID_J: &[&str] = &[
        "..........",
        "....*.....",
        "..........",
        "..........",
        "...###....",
        "...#@#....",
        "...###....",
        "..........",
        "..........",
        "..........",
    ];

    #[test]
    fn grid_j() {
        test(GRID_J, None);
    }

    const GRID_K: &[&str] = &[
        "..........",
        ".#####....",
        ".....#....",
        ".###.#....",
        ".###.####.",
        "....@...#.",
        ".###.####.",
        "...#.#....",
        "..####....",
        "..#.*.....",
    ];

    #[test]
    fn grid_k() {
        test(GRID_K, Some(32));
    }

    const GRID_L: &[&str] = &[
        ".#........",
        ".#...#.*#.",
        "...#...#..",
        ".#..#.....",
        ".......#..",
        ".#...#.#..",
        "..#.......",
        "....#..#..",
        ".@.......#",
        "...#..#...",
    ];

    #[test]
    fn grid_l() {
        test(GRID_L, Some(13));
    }

    const GRID_M: &[&str] = &[
        "@......#..",
        "..........",
        ".#....##..",
        "...###....",
        "..#..#...#",
        "..#.##..#.",
        "......#...",
        "#...#.....",
        "...#..#...",
        "....#....*",
    ];

    #[test]
    fn grid_m() {
        test(GRID_M, Some(18));
    }

    const GRID_N: &[&str] = &[
        "@#........",
        "....#..##.",
        "..#.....#.",
        "..#.#..##.",
        "#.........",
        "...#.#.#..",
        "...#..#...",
        "......#..#",
        ".......#..",
        ".#.#....#*",
    ];

    #[test]
    fn grid_n() {
        test(GRID_N, Some(18));
    }

    const GRID_O: &[&str] = &[
        "@.........",
        "##...##...",
        ".....#....",
        "#..#...#.#",
        "..##.#....",
        "...#......",
        "...#...###",
        ".......#..",
        ".....#....",
        "...#..#.#*",
    ];

    #[test]
    fn grid_o() {
        test(GRID_O, Some(18));
    }

    #[test]
    fn grid_random() {
        let mut rng = Isaac64Rng::seed_from_u64(0);
        let num_tests = 1000;
        let size = Size::new(10, 10);
        let mut ctx = Context::new(size);
        let mut path = Path::default();
        for _ in 0..num_tests {
            let Test { grid, start, goal } = random_test(size, &mut rng);
            let seq_result =
                ctx.point_to_point_search_path(expand::Sequential, Search { grid: &grid }, start, goal, &mut path);
            let seq_len = path.len();
            let jps_result =
                ctx.point_to_point_search_path(expand::JumpPoint, Search { grid: &grid }, start, goal, &mut path);
            let jps_len = path.len();
            let test = Test { grid, start, goal };
            if seq_result != jps_result || seq_len != jps_len {
                print_test(&test);
            }
            assert_eq!(seq_result, jps_result);
            assert_eq!(seq_len, jps_len);
        }
    }
}
