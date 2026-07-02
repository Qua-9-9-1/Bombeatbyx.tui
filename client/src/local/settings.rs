use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientSettings {
    pub name: String,
    pub skin: String,
    pub gauge_skin: GaugeSkin,
    pub key_up: char,
    pub key_down: char,
    pub key_left: char,
    pub key_right: char,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum GaugeSkin {
    NecroDancer,
    Undertale,
    Simple,
}

impl Default for ClientSettings {
    fn default() -> Self {
        ClientSettings {
            name: "Newbie".to_string(),
            skin: "😊".to_string(),
            key_up: 'z',
            key_down: 's',
            key_left: 'q',
            key_right: 'd',
            gauge_skin: GaugeSkin::Simple,
        }
    }
}
