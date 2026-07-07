use common::game::{GameContext, Player, RoomSettings};
use common::messages::ServerMessage;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

pub struct Peer {
    pub id: u32,
    pub name: String,
    pub skin: String,
    pub tx: UnboundedSender<ServerMessage>,
    pub is_ready: bool,
    pub is_spectator: bool,
    pub ip: IpAddr,
}

pub struct Room {
    pub code: String,
    pub is_public: bool,
    pub is_lan: bool,
    pub peers: HashMap<u32, Peer>,
    pub host_id: Option<u32>,
    pub next_peer_id: u32,
    pub room_settings: RoomSettings,
    pub game_ctx: Option<GameContext>,
    pub in_game: bool,
    pub banned_ips: HashSet<IpAddr>,
}

impl Room {
    pub fn new(code: String, is_public: bool, is_lan: bool) -> Self {
        Self {
            code,
            is_public,
            is_lan,
            peers: HashMap::new(),
            host_id: None,
            next_peer_id: 1,
            room_settings: RoomSettings::default(),
            game_ctx: None,
            in_game: false,
            banned_ips: HashSet::new(),
        }
    }

    pub fn get_lobby_players(&self) -> Vec<Player> {
        let mut players: Vec<Player> = self
            .peers
            .values()
            .map(|p| Player {
                id: p.id,
                is_host: Some(p.id) == self.host_id,
                name: p.name.clone(),
                skin: p.skin.clone(),
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
                is_spectator: p.is_spectator,
                second_item: None,
                shield_until_beat: None,
                is_ready: p.is_ready,
                death_beat: None,
            })
            .collect();
        players.sort_by_key(|p| p.id);
        players
    }

    pub fn broadcast(&self, msg: ServerMessage) {
        for peer in self.peers.values() {
            let _ = peer.tx.send(msg.clone());
        }
    }
}

pub struct ServerState {
    pub rooms: HashMap<String, Room>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
}

pub type SharedState = Arc<Mutex<ServerState>>;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;
    use std::net::IpAddr;

    #[test]
    fn get_lobby_players_returns_sorted_list() {
        let mut room = Room::new("TEST".to_string(), true, false);
        room.host_id = Some(1);
        
        let (tx, _) = unbounded_channel();
        let ip = "127.0.0.1".parse::<IpAddr>().unwrap();

        room.peers.insert(2, Peer {
            id: 2,
            name: "Player 2".to_string(),
            skin: "🐱".to_string(),
            tx: tx.clone(),
            is_ready: true,
            is_spectator: false,
            ip,
        });

        room.peers.insert(1, Peer {
            id: 1,
            name: "Player 1".to_string(),
            skin: "🤖".to_string(),
            tx: tx.clone(),
            is_ready: false,
            is_spectator: false,
            ip,
        });

        let players = room.get_lobby_players();

        assert_eq!(players.len(), 2);
        assert_eq!(players[0].id, 1);
        assert!(players[0].is_host);
        assert_eq!(players[1].id, 2);
        assert!(!players[1].is_host);
    }
}
