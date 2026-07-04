use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use common::messages::ServerMessage;
use common::game::{GameContext, RoomSettings, Player};

pub struct Peer {
    pub id: u32,
    pub name: String,
    pub skin: String,
    pub color: String,
    pub tx: UnboundedSender<ServerMessage>,
}

pub struct ServerState {
    pub peers: HashMap<u32, Peer>,
    pub host_id: Option<u32>,
    pub next_peer_id: u32,
    pub room_settings: RoomSettings,
    pub game_ctx: Option<GameContext>,
    pub in_game: bool,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            host_id: None,
            next_peer_id: 1,
            room_settings: RoomSettings::default(),
            game_ctx: None,
            in_game: false,
        }
    }

    pub fn get_lobby_players(&self) -> Vec<Player> {
        let mut players: Vec<Player> = self.peers.values().map(|p| Player {
            id: p.id,
            is_host: Some(p.id) == self.host_id,
            name: p.name.clone(),
            skin: p.skin.clone(),
            color: p.color.clone(),
            sub_x: 0,
            sub_y: 0,
            is_alive: true,
            score: 0,
            combo: 0,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 1,
            last_acted_beat: None,
            last_accuracy: common::game::BeatAccuracy::Waiting,
            last_action_time: None,
            spam_lockout_until: None,
            active_emote: None,
            emote_until: None,
            lives: self.room_settings.lives,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
        }).collect();
        players.sort_by_key(|p| p.id);
        players
    }

    pub fn broadcast(&self, msg: ServerMessage) {
        for peer in self.peers.values() {
            let _ = peer.tx.send(msg.clone());
        }
    }
}

pub type SharedState = Arc<Mutex<ServerState>>;

pub fn get_color_for_id(id: u32) -> String {
    let colors = ["green", "magenta", "yellow", "blue", "red", "cyan", "white"];
    colors[(id as usize) % colors.len()].to_string()
}
