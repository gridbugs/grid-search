use direction::{CardinalDirection, CardinalDirections};
use grid_2d::{Coord, Grid, Size};
use grid_search_cardinal_common::{
    coord::UNIT_COORDS,
    path::Path,
    seen_set::{SeenSet, Visit},
    step::Step,
};
use std::collections::VecDeque;

pub type Distance = u32;

struct Cell {
    count: u64,
    distance: Distance,
}

pub struct DistanceMap {
    count: u64,
    grid: Grid<Cell>,
}

struct PopulateNode {
    coord: Coord,
    distance: Distance,
}

#[derive(Default)]
pub struct PopulateContext {
    queue: VecDeque<PopulateNode>,
}

struct SearchNode {
    step: Step,
    distance: Distance,
}

pub struct SearchContext {
    seen_set: SeenSet,
    queue: VecDeque<SearchNode>,
}

struct SearchState {
    distance_to_goal: Distance,
    closest_coord: Coord,
}

pub trait CanEnter {
    fn can_enter(&self, coord: Coord) -> bool;
}

impl DistanceMap {
    pub fn new(size: Size) -> Self {
        Self {
            count: 1,
            grid: Grid::new_fn(size, |_| Cell { count: 0, distance: 0 }),
        }
    }

    pub fn clear(&mut self) {
        self.count += 1;
    }

    pub fn size(&self) -> Size {
        self.grid.size()
    }

    pub fn direction_to_best_neighbour(&self, coord: Coord) -> Option<CardinalDirection> {
        let mut shortest_distance = std::u32::MAX;
        let mut direction_to_best_neighbour = None;
        if let Some(distance) = self.distance(coord) {
            shortest_distance = distance;
        }
        for direction in CardinalDirections {
            let neighbour_coord = coord + direction.coord();
            if let Some(distance) = self.distance(neighbour_coord) {
                if distance <= shortest_distance {
                    shortest_distance = distance;
                    direction_to_best_neighbour = Some(direction);
                }
            }
        }
        direction_to_best_neighbour
    }

    pub fn distance(&self, coord: Coord) -> Option<Distance> {
        if let Some(cell) = self.grid.get(coord) {
            if cell.count == self.count {
                return Some(cell.distance);
            }
        }
        None
    }
}

impl PopulateContext {
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn add(&mut self, coord: Coord) {
        self.queue.push_front(PopulateNode { coord, distance: 0 });
    }

    pub fn populate_approach<C: CanEnter>(
        &mut self,
        can_enter: &C,
        max_distance: Distance,
        distance_map: &mut DistanceMap,
    ) {
        distance_map.clear();
        for node in self.queue.iter() {
            if let Some(cell) = distance_map.grid.get_mut(node.coord) {
                cell.count = distance_map.count;
                cell.distance = 0;
            }
        }
        if max_distance == 0 {
            self.queue.clear();
            return;
        }
        while let Some(PopulateNode { coord, distance }) = self.queue.pop_back() {
            debug_assert!(distance < max_distance);
            let neighbour_distance = distance + 1;
            for direction in CardinalDirections {
                let neighbour_coord = coord + direction.coord();
                if can_enter.can_enter(neighbour_coord) {
                    if let Some(cell) = distance_map.grid.get_mut(neighbour_coord) {
                        if cell.count != distance_map.count {
                            cell.count = distance_map.count;
                            cell.distance = neighbour_distance;
                            if neighbour_distance != max_distance {
                                self.queue.push_front(PopulateNode {
                                    coord: neighbour_coord,
                                    distance: neighbour_distance,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn populate_flee<C: CanEnter>(
        &mut self,
        can_enter: &C,
        max_distance: Distance,
        distance_map: &mut DistanceMap,
    ) {
        distance_map.count += 1;
        for node in self.queue.iter() {
            if let Some(cell) = distance_map.grid.get_mut(node.coord) {
                cell.count = distance_map.count;
                cell.distance = 0;
            }
        }
        if max_distance == 0 {
            self.queue.clear();
            return;
        }
        while let Some(PopulateNode { coord, distance }) = self.queue.pop_back() {
            debug_assert!(distance <= max_distance);
            if distance == max_distance {
                self.queue.push_back(PopulateNode { coord, distance });
                break;
            }
            let neighbour_distance = distance + 1;
            for direction in CardinalDirections {
                let neighbour_coord = coord + direction.coord();
                if can_enter.can_enter(neighbour_coord) {
                    if let Some(cell) = distance_map.grid.get_mut(neighbour_coord) {
                        if cell.count != distance_map.count {
                            cell.count = distance_map.count;
                            cell.distance = neighbour_distance;
                            self.queue.push_front(PopulateNode {
                                coord: neighbour_coord,
                                distance: neighbour_distance,
                            });
                        }
                    }
                }
            }
        }
        if self.queue.is_empty() {
            return;
        }
        // at this point we know that all the nodes in the queue have a distance of max_distance
        distance_map.count += 1;
        for node in self.queue.iter_mut() {
            debug_assert!(node.distance <= max_distance);
            node.distance = 0;
            if let Some(cell) = distance_map.grid.get_mut(node.coord) {
                cell.count = distance_map.count;
                cell.distance = 0;
            }
        }
        while let Some(PopulateNode { coord, distance }) = self.queue.pop_back() {
            let neighbour_distance = distance + 1;
            for direction in CardinalDirections {
                let neighbour_coord = coord + direction.coord();
                if let Some(cell) = distance_map.grid.get_mut(neighbour_coord) {
                    if cell.count == distance_map.count - 1 {
                        cell.count += 1;
                        cell.distance = neighbour_distance;
                        self.queue.push_front(PopulateNode {
                            coord: neighbour_coord,
                            distance: neighbour_distance,
                        });
                    }
                }
            }
        }
    }
}

impl SearchContext {
    pub fn new(size: Size) -> Self {
        Self {
            seen_set: SeenSet::new(size),
            queue: VecDeque::new(),
        }
    }

    fn consider<C: CanEnter>(
        &mut self,
        can_enter: &C,
        step: Step,
        distance: Distance,
        max_distance: Distance,
        distance_map: &DistanceMap,
        search_state: &mut SearchState,
    ) {
        if let Some(Visit) = self.seen_set.try_visit_step(step, distance) {
            if can_enter.can_enter(step.to_coord) {
                if let Some(distance_to_goal) = distance_map.distance(step.to_coord) {
                    if distance <= max_distance {
                        if distance_to_goal < search_state.distance_to_goal {
                            search_state.closest_coord = step.to_coord;
                            search_state.distance_to_goal = distance_to_goal;
                        }
                        self.queue.push_back(SearchNode { step, distance });
                    }
                }
            }
        }
    }

    fn search_core<C: CanEnter>(
        &mut self,
        can_enter: &C,
        start: Coord,
        max_distance: Distance,
        distance_map: &DistanceMap,
    ) -> Option<Coord> {
        let mut search_state = if let Some(distance_to_goal) = distance_map.distance(start) {
            SearchState {
                distance_to_goal,
                closest_coord: start,
            }
        } else {
            return None;
        };
        self.seen_set.init(start);
        self.queue.clear();
        for &in_direction in &UNIT_COORDS {
            let step = Step {
                to_coord: start + in_direction.to_coord(),
                in_direction,
            };
            self.consider(can_enter, step, 1, max_distance, distance_map, &mut search_state);
        }
        while let Some(SearchNode { step, distance }) = self.queue.pop_front() {
            let next_distance = distance + 1;
            self.consider(
                can_enter,
                step.forward(),
                next_distance,
                max_distance,
                distance_map,
                &mut search_state,
            );
            self.consider(
                can_enter,
                step.left(),
                next_distance,
                max_distance,
                distance_map,
                &mut search_state,
            );
            self.consider(
                can_enter,
                step.right(),
                next_distance,
                max_distance,
                distance_map,
                &mut search_state,
            );
        }
        Some(search_state.closest_coord)
    }

    pub fn search_path<C: CanEnter>(
        &mut self,
        can_enter: &C,
        start: Coord,
        max_distance: Distance,
        distance_map: &DistanceMap,
        path: &mut Path,
    ) {
        if let Some(end) = self.search_core(can_enter, start, max_distance, distance_map) {
            self.seen_set.build_path_to(end, path);
        }
    }

    pub fn search_first<C: CanEnter>(
        &mut self,
        can_enter: &C,
        start: Coord,
        max_distance: Distance,
        distance_map: &DistanceMap,
    ) -> Option<CardinalDirection> {
        if let Some(end) = self.search_core(can_enter, start, max_distance, distance_map) {
            self.seen_set.first_direction_towards(end)
        } else {
            None
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

    struct World {
        grid: Grid<Cell>,
    }

    impl CanEnter for World {
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

    struct Test {
        world: World,
        goals: Vec<Coord>,
    }

    impl Test {
        fn from_str_slice(str_slice: &[&str]) -> Self {
            let width = str_slice[0].len() as u32;
            let height = str_slice.len() as u32;
            let size = Size::new(width, height);
            let mut grid = Grid::new_clone(size, Cell::Solid);
            let mut goals = Vec::new();
            for (y, line) in str_slice.iter().enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    let coord = Coord::new(x as i32, y as i32);
                    let cell = match ch {
                        '.' => Cell::Traversable,
                        '#' => Cell::Solid,
                        '@' => {
                            goals.push(coord);
                            Cell::Traversable
                        }
                        _ => panic!(),
                    };
                    *grid.get_checked_mut(coord) = cell;
                }
            }
            Self {
                world: World { grid },
                goals,
            }
        }
    }

    const GRID_A: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "####.#####",
        "..........",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_a() {
        use CardinalDirection::*;
        let mut path = Path::default();
        let Test { world, goals } = Test::from_str_slice(GRID_A);
        let mut populate_context = PopulateContext::default();
        let mut distance_map = DistanceMap::new(world.grid.size());
        let mut search_context = SearchContext::new(distance_map.size());
        for &coord in &goals {
            populate_context.add(coord);
        }
        populate_context.populate_approach(&world, 7, &mut distance_map);
        assert_eq!(distance_map.distance(Coord::new(4, 6)), Some(5));
        assert_eq!(distance_map.distance(Coord::new(4, 5)), Some(6));
        assert_eq!(distance_map.distance(Coord::new(3, 5)), Some(7));
        assert_eq!(distance_map.distance(Coord::new(5, 5)), Some(7));
        assert_eq!(distance_map.distance(Coord::new(4, 4)), Some(7));
        assert_eq!(distance_map.distance(Coord::new(4, 3)), None);
        assert_eq!(
            distance_map.direction_to_best_neighbour(Coord::new(4, 6)),
            Some(CardinalDirection::South)
        );
        search_context.search_path(&world, Coord::new(7, 7), 100, &distance_map, &mut path);
        let directions = path.iter().map(|n| n.in_direction).collect::<Vec<_>>();
        assert_eq!(&directions, &[West, West, West, West, West, West, South]);
        assert_eq!(distance_map.direction_to_best_neighbour(Coord::new(1, 8)), None,);
        for &coord in &goals {
            populate_context.add(coord);
        }
        populate_context.populate_flee(&world, 10, &mut distance_map);
        assert_eq!(distance_map.distance(Coord::new(4, 6)), Some(5));
        assert_eq!(distance_map.distance(Coord::new(9, 7)), Some(11));
        assert_eq!(distance_map.distance(Coord::new(1, 8)), Some(10));
        assert_eq!(
            distance_map.direction_to_best_neighbour(Coord::new(1, 7)),
            Some(CardinalDirection::East)
        );
        search_context.search_path(&world, Coord::new(7, 7), 5, &distance_map, &mut path);
        let directions = path.iter().map(|n| n.in_direction).collect::<Vec<_>>();
        assert_eq!(&directions, &[West, West, West, North, North]);
    }

    const GRID_B: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];

    #[test]
    fn grid_b() {
        let Test { world, .. } = Test::from_str_slice(GRID_B);
        let mut populate_context = PopulateContext::default();
        let mut distance_map = DistanceMap::new(world.grid.size());
        let mut search_context = SearchContext::new(distance_map.size());
        populate_context.populate_approach(&world, 7, &mut distance_map);
        assert_eq!(distance_map.distance(Coord::new(4, 5)), None);
        populate_context.populate_flee(&world, 7, &mut distance_map);
        assert_eq!(distance_map.distance(Coord::new(4, 5)), None);
        let mut path = Path::default();
        search_context.search_path(&world, Coord::new(7, 7), 5, &distance_map, &mut path);
        let directions = path.iter().map(|n| n.in_direction).collect::<Vec<_>>();
        assert_eq!(&directions, &[]);
    }

    const GRID_C: &[&str] = &[
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "####.#####",
        ".@.......@",
        ".@........",
        "..........",
    ];

    #[test]
    fn grid_c() {
        let Test { world, goals } = Test::from_str_slice(GRID_C);
        let mut populate_context = PopulateContext::default();
        let mut distance_map = DistanceMap::new(world.grid.size());
        for &coord in &goals {
            populate_context.add(coord);
        }
        populate_context.populate_approach(&world, 7, &mut distance_map);
        assert_eq!(distance_map.distance(Coord::new(4, 6)), Some(4));
        assert_eq!(
            distance_map.direction_to_best_neighbour(Coord::new(4, 6)),
            Some(CardinalDirection::South)
        );
        for &coord in &goals {
            populate_context.add(coord);
        }
        populate_context.populate_flee(&world, 10, &mut distance_map);
        assert_eq!(distance_map.distance(Coord::new(4, 6)), Some(6));
        assert_eq!(
            distance_map.direction_to_best_neighbour(Coord::new(1, 7)),
            Some(CardinalDirection::East)
        );
        assert_eq!(
            distance_map.direction_to_best_neighbour(Coord::new(6, 7)),
            Some(CardinalDirection::West)
        );
    }
}
