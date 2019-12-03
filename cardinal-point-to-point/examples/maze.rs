use grid_search_cardinal_point_to_point::{expand, Context, PointToPointSearch};
use grid_search_maze::{Coord, Grid, MazeCell, MazeGenerator, Size};
use rand::SeedableRng;
use rand_isaac::Isaac64Rng;

struct Search<'a> {
    maze: &'a Grid<MazeCell>,
}

impl<'a> PointToPointSearch for Search<'a> {
    fn can_enter(&self, coord: Coord) -> bool {
        match self.maze.get(coord) {
            Some(MazeCell::Passage) => true,
            _ => false,
        }
    }
}

fn main() {
    let seed = 17761629189777429372;
    let size = Size::new(5, 5);
    let start = Coord::new(0, 0);
    let mut generator = MazeGenerator::new(size);
    let mut rng = Isaac64Rng::seed_from_u64(seed);
    let maze = generator.generate(start, &mut rng);
    let goal = maze.size().to_coord().unwrap() - Coord::new(1, 1);
    let mut context = Context::new(maze.size());
    let (profile, _) = context.point_to_point_search_profile(expand::JumpPoint, Search { maze: &maze }, start, goal);
    println!("{:#?}", profile);
}
