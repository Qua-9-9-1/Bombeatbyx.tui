# Business & Gameplay Context (Bombeatbyx.tui)

This document describes the high-level goals, game rules, and accessibility modes of the Bombeatbyx rhythm battle game.

## 1. Project & Game Overview
Bombeatbyx TUI is a rhythm-based multiplayer grid-arena battle game. Players move, drop bombs, and trigger actions strictly synchronized with a music rhythm/BPM beat. The main objective is to defeat other players in real-time grid combat.

## 2. Gameplay Mechanics
* **Rhythm Synchronization:** Action inputs must align with the active BPM pulse.
* **Emotes:**
  - Players can trigger temporary gestures mapped to keys `1` to `4`:
    - `1` -> `👋` (Wave)
    - `2` -> `✌️` (Peace)
    - `3` -> `🖕` (Middle Finger)
    - `4` -> `👍` (Thumbs Up)
  - Emotes override the player's skin on the grid map for 1.5 seconds (`emote_until` field in the player state).
  - Emotes bypass rhythm combo lockouts and spam delays, rendering immediately on trigger.

## 3. ASCII Fallback Rendering
To maintain compatibility on terminals lacking UTF-8 emoji support, the game allows toggling `ascii_mode` in settings. The rendering fallback mappings are:

| Element | Emoji / Unicode | ASCII Fallback | Description |
| :--- | :--- | :--- | :--- |
| **Grid Wall** | `██` | `##` | Impassable grid boundaries |
| **Brick Wall** | `░░` | `[]` | Destructible obstacles |
| **Bomb** | Animated | `()` | Active bomb ticking |
| **Fire Explosion**| `💥` / `🔥` | `##` / `**` / `::` | Visual blast states |
| **Alive Player** | Emoji (dynamic) | Two-letter tag | Tags: `RO`, `CA`, `FR`, `FO`, `PE` |
| **Dead Player** | `💀` | `XX` | Defeated player state |
| **Emote 1 (Wave)** | `👋` | `HI` | Avoids consecutive double letters |
| **Emote 2 (Peace)**| `✌️` | `VI` | Avoids consecutive double letters |
| **Emote 3 (Finger)**| `🖕` | `FU` | Avoids consecutive double letters |
| **Emote 4 (Thumbs)**| `👍` | `OK` | Avoids consecutive double letters |
