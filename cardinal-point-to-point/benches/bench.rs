use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grid_2d::{Coord, Grid, Size};
use grid_search_cardinal_point_to_point::{Context, PointToPointSearch};

struct Cell {
    solid: bool,
}

struct World {
    grid: Grid<Cell>,
}

struct Search<'a> {
    world: &'a World,
}

impl<'a> PointToPointSearch for Search<'a> {
    fn can_enter(&self, coord: Coord) -> bool {
        self.world.grid.get(coord).map(|cell| !cell.solid).unwrap_or(true)
    }
}

struct Benchmark {
    world: World,
    context: Context,
    start: Coord,
    goal: Coord,
}

impl Benchmark {
    fn new_empty(size: Size, start: Coord, goal: Coord) -> Self {
        let world = World {
            grid: Grid::new_fn(size, |_| Cell { solid: false }),
        };
        let context = Context::new(size);
        Self {
            world,
            context,
            start,
            goal,
        }
    }
    fn search(&mut self) {
        let first = self
            .context
            .point_to_point_search_first(Search { world: &self.world }, self.start, self.goal);
        assert!(first.is_some());
    }
}

fn format_size(size: Size) -> String {
    format!("{}x{}", size.width(), size.height())
}

fn empty_corner_to_corner(c: &mut Criterion, size: Size) {
    let name = format!("empty corner to corner {}", format_size(size));
    c.bench_function(name.as_str(), |b| {
        let mut benchmark = Benchmark::new_empty(size, Coord::new(0, 0), size.to_coord().unwrap() - Coord::new(1, 1));
        b.iter(|| benchmark.search())
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    empty_corner_to_corner(c, Size::new(10, 10));
    empty_corner_to_corner(c, Size::new(100, 100));
    empty_corner_to_corner(c, Size::new(1000, 1000));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
