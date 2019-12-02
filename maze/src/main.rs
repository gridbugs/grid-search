use grid_search_maze::{Coord, MazeCell, MazeGenerator, Size};
use rand::SeedableRng;
use rand_isaac::Isaac64Rng;

fn main() {
    let size = Size::new(100, 100);
    let mut rng = Isaac64Rng::seed_from_u64(0);
    let mut generator = MazeGenerator::new(size);
    let maze = generator.generate(Coord::new(1, 1), &mut rng);
    for row in maze.rows() {
        for cell in row {
            let ch = match cell {
                MazeCell::Passage => '.',
                MazeCell::Wall => '█',
            };
            print!("{}", ch);
        }
        println!("");
    }
}
