use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grid_2d::{Coord, Grid, Size};
use grid_search_cardinal_point_to_point::{expand, Context, PointToPointSearch};
use grid_search_maze::{MazeCell, MazeGenerator};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;

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
        self.world.grid.get(coord).map(|cell| !cell.solid).unwrap_or(false)
    }
}

struct Benchmark {
    world: World,
    context: Context,
    start: Coord,
    goal: Coord,
}

impl Benchmark {
    fn new_empty(size: Size) -> Self {
        let world = World {
            grid: Grid::new_fn(size, |_| Cell { solid: false }),
        };
        let context = Context::new(size);
        let start = Coord::new(0, 0);
        let goal = size.to_coord().unwrap() - Coord::new(1, 1);
        Self {
            world,
            context,
            start,
            goal,
        }
    }
    fn new_maze(size: Size, seed: u64) -> Self {
        let mut generator = MazeGenerator::new(size);
        let mut rng = Isaac64Rng::seed_from_u64(seed);
        let maze = generator.generate(Coord::new(0, 0), &mut rng);
        let world = World {
            grid: Grid::new_grid_map(maze, |cell| match cell {
                MazeCell::Passage => Cell { solid: false },
                MazeCell::Wall => Cell { solid: true },
            }),
        };
        Self {
            context: Context::new(world.grid.size()),
            start: Coord::new(0, 0),
            goal: world.grid.size().to_coord().unwrap() - Coord::new(1, 1),
            world,
        }
    }
    fn size(&self) -> Size {
        self.world.grid.size()
    }
    fn search<E: expand::Expand>(&mut self, expand: E) {
        let first = self
            .context
            .point_to_point_search_first(expand, Search { world: &self.world }, self.start, self.goal)
            .unwrap();
        assert!(first.is_some());
        black_box(first);
    }
    fn add(mut self, c: &mut Criterion, name: String) {
        c.bench_function(&format!("{} {:?}", name, expand::Sequential), |b| {
            b.iter(|| self.search(expand::Sequential))
        });
        c.bench_function(&format!("{} {:?}", name, expand::JumpPoint), |b| {
            b.iter(|| self.search(expand::JumpPoint))
        });
    }
}

fn format_size(size: Size) -> String {
    format!("{}x{}", size.width(), size.height())
}

fn empty(c: &mut Criterion, size: Size) {
    Benchmark::new_empty(size).add(c, format!("empty {}", format_size(size)));
}

fn maze(c: &mut Criterion, size: Size, seed: u64) {
    let benchmark = Benchmark::new_maze(size, seed);
    let name = format!("maze (seed = {}) {}", seed, format_size(benchmark.size()));
    benchmark.add(c, name);
}

fn maze_benchmark(c: &mut Criterion) {
    let sizes = [Size::new(5, 5), Size::new(50, 50), Size::new(100, 100)];
    let mut rng = Isaac64Rng::seed_from_u64(0);
    let seeds = [rng.gen::<u64>(), rng.gen::<u64>(), rng.gen::<u64>()];
    for &size in &sizes {
        for &seed in &seeds {
            maze(c, size, seed);
        }
    }
}

fn empty_benchmark(c: &mut Criterion) {
    empty(c, Size::new(9, 9));
    empty(c, Size::new(99, 99));
    empty(c, Size::new(199, 199));
}

fn criterion_benchmark(c: &mut Criterion) {
    maze_benchmark(c);
    empty_benchmark(c);
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
