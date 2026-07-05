use common::game::RoomSettings;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum KeyPreset {
    ZQSD,
    WASD,
}

impl KeyPreset {
    pub fn label(&self) -> &'static str {
        match self {
            KeyPreset::ZQSD => "ZQSD (AZERTY)",
            KeyPreset::WASD => "WASD (QWERTY)",
        }
    }

    pub fn keys(&self) -> (char, char, char, char, char, char) {
        match self {
            KeyPreset::ZQSD => ('z', 's', 'q', 'd', ' ', 'e'),
            KeyPreset::WASD => ('w', 's', 'a', 'd', ' ', 'e'),
        }
    }

    pub fn detect() -> Self {
        let lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .unwrap_or_default()
            .to_lowercase();
        if lang.starts_with("fr") {
            KeyPreset::ZQSD
        } else {
            KeyPreset::WASD
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum GaugeSkin {
    NecroDancer,
    Undertale,
    Simple,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientSettings {
    pub name: String,
    pub skin: String,
    pub gauge_skin: GaugeSkin,
    pub key_preset: KeyPreset,
    pub key_up: char,
    pub key_down: char,
    pub key_left: char,
    pub key_right: char,
    pub key_bomb: char,
    pub key_spell: char,
    pub ascii_mode: bool,
    pub server_addr: String,
    pub last_room_settings: RoomSettings,
}

impl ClientSettings {
    pub fn apply_preset(&mut self, preset: KeyPreset) {
        let (up, down, left, right, bomb, spell) = preset.keys();
        self.key_up = up;
        self.key_down = down;
        self.key_left = left;
        self.key_right = right;
        self.key_bomb = bomb;
        self.key_spell = spell;
        self.key_preset = preset;
    }

    pub fn keys(&self) -> (char, char, char, char, char, char) {
        (self.key_up, self.key_down, self.key_left, self.key_right, self.key_bomb, self.key_spell)
    }

    fn config_path() -> std::path::PathBuf {
        let exe = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        exe.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<Self>(&data) {
                return settings;
            }
        }
        let preset = KeyPreset::detect();
        let s = Self::default_with_preset(preset);
        let _ = s.save();
        s
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn default_with_preset(preset: KeyPreset) -> Self {
        let (up, down, left, right, bomb, spell) = preset.keys();
        ClientSettings {
            name: "Newbie".to_string(),
            skin: "😊".to_string(),
            gauge_skin: GaugeSkin::Simple,
            key_preset: preset,
            key_up: up,
            key_down: down,
            key_left: left,
            key_right: right,
            key_bomb: bomb,
            key_spell: spell,
            ascii_mode: false,
            server_addr: "127.0.0.1:3000".to_string(),
            last_room_settings: RoomSettings::default(),
        }
    }
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self::default_with_preset(KeyPreset::detect())
    }
}
