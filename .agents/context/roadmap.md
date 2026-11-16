# Gameplay Roadmap (Ideas Not Yet Implemented)

Candidate features, ordered by expected fun per effort. Keep the core rule in mind: **every meaningful action happens on the beat**. When one is implemented, move its description to `business-context.md` and delete it here.

## 1. Quick Wins
* **Bots in online rooms:** let the host fill empty slots with AI bots. The AI is already in `common`; the server needs bot players without a `Peer`, driven from the server tick on each beat, and counted in the start and disconnect rules.
* **Beat-synced bomb fuse display:** show the remaining beats on each bomb (`4`, `3`, `2`, `1`) to make timing readable.
* **Kill feed:** a short sidebar log ("Alpha 💥 Beta") using the existing kill credit.
* **Combo milestone feedback:** flash the HUD when the multiplier goes up or Fever starts.

## 2. Rhythm Depth
* **Off-beat traps:** tiles that are only walkable on even beats.
* **Tempo ramp:** BPM rises during sudden death. Requires broadcasting the BPM change so the client `RhythmEngine` resyncs, so do it after a dedicated rhythm-sync message exists.
* **Perfect streak reward:** 8 Perfects in a row grant a free shield beat.
* **Rhythm duel:** when two players are adjacent, the more accurate one on the same beat pushes the other back one cell.

## 3. New Bonuses / Items
* **Kick:** walking into a bomb pushes it one cell per beat until it hits something.
* **Remote bomb:** the bomb only explodes when the spell key is pressed.
* **Curse (malus):** inverted controls or forced bomb drop for 8 beats; can be passed on by touching another player.
* **Ghost:** walk through bricks for 4 beats.

## 4. Modes & Meta
* **Teams (2v2):** friendly fire toggle, shared win condition.
* **King of the Hill:** hold the center cell on beat to earn points.
* **Map presets:** open arena, maze, cross; plus a brick density setting in the lobby.
* **Replays / ghosts:** the simulation is deterministic given a seed and the inputs, so matches can be recorded.

## 5. Technical Prerequisites
* **Server-side settings validation:** clamp every `RoomSettings` field received in `UpdateSettings` (map size, BPM, lives, bot count).
* **Rhythm resync:** clients currently realign `beat_count` from `last_acted_beat`; using `GameState.current_beat` would be more robust.
