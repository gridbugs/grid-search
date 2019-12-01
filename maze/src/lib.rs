pub use grid_2d::{Coord, Grid, Size};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
enum WallDirection {
    East,
    South,
}

#[derive(Debug, Clone, Copy)]
struct Wall {
    coord: Coord,
    direction: WallDirection,
}

impl Wall {
    fn walls_around(coord: Coord) -> [Wall; 4] {
        [
            Wall {
                coord,
                direction: WallDirection::East,
            },
            Wall {
                coord,
                direction: WallDirection::South,
            },
            Wall {
                coord: coord - Coord::new(1, 0),
                direction: WallDirection::East,
            },
            Wall {
                coord: coord - Coord::new(0, 1),
                direction: WallDirection::South,
            },
        ]
    }
    fn coords(&self) -> [Coord; 2] {
        [
            self.coord,
            match self.direction {
                WallDirection::East => self.coord + Coord::new(1, 0),
                WallDirection::South => self.coord + Coord::new(0, 1),
            },
        ]
    }
}

struct Walls {
    east: bool,
    south: bool,
}

impl Walls {
    fn get_mut(&mut self, direction: WallDirection) -> &mut bool {
        match direction {
            WallDirection::East => &mut self.east,
            WallDirection::South => &mut self.south,
        }
    }
}

struct GenerationCell {
    in_maze: bool,
    seen_walls: Walls,
    passages: Walls,
}

impl Default for GenerationCell {
    fn default() -> Self {
        Self {
            in_maze: false,
            seen_walls: Walls {
                east: false,
                south: false,
            },
            passages: Walls {
                east: false,
                south: false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MazeCell {
    Wall,
    Passage,
}

pub struct MazeGenerator {
    grid: Grid<GenerationCell>,
    walls_to_visit: Vec<Wall>,
}

impl MazeGenerator {
    pub fn new(size: Size) -> Self {
        Self {
            grid: Grid::new_default(size),
            walls_to_visit: Vec::new(),
        }
    }

    fn add_wall(&mut self, wall: Wall) {
        if let Some(cell) = self.grid.get_mut(wall.coord) {
            let seen = cell.seen_walls.get_mut(wall.direction);
            if !*seen {
                *seen = true;
                self.walls_to_visit.push(wall);
            }
        }
    }

    fn add_cell_at(&mut self, coord: Coord) {
        if let Some(cell) = self.grid.get_mut(coord) {
            if !cell.in_maze {
                cell.in_maze = true;
                for &wall in Wall::walls_around(coord).iter() {
                    self.add_wall(wall);
                }
            }
        }
    }

    fn add_passage(&mut self, wall: Wall) {
        if let Some(cell) = self.grid.get_mut(wall.coord) {
            let passage = cell.passages.get_mut(wall.direction);
            *passage = true;
        }
    }

    fn process_wall(&mut self, wall: Wall) -> Option<()> {
        let [coord_a, coord_b] = wall.coords();
        let in_maze_a = self.grid.get(coord_a)?.in_maze;
        let in_maze_b = self.grid.get(coord_b)?.in_maze;
        if in_maze_a {
            if in_maze_b {
                return None;
            } else {
                self.add_cell_at(coord_b);
            }
        } else {
            if in_maze_b {
                self.add_cell_at(coord_a);
            } else {
                return None;
            }
        }
        self.add_passage(wall);
        Some(())
    }

    fn remove_random_wall<R: Rng>(&mut self, rng: &mut R) -> Option<Wall> {
        if self.walls_to_visit.is_empty() {
            None
        } else {
            let index = rng.gen_range(0, self.walls_to_visit.len());
            Some(self.walls_to_visit.swap_remove(index))
        }
    }

    fn build_maze(&self) -> Grid<MazeCell> {
        let size = self.grid.size() * 2 - Size::new(1, 1);
        let mut maze = Grid::new_clone(size, MazeCell::Wall);
        for (coord, cell) in self.grid.enumerate() {
            let maze_cell_coord = coord * 2;
            if cell.in_maze {
                if let Some(maze_cell) = maze.get_mut(maze_cell_coord) {
                    *maze_cell = MazeCell::Passage;
                }
            }
            if cell.passages.east {
                let coord = maze_cell_coord + Coord::new(1, 0);
                if let Some(maze_cell) = maze.get_mut(coord) {
                    *maze_cell = MazeCell::Passage;
                }
            }
            if cell.passages.south {
                let coord = maze_cell_coord + Coord::new(0, 1);
                if let Some(maze_cell) = maze.get_mut(coord) {
                    *maze_cell = MazeCell::Passage;
                }
            }
        }
        maze
    }

    pub fn generate<R: Rng>(&mut self, start: Coord, rng: &mut R) -> Grid<MazeCell> {
        for cell in self.grid.iter_mut() {
            *cell = GenerationCell::default();
        }
        self.add_cell_at(start);
        while let Some(wall) = self.remove_random_wall(rng) {
            self.process_wall(wall);
        }
        self.build_maze()
    }
}
