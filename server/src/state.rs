use common::game::{GameContext, Player, RoomSettings};
use common::messages::ServerMessage;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};

/// Messages queued for one client. A client that stops reading fills it: new
/// messages are then dropped and the connection is closed, instead of letting
/// server memory grow without limit.
pub const OUTBOX_CAPACITY: usize = 512;

/// Bounded, non-blocking sender to one client.
#[derive(Clone, Debug)]
pub struct PeerSender(Sender<ServerMessage>);

impl PeerSender {
    /// Queues a message; fails (message dropped) when the outbox is full or closed.
    pub fn send(&self, msg: ServerMessage) -> Result<(), ()> {
        self.0.try_send(msg).map_err(|_| ())
    }

    /// `true` once the client stopped draining its outbox.
    pub fn is_saturated(&self) -> bool {
        self.0.capacity() == 0
    }
}

pub fn peer_channel() -> (PeerSender, Receiver<ServerMessage>) {
    let (tx, rx) = mpsc::channel(OUTBOX_CAPACITY);
    (PeerSender(tx), rx)
}

pub struct Peer {
    pub id: u32,
    pub name: String,
    pub skin: String,
    pub tx: PeerSender,
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
    /// Smoothed round-trip time per peer, measured with Ping/Pong.
    pub latencies: HashMap<u32, std::time::Duration>,
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
            latencies: HashMap::new(),
        }
    }

    /// One-way network delay estimate for a peer (half its round-trip time).
    pub fn one_way_latency(&self, peer_id: u32) -> std::time::Duration {
        self.latencies
            .get(&peer_id)
            .map_or(std::time::Duration::ZERO, |rtt| *rtt / 2)
    }

    pub fn get_lobby_players(&self) -> Vec<Player> {
        let mut players: Vec<Player> = self
            .peers
            .values()
            .map(|p| {
                let mut player =
                    Player::new(p.id, p.name.clone(), p.skin.clone(), self.room_settings.lives);
                player.is_host = Some(p.id) == self.host_id;
                player.is_spectator = p.is_spectator;
                player.is_ready = p.is_ready;
                player
            })
            .collect();
        players.sort_by_key(|p| p.id);
        players
    }

    /// Changes the host everywhere (room, running game) and tells every peer.
    pub fn set_host(&mut self, new_host_id: u32) {
        self.host_id = Some(new_host_id);
        if let Some(ref mut ctx) = self.game_ctx {
            for player in &mut ctx.state.players {
                player.is_host = player.id == new_host_id;
            }
            ctx.state.host_player_id = Some(new_host_id);
        }
        let new_host_name = self
            .peers
            .get(&new_host_id)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        self.broadcast(ServerMessage::HostTransferred {
            new_host_id,
            new_host_name,
        });
    }

    pub fn broadcast(&self, msg: ServerMessage) {
        for peer in self.peers.values() {
            let _ = peer.tx.send(msg.clone());
        }
    }
}

/// Simultaneous WebSocket connections allowed from one IP address
/// (a LAN party behind one router stays well below it).
pub const MAX_CONNECTIONS_PER_IP: usize = 16;

pub struct ServerState {
    pub rooms: HashMap<String, Room>,
    /// Open connections per client IP.
    pub connections: HashMap<IpAddr, usize>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    /// Registers a new connection; `false` if the IP already has too many.
    pub fn try_open_connection(&mut self, ip: IpAddr) -> bool {
        let count = self.connections.entry(ip).or_insert(0);
        if *count >= MAX_CONNECTIONS_PER_IP {
            return false;
        }
        *count += 1;
        true
    }

    pub fn close_connection(&mut self, ip: IpAddr) {
        if let Some(count) = self.connections.get_mut(&ip) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.connections.remove(&ip);
            }
        }
    }
}

pub type SharedState = Arc<Mutex<ServerState>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[test]
    fn connections_per_ip_are_capped_and_released() {
        let mut state = ServerState::new();
        let ip = "10.0.0.7".parse::<IpAddr>().unwrap();
        for _ in 0..MAX_CONNECTIONS_PER_IP {
            assert!(state.try_open_connection(ip));
        }

        let over_limit = state.try_open_connection(ip);
        state.close_connection(ip);
        let after_release = state.try_open_connection(ip);

        assert!(!over_limit);
        assert!(after_release);
    }

    #[test]
    fn peer_sender_drops_messages_once_outbox_is_full() {
        let (tx, _rx) = peer_channel();
        for _ in 0..OUTBOX_CAPACITY {
            assert!(tx.send(ServerMessage::GameEnded).is_ok());
        }

        let overflow = tx.send(ServerMessage::GameEnded);

        assert!(overflow.is_err());
        assert!(tx.is_saturated());
    }

    #[test]
    fn get_lobby_players_returns_sorted_list() {
        let mut room = Room::new("TEST".to_string(), true, false);
        room.host_id = Some(1);
        
        let (tx, _rx) = peer_channel();
        let ip = "127.0.0.1".parse::<IpAddr>().unwrap();

        room.peers.insert(2, Peer {
            id: 2,
            name: "Player 2".to_string(),
            skin: "ðŸ±".to_string(),
            tx: tx.clone(),
            is_ready: true,
            is_spectator: false,
            ip,
        });

        room.peers.insert(1, Peer {
            id: 1,
            name: "Player 1".to_string(),
            skin: "ðŸ¤–".to_string(),
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

    #[test]
    fn set_host_updates_running_game_and_notifies_peers() {
        let mut room = Room::new("TEST".to_string(), true, false);
        let (tx, mut rx) = peer_channel();
        let ip = "127.0.0.1".parse::<IpAddr>().unwrap();
        for id in [1, 2] {
            room.peers.insert(id, Peer {
                id,
                name: format!("Player {}", id),
                skin: "ðŸ¤–".to_string(),
                tx: tx.clone(),
                is_ready: true,
                is_spectator: false,
                ip,
            });
        }
        room.host_id = Some(1);
        let players = room.get_lobby_players();
        room.game_ctx = Some(GameContext::new_match(&room.room_settings, players));

        room.set_host(2);

        let ctx = room.game_ctx.as_ref().unwrap();
        assert!(ctx.state.players.iter().find(|p| p.id == 2).unwrap().is_host);
        assert!(!ctx.state.players.iter().find(|p| p.id == 1).unwrap().is_host);
        assert!(matches!(
            rx.try_recv(),
            Ok(ServerMessage::HostTransferred { new_host_id: 2, .. })
        ));
    }
}
