use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

pub struct Maze {
    pub path: Vec<Vec<Cell>>,
}

impl Maze {
    pub fn new(size: usize) -> Maze {
        let mut visited: Vec<Point> = Vec::new();

        let mut path: Vec<Vec<Cell>> = vec![vec![Cell::new(); size]; size];
        let directions = vec![
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        let mut rng = thread_rng();

        visited.push(Point {
            x: rng.gen_range(0..size) as i128,
            y: rng.gen_range(0..size) as i128,
        });

        while !visited.is_empty() {
            let cell = visited.last().unwrap();
            let mut directions_tmp = directions.clone();
            directions_tmp.shuffle(&mut rng);

            let mut found = false;

            for dir in directions_tmp {
                let nx = cell.x + delta_dir(dir).x;
                let ny = cell.y + delta_dir(dir).y;

                if nx >= 0
                    && ny >= 0
                    && nx < size as i128
                    && ny < size as i128
                    && path[ny as usize][nx as usize].is_zero()
                {
                    path[cell.y as usize][cell.x as usize].carve(dir);
                    path[ny as usize][nx as usize].carve(opposite_dir(dir));
                    visited.push(Point { x: nx, y: ny });
                    found = true;
                    break;
                }
            }

            if !found {
                visited.pop();
            }
        }

        path[(size as f32 / 2.).floor() as usize][0].carve(Direction::West);
        path[(size as f32 / 2.).floor() as usize][size - 1].carve(Direction::East);

        let mut coords: Vec<Point> = Vec::new();
        for (i, row) in path.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if cell.directions.len() == 1 {
                    coords.push(Point {
                        x: j as i128,
                        y: i as i128,
                    });
                }
            }
        }

        coords.shuffle(&mut rng);

        for i in 0..5 {
            if coords.len() > i {
                path[coords[i].y as usize][coords[i].x as usize].coin = true;
            }
        }

        Maze { path }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub directions: Vec<Direction>,
    pub coin: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: i128,
    y: i128,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            directions: Vec::new(),
            coin: false,
        }
    }

    pub fn carve(&mut self, direction: Direction) {
        self.directions.push(direction);
    }

    pub fn is_zero(&self) -> bool {
        self.directions.is_empty()
    }
}

pub fn delta_dir(dir: Direction) -> Point {
    match dir {
        Direction::North => Point { x: 0, y: -1 },
        Direction::South => Point { x: 0, y: 1 },
        Direction::East => Point { x: 1, y: 0 },
        Direction::West => Point { x: -1, y: 0 },
    }
}

pub fn opposite_dir(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
    }
}
