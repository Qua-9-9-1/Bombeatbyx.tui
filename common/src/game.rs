use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Wall,
    Brick,
    Bomb { owner_id: u32, ticks_left: u8 },
    Explosion { ticks_left: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub sub_x: i32,
    pub sub_y: i32,
    pub is_alive: bool,
    pub max_bombs: u8,
    pub active_bombs: u8,
    pub bomb_range: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
    pub players: Vec<Player>,
}

impl GameState {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![Cell::Empty; width * height];
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 || (x % 2 == 0 && y % 2 == 0) {
                    grid[y * width + x] = Cell::Wall;
                }
            }
        }
        Self { width, height, grid, players: Vec::new() }
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return Cell::Wall;
        }
        self.grid[(y as usize) * self.width + (x as usize)]
    }
}