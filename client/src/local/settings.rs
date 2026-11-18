use common::game::RoomSettings;
use serde::{Deserialize, Serialize};

/// The server binds 27300 by default (see `server/src/main.rs`).
pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:27300";
/// Old default that never matched the server port; migrated on load.
const LEGACY_SERVER_ADDR: &str = "127.0.0.1:3000";
pub const MAX_SERVER_ADDR_CHARS: usize = 64;

/// Accepts `host:port`, optionally prefixed by `ws://` or `wss://`.
pub fn is_valid_server_addr(addr: &str) -> bool {
    let rest = addr
        .strip_prefix("ws://")
        .or_else(|| addr.strip_prefix("wss://"))
        .unwrap_or(addr);
    let host_port = rest.trim_end_matches("/ws").trim_end_matches('/');
    match host_port.rsplit_once(':') {
        Some((host, port)) => !host.is_empty() && port.parse::<u16>().is_ok_and(|p| p > 0),
        None => false,
    }
}

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

/// Metronome played through the terminal bell (works over SSH and Docker).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BeatSound {
    Off,
    #[default]
    EveryBeat,
    /// Only the first beat of each 4-beat bar.
    EveryBar,
}

pub const BEATS_PER_BAR: u64 = 4;

pub const EMOTE_LABELS: [&str; 4] = ["Emote Hi", "Emote Peace", "Emote Rude", "Emote Thumb"];

fn default_emote_keys() -> [char; 4] {
    ['1', '2', '3', '4']
}

impl BeatSound {
    pub fn label(self) -> &'static str {
        match self {
            BeatSound::Off => "Off",
            BeatSound::EveryBeat => "Every beat",
            BeatSound::EveryBar => "Every 4 beats",
        }
    }

    pub fn next(self) -> Self {
        match self {
            BeatSound::Off => BeatSound::EveryBeat,
            BeatSound::EveryBeat => BeatSound::EveryBar,
            BeatSound::EveryBar => BeatSound::Off,
        }
    }

    pub fn rings_on(self, beat: u64) -> bool {
        match self {
            BeatSound::Off => false,
            BeatSound::EveryBeat => true,
            BeatSound::EveryBar => beat % BEATS_PER_BAR == 0,
        }
    }
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
    #[serde(default)]
    pub beat_sound: BeatSound,
    /// Keys of the 4 emotes (👋 ✌️ 🖕 👍). F1-F4 always work too.
    #[serde(default = "default_emote_keys")]
    pub emote_keys: [char; 4],
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
        (
            self.key_up,
            self.key_down,
            self.key_left,
            self.key_right,
            self.key_bomb,
            self.key_spell,
        )
    }

    fn config_path() -> std::path::PathBuf {
        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
        exe.parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(mut settings) = serde_json::from_str::<Self>(&data) {
                if settings.server_addr == LEGACY_SERVER_ADDR {
                    settings.server_addr = DEFAULT_SERVER_ADDR.to_string();
                    let _ = settings.save();
                }
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
            skin: "👶".to_string(),
            gauge_skin: GaugeSkin::Simple,
            key_preset: preset,
            key_up: up,
            key_down: down,
            key_left: left,
            key_right: right,
            key_bomb: bomb,
            key_spell: spell,
            ascii_mode: false,
            beat_sound: BeatSound::default(),
            emote_keys: default_emote_keys(),
            server_addr: DEFAULT_SERVER_ADDR.to_string(),
            last_room_settings: RoomSettings::default(),
        }
    }
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self::default_with_preset(KeyPreset::detect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_map_correct_characters() {
        assert_eq!(KeyPreset::ZQSD.keys(), ('z', 's', 'q', 'd', ' ', 'e'));
        assert_eq!(KeyPreset::WASD.keys(), ('w', 's', 'a', 'd', ' ', 'e'));
    }

    #[test]
    fn default_server_addr_matches_server_default_port() {
        let settings = ClientSettings::default_with_preset(KeyPreset::WASD);

        assert!(settings.server_addr.ends_with(":27300"));
    }

    #[test]
    fn is_valid_server_addr_accepts_host_port_and_ws_urls() {
        let valid = ["127.0.0.1:27300", "play.example.com:443", "ws://10.0.0.2:27300/ws"];

        let results: Vec<bool> = valid.iter().map(|a| is_valid_server_addr(a)).collect();

        assert_eq!(results, vec![true, true, true]);
    }

    #[test]
    fn is_valid_server_addr_rejects_missing_or_bad_port() {
        let invalid = ["localhost", "127.0.0.1:", ":27300", "127.0.0.1:99999", "host:abc"];

        let has_valid = invalid.iter().any(|a| is_valid_server_addr(a));

        assert!(!has_valid);
    }

    #[test]
    fn beat_sound_every_bar_rings_on_bar_starts_only() {
        let beats = 0..8u64;

        let rung: Vec<u64> = beats.filter(|b| BeatSound::EveryBar.rings_on(*b)).collect();

        assert_eq!(rung, vec![0, 4]);
    }

    #[test]
    fn beat_sound_cycles_through_all_modes() {
        let start = BeatSound::Off;

        let back = start.next().next().next();

        assert_eq!(start.next(), BeatSound::EveryBeat);
        assert_eq!(back, start);
    }

    #[test]
    fn config_without_beat_sound_field_still_loads() {
        let mut json = serde_json::to_value(ClientSettings::default_with_preset(KeyPreset::WASD))
            .expect("serializable settings");
        json.as_object_mut().expect("object").remove("beat_sound");

        let loaded: ClientSettings = serde_json::from_value(json).expect("legacy config loads");

        assert_eq!(loaded.beat_sound, BeatSound::EveryBeat);
    }

    #[test]
    fn config_without_emote_keys_defaults_to_number_keys() {
        let mut json = serde_json::to_value(ClientSettings::default_with_preset(KeyPreset::ZQSD))
            .expect("serializable settings");
        json.as_object_mut().expect("object").remove("emote_keys");

        let loaded: ClientSettings = serde_json::from_value(json).expect("legacy config loads");

        assert_eq!(loaded.emote_keys, ['1', '2', '3', '4']);
    }

    #[test]
    fn apply_preset_updates_settings_keybindings() {
        let mut settings = ClientSettings::default_with_preset(KeyPreset::WASD);

        settings.apply_preset(KeyPreset::ZQSD);

        assert_eq!(settings.key_preset, KeyPreset::ZQSD);
        assert_eq!(settings.key_up, 'z');
        assert_eq!(settings.key_left, 'q');
    }
}
