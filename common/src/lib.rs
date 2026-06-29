// common/src/lib.rs
pub struct Player {
    pub x: u16,
    pub y: u16,
}

pub struct GameState {
    pub player: Player,
}

impl GameState {
    pub fn move_player(&mut self, dx: i16, dy: i16) {
        self.player.x = (self.player.x as i16 + dx) as u16;
        self.player.y = (self.player.y as i16 + dy) as u16;
    }
}