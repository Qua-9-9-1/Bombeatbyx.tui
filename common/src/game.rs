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

    pub fn move_player(&mut self, player_id: u32, move_x: i32, move_y: i32) {
        let target_pos = if let Some(player) = self.players.iter().find(|p| p.id == player_id) {
            if !player.is_alive { return; }
            Some((player.sub_x + move_x, player.sub_y + move_y))
        } else {
            None
        };

        if let Some((next_x, next_y)) = target_pos {
            let cell_x = next_x / 2;
            let cell_y = next_y / 1;
            let cell = self.get_cell(cell_x, cell_y);

            if !matches!(cell, Cell::Wall | Cell::Brick | Cell::Bomb { .. }) {
                if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                    player.sub_x = next_x;
                    player.sub_y = next_y;
                }
            }
        }
    }

    pub fn try_place_bomb(&mut self, player_id: u32) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if !player.is_alive { return; }

            if player.active_bombs < player.max_bombs {
                let grid_x = (player.sub_x / 2) as usize;
                let grid_y = (player.sub_y / 1) as usize;
                let idx = grid_y * self.width + grid_x;

                if self.grid[idx] == Cell::Empty {
                    self.grid[idx] = Cell::Bomb { owner_id: player.id, ticks_left: 4 };
                    player.active_bombs += 1;
                }
            }
        }
    }

    pub fn trigger_action_2(&mut self, player_id: u32) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if player.is_alive && player.bomb_range < 6 {
                player.bomb_range += 1;
            }
        }
    }

    pub fn tick_bombs_and_explosions(&mut self) {
        let mut bombs_to_explode = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                match self.grid[idx] {
                    Cell::Bomb { owner_id, mut ticks_left } => {
                        if ticks_left > 0 { ticks_left -= 1; }
                        self.grid[idx] = Cell::Bomb { owner_id, ticks_left };
                        if ticks_left == 0 { bombs_to_explode.push((x, y, owner_id)); }
                    }
                    Cell::Explosion { mut ticks_left } => {
                        if ticks_left > 0 { ticks_left -= 1; }
                        self.grid[idx] = if ticks_left == 0 { Cell::Empty } else { Cell::Explosion { ticks_left } };
                    }
                    _ => {}
                }
            }
        }

        for (bx, by, owner_id) in bombs_to_explode {
            self.explode_bomb(bx, by, owner_id);
        }
    }

    fn explode_bomb(&mut self, bx: usize, by: usize, owner_id: u32) {
        let range = self.players.iter().find(|p| p.id == owner_id).map(|p| p.bomb_range).unwrap_or(2);

        if let Some(p) = self.players.iter_mut().find(|p| p.id == owner_id) {
            if p.active_bombs > 0 { p.active_bombs -= 1; }
        }

        self.grid[by * self.width + bx] = Cell::Explosion { ticks_left: 1 };
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for (dx, dy) in directions {
            for i in 1..=range {
                let tx = bx as i32 + dx * (i as i32);
                let ty = by as i32 + dy * (i as i32);

                if tx < 0 || tx >= self.width as i32 || ty < 0 || ty >= self.height as i32 { break; }
                let idx = (ty as usize) * self.width + (tx as usize);

                match self.grid[idx] {
                    Cell::Wall => break,
                    Cell::Brick => {
                        self.grid[idx] = Cell::Explosion { ticks_left: 1 };
                        break;
                    }
                    Cell::Empty | Cell::Explosion { .. } => {
                        self.grid[idx] = Cell::Explosion { ticks_left: 1 };
                    }
                    Cell::Bomb { owner_id: other_owner, .. } => {
                        self.grid[idx] = Cell::Bomb { owner_id: other_owner, ticks_left: 0 };
                    }
                }
            }
        }
    }

    pub fn check_deaths(&mut self) {
        for i in 0..self.players.len() {
            if !self.players[i].is_alive { continue; }
            
            let cx = self.players[i].sub_x / 2;
            let cy = self.players[i].sub_y / 1;

            let cell = self.get_cell(cx, cy);

            if let Cell::Explosion { .. } = cell {
                self.players[i].is_alive = false;
            }
        }
    }
}