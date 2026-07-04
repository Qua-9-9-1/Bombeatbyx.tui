use crate::game::actions::GameAction;
use crate::game::models::RoomSettings;
use crate::game::state::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub code: String,
    pub host_name: String,
    pub player_count: usize,
    pub max_players: usize,
    pub is_public: bool,
    pub is_lan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    GetRooms,
    CreateRoom {
        is_public: bool,
        is_lan: bool,
    },
    JoinRoom {
        code: String,
        name: String,
        skin: String,
    },
    UpdateSettings(RoomSettings),
    StartGame,
    ToggleReady,
    Action(GameAction),
    LeaveLobby,
    Pong,
    StopGame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    RoomList(Vec<RoomInfo>),
    Joined {
        your_id: u32,
        room_code: String,
        current_state: GameState,
        settings: RoomSettings,
    },
    LobbyUpdate {
        players: Vec<crate::game::models::Player>,
        settings: RoomSettings,
    },
    GameStarted {
        initial_state: GameState,
    },
    GameStateUpdate(GameState),
    GameEnded,
    ConnectionFailed(String),
    Ping,
    GameStopped {
        players: Vec<crate::game::models::Player>,
        settings: RoomSettings,
    },
}
