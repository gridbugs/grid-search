use grid_search_maze::{Coord, MazeCell, MazeGenerator, Size};

fn main() {
    let mut generator = MazeGenerator::new(Size::new(30, 15));
    let mut rng = rand::thread_rng();
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
