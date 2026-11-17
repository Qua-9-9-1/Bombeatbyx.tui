# How to Play

## 1. Main Menu

| Entry | What it does |
| :--- | :--- |
| **Host Game** | Create a room, online (on the configured server) or LAN (a server is started on your machine). |
| **Join Game** | Browse online or LAN rooms, or join a private room with its code. |
| **Solo vs Bots** | Play offline against AI opponents. |
| **Settings** | Key bindings, rhythm gauge style, display mode, server address. |
| **Quit** | Leave the game. |

Navigate with `Z`/`S` (or `W`/`S`, or the arrow keys) and confirm with `Enter`. `Esc` goes back.

## 2. Controls

Default bindings depend on your system language: **ZQSD** for French layouts, **WASD** otherwise. Change them in **Settings**.

| Action | AZERTY | QWERTY | Also |
| :--- | :--- | :--- | :--- |
| Move | `Z` `Q` `S` `D` | `W` `A` `S` `D` | Arrow keys |
| Place bomb | `Space` | `Space` | |
| Use item | `E` | `E` | |
| Emotes | `F1` to `F4` | `F1` to `F4` | 👋 ✌️ 🖕 👍 |
| Pause | `Esc` | `Esc` | |

Emotes are free: they ignore the beat and never break your combo.

## 3. The Rhythm Rule

Everything you do (moving, bombing, using an item) must be **on the beat**.

### Reading the beat without leaving the map

| Cue | Meaning |
| :--- | :--- |
| Arena border turns **thick and green** | Press now: the hit window is open. |
| Your cell turns **green** | Same, right under your eyes. |
| Floor tiles swap | A new beat just started. |
| `◆ ◇ ◇ ◇` under the arena | Position in the 4-beat bar. |
| Title above the arena | Rating of your last input, with `EARLY` / `LATE` when you were off, or `ONE ACTION PER BEAT` if you pressed twice. |
| Beat sound | Optional metronome, see [Settings](#beat-sound). |

The gauge under the map shows the same timing in more detail.

| Timing (distance to the beat) | Rating | Points | Combo |
| :--- | :--- | :--- | :--- |
| within 6 % of a beat | **Perfect** | 50 | +1 |
| within 12 % | **Great** | 20 | +1 |
| within 20 % | **Okay** | 5 | kept |
| further | **Miss** | 0 | reset, action ignored |

* **One action per beat.** Extra presses in the same beat are ignored, and pressing keys faster than every 100 ms triggers a short lockout.
* **Keep moving.** Letting a beat pass without acting resets your combo.
* Bumping into a wall on the beat still counts as keeping the groove (it consumes your beat).
* **Combo multiplier:** rhythm points are multiplied by x2 from combo 8, x3 from 16, x4 from 24.
* **Fever:** from combo 16, your bombs get **+1 range**. The HUD shows `FEVER!`.

## 4. The Arena

| Cell | Look (emoji / ASCII) | Rule |
| :--- | :--- | :--- |
| Wall | `██` / `##` | Indestructible. |
| Brick | `░░` / `[]` | Destroyed by blasts; may hide a bonus. |
| Bomb | 💣 / `()` | Explodes 4 beats after being placed. Blocks movement. |
| Fire | 💥 🔥 / `##` `**` | Lethal on contact for one beat. |

* Blasts go in a cross, stop at walls, at bricks (unless *Pierce*) and at other bombs, which **explode instantly** (chain reaction).
* A bomb keeps the range it had when it was placed.
* When you are hit you lose a life, all your power-ups and your combo. You respawn after 3 seconds, far from other players, with 3 beats of invulnerability (shown by a blue background).

## 5. Bonuses and Items

Bonuses appear every few beats (room setting) and inside destroyed bricks. Blinking bonuses are about to disappear.

| Bonus | Emoji / ASCII | Effect |
| :--- | :--- | :--- |
| Bomb | 💣 / `B+` | +1 bomb at the same time |
| Fire | 🔥 / `R+` | +1 blast range |
| Pierce | 🔪 / `P+` | Blasts go through bricks |
| Extra life | 💖 / `L+` | +1 life (rare, capped at 9) |
| Shield *(item)* | 🛡️ / `S+` | Invulnerable for 3 beats |
| Detonator *(item)* | 🧨 / `D+` | Explodes all your bombs right now |
| Dash *(item)* | 👟 / `F+` | Moves up to 3 cells in the direction you face |

You hold **one item at a time** (shown between brackets next to your name). Use it with `E`, on the beat. Picking up a new item replaces the current one.

## 6. Game Modes and Scoring

| Mode | Victory |
| :--- | :--- |
| **Deathmatch** | Last player with lives left. |
| **Score** | First player to reach the target score. |

With a **time limit**, the match also ends when the timer runs out; the ranking then decides the podium.

| Event | Points |
| :--- | :--- |
| Kill another player | +300 |
| Kill yourself | -150 |
| Destroy a brick | +10 |
| Pick up a bonus | +25 |
| Successful on-beat action | rating points x combo multiplier |

**Sudden death** (room option): after 2 minutes (or half the time limit), the arena closes in one cell per beat, spiralling towards the center. Players caught by a closing wall lose a life.

## 7. The Lobby

The lobby has three panels: room settings (center), help for the selected line (left) and players (right).

* `Z`/`S` select a line, `Q`/`D` change its value (host only for room settings).
* `Enter` on **Your Name** edits it (letters, digits, spaces, `-` and `_`, 12 characters max).
* `Q`/`D` on **Your Skin** browses the 25 skins. Two ready players cannot share a skin.
* `Enter` on **READY** toggles your ready state. The game starts when every player (at least 2) is ready.

Host-only shortcuts on the player list:

| Key | Action |
| :--- | :--- |
| `Tab` | Select the next player |
| `T` | Transfer host rights |
| `K` | Kick |
| `B` | Ban (by IP, for this room) |

## 8. Solo vs Bots

Choose **Solo vs Bots** in the main menu. Two extra lobby lines appear:

| Setting | Values |
| :--- | :--- |
| **Bots** | 1 to 7 opponents |
| **Bot Level** | **Easy**: hesitant and clumsy, only breaks bricks. **Normal**: hunts players and uses items. **Hard**: near-perfect rhythm, aggressive but careful. |

Bots follow the same rules as you, including the beat. Pausing (`Esc`) freezes the whole game.

## 9. Settings

| Setting | Description |
| :--- | :--- |
| Key Preset | Switch between ZQSD and WASD. |
| Move / Bomb / Item keys | `Enter`, then press the new key. |
| Gauge Skin | Rhythm gauge style: NecroDancer, Undertale or Simple. |
| Display Mode | **Emojis**, or **ASCII** for terminals that cannot render emojis. |
| Beat Sound | Metronome: **Every beat**, **Every 4 beats** or **Off**. See below. |
| Server | Address of the online server (`host:port`). `Enter` to edit, `Enter` to save, `Esc` to cancel. |

Settings are saved in `config.json`, next to the client executable.

### Beat sound

The metronome uses the **terminal bell**, so the sound is played by your terminal (it also works through SSH or Docker). Changing the setting rings once so you can check it.

Most terminals play a long system "ding" by default, which is too slow for a beat. Point the bell to the short click shipped in `assets/beat-click.wav` instead. In **Windows Terminal**, open the settings JSON (`Ctrl+Shift+,`) and add to `profiles.defaults`:

```json
"bellStyle": "audible",
"bellSound": "C:/path/to/Bombeatbyx.tui/assets/beat-click.wav"
```

If you hear nothing: check that the bell style is `audible` and that system sounds are not muted. Some terminals only flash the window or the taskbar on a bell.
