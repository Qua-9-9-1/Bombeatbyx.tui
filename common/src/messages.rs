pub enum ClientMessage {
    Move(Direction),
    PlaceBomb,
    JoinGame(String),
    LeaveGame(String),
}

pub enum ServerMessage {
    GameState(GameState),
    PlayerDied(u32),
    BombExploded { x: usize, y: usize, radius: usize },
}