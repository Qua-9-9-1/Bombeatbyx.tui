use crate::game::models::Cell;
use crate::game::rhythm::BeatAccuracy;
use crate::game::state::GameState;

impl GameState {
    pub fn try_place_bomb(&mut self, player_id: u32, _accuracy: BeatAccuracy) -> bool {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if !player.is_alive {
                return false;
            }

            if player.active_bombs < player.max_bombs {
                let grid_x = (player.sub_x / 2) as usize;
                let grid_y = (player.sub_y / 1) as usize;
                let idx = grid_y * self.width + grid_x;

                if self.grid[idx] == Cell::Empty {
                    let ticks = 4;

                    self.grid[idx] = Cell::Bomb {
                        owner_id: player.id,
                        ticks_left: ticks,
                    };
                    player.active_bombs += 1;
                    return true;
                }
            }
        }
        false
    }

    pub fn tick_bombs_and_explosions(&mut self) {
        let mut bombs_to_explode = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                match self.grid[idx] {
                    Cell::Bomb {
                        owner_id,
                        mut ticks_left,
                    } => {
                        if ticks_left > 0 {
                            ticks_left -= 1;
                        }
                        self.grid[idx] = Cell::Bomb {
                            owner_id,
                            ticks_left,
                        };
                        if ticks_left == 0 {
                            bombs_to_explode.push((x, y, owner_id));
                        }
                    }
                    Cell::Explosion { mut ticks_left } => {
                        if ticks_left > 0 {
                            ticks_left -= 1;
                        }
                        self.grid[idx] = if ticks_left == 0 {
                            Cell::Empty
                        } else {
                            Cell::Explosion { ticks_left }
                        };
                    }
                    _ => {}
                }
            }
        }

        for (bx, by, owner_id) in bombs_to_explode {
            self.explode_bomb(bx, by, owner_id);
        }
    }

    fn explode_bomb(&mut self, bx: usize, by: usize, owner_id: u32) {
        let range = self
            .players
            .iter()
            .find(|p| p.id == owner_id)
            .map(|p| p.bomb_range)
            .unwrap_or(2);

        if let Some(p) = self.players.iter_mut().find(|p| p.id == owner_id) {
            if p.active_bombs > 0 {
                p.active_bombs -= 1;
            }
        }

        self.grid[by * self.width + bx] = Cell::Explosion { ticks_left: 1 };
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for (dx, dy) in directions {
            for i in 1..=range {
                let tx = bx as i32 + dx * (i as i32);
                let ty = by as i32 + dy * (i as i32);

                if tx < 0 || tx >= self.width as i32 || ty < 0 || ty >= self.height as i32 {
                    break;
                }
                let idx = (ty as usize) * self.width + (tx as usize);

                match self.grid[idx] {
                    Cell::Wall => break,
                    Cell::Brick => {
                        self.grid[idx] = Cell::Explosion { ticks_left: 1 };
                        break;
                    }
                    Cell::Empty | Cell::Explosion { .. } | Cell::Bonus(_) => {
                        self.grid[idx] = Cell::Explosion { ticks_left: 1 };
                    }
                    Cell::Bomb {
                        owner_id: other_owner,
                        ..
                    } => {
                        self.grid[idx] = Cell::Bomb {
                            owner_id: other_owner,
                            ticks_left: 0,
                        };
                    }
                }
            }
        }
    }

    pub fn check_deaths(&mut self, current_beat: u64) {
        for i in 0..self.players.len() {
            if !self.players[i].is_alive || self.players[i].is_spectator {
                continue;
            }

            if self.players[i].respawn_timer.is_some() {
                continue;
            }

            let cx = self.players[i].sub_x / 2;
            let cy = self.players[i].sub_y / 1;
            let cell = self.get_cell(cx, cy);

            if let Cell::Explosion { .. } = cell {
                if self.players[i].shield_until_beat == Some(current_beat) {
                    continue;
                }
                self.players[i].is_alive = false;
                self.players[i].lives = self.players[i].lives.saturating_sub(1);
                if self.players[i].lives == 0 {
                    if self.players[i].death_beat.is_none() {
                        self.players[i].death_beat = Some(current_beat);
                    }
                }
                self.players[i].death_pos = Some((self.players[i].sub_x, self.players[i].sub_y));
                self.players[i].respawn_timer =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
                self.players[i].combo = 0;
                self.players[i].collected_bonuses.clear();
                self.players[i].second_item = None;
                self.players[i].max_bombs = 1;
                self.players[i].bomb_range = 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::Player;
    use crate::game::rhythm::BeatAccuracy;

    fn create_test_player(id: u32) -> Player {
        Player {
            id,
            is_host: id == 1,
            name: format!("Player {}", id),
            skin: "🤖".to_string(),
            sub_x: 2,
            sub_y: 1,
            is_alive: true,
            score: 0,
            combo: 0,
            max_bombs: 1,
            active_bombs: 0,
            bomb_range: 1,
            last_acted_beat: None,
            last_accuracy: BeatAccuracy::Waiting,
            last_action_time: None,
            spam_lockout_until: None,
            active_emote: None,
            emote_until: None,
            lives: 3,
            death_pos: None,
            respawn_timer: None,
            collected_bonuses: Vec::new(),
            is_spectator: false,
            second_item: None,
            shield_until_beat: None,
            is_ready: false,
            death_beat: None,
        }
    }

    #[test]
    fn try_place_bomb_increases_active_bombs_when_cell_empty() {
        let mut state = GameState::new(5, 5);
        state.players = vec![create_test_player(1)];

        let placed = state.try_place_bomb(1, BeatAccuracy::Perfect);

        assert!(placed);
        assert_eq!(state.players[0].active_bombs, 1);
        let cell_idx = 1 * 5 + 1;
        assert!(matches!(state.grid[cell_idx], Cell::Bomb { owner_id: 1, .. }));
    }

    #[test]
    fn try_place_bomb_fails_when_max_bombs_reached() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.active_bombs = 1;
        state.players = vec![player];

        let placed = state.try_place_bomb(1, BeatAccuracy::Perfect);

        assert!(!placed);
    }

    #[test]
    fn tick_bombs_and_explosions_triggers_explosion_on_zero_ticks() {
        let mut state = GameState::new(5, 5);
        let mut player = create_test_player(1);
        player.active_bombs = 1;
        player.bomb_range = 1;
        state.players = vec![player];

        let cell_idx = 2 * 5 + 2;
        state.grid[cell_idx] = Cell::Bomb {
            owner_id: 1,
            ticks_left: 1,
        };

        state.tick_bombs_and_explosions();

        assert_eq!(state.players[0].active_bombs, 0);
        assert!(matches!(state.grid[cell_idx], Cell::Explosion { .. }));
        assert!(matches!(state.grid[2 * 5 + 1], Cell::Explosion { .. }));
        assert!(matches!(state.grid[2 * 5 + 3], Cell::Explosion { .. }));
        assert!(matches!(state.grid[1 * 5 + 2], Cell::Explosion { .. }));
        assert!(matches!(state.grid[3 * 5 + 2], Cell::Explosion { .. }));
    }
}
