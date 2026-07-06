use common::messages::{ClientMessage, RoomInfo, ServerMessage};
use std::time::Instant;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod connection;
pub mod messages;
pub mod tick;

pub struct NetworkContext {
    pub is_multiplayer: bool,
    pub server_tx: Option<UnboundedSender<ClientMessage>>,
    pub server_rx: Option<UnboundedReceiver<ServerMessage>>,
    pub network_error: Option<String>,
    pub room_code: Option<String>,
    pub lan_rooms: Vec<(String, String, usize, std::net::SocketAddr, Instant)>,
    pub online_rooms: Vec<RoomInfo>,
    pub show_private_join_prompt: bool,
    pub private_room_code_input: String,
}

impl NetworkContext {
    pub fn new() -> Self {
        Self {
            is_multiplayer: false,
            server_tx: None,
            server_rx: None,
            network_error: None,
            room_code: None,
            lan_rooms: Vec::new(),
            online_rooms: Vec::new(),
            show_private_join_prompt: false,
            private_room_code_input: String::new(),
        }
    }
}
