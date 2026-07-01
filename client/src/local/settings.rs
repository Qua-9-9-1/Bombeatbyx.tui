use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientSettings {
    pub player_name: String,
    pub key_up: char,
    pub key_down: char,
    pub key_left: char,
    pub key_right: char,
    pub gauge_skin: GaugeSkin,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum GaugeSkin {
    NecroDancer,
    Undertale,
    Simple,
}
