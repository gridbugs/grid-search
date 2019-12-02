use grid_search_cardinal_point_to_point::{Context, PointToPointSearch};
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
    let seed = 3;
    let size = Size::new(200, 200);
    let start = Coord::new(0, 0);
    let mut generator = MazeGenerator::new(size);
    let mut rng = Isaac64Rng::seed_from_u64(seed);
    let maze = generator.generate(start, &mut rng);
    let goal = maze.size().to_coord().unwrap() - Coord::new(1, 1);
    let mut context = Context::new(maze.size());
    let profile = context.point_to_point_search_profile(Search { maze: &maze }, start, goal);
    println!("{:#?}", profile);
}
