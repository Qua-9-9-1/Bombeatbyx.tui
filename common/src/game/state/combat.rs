use super::GameState;
use crate::game::models::{Cell, KILL_POINTS, SELF_KILL_PENALTY};

pub const RESPAWN_DELAY_SECS: u64 = 3;

impl GameState {
    pub fn award_points(&mut self, player_id: u32, points: u32) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.score = p.score.saturating_add(points);
        }
    }

    pub fn check_deaths(&mut self, current_beat: u64) {
        for i in 0..self.players.len() {
            let player = &self.players[i];
            if !player.is_alive || player.is_spectator || player.respawn_timer.is_some() {
                continue;
            }
            if player.is_protected(current_beat) {
                continue;
            }

            if let Cell::Explosion { owner_id, .. } = self.get_cell(player.sub_x / 2, player.sub_y) {
                self.kill_player(i, Some(owner_id), current_beat);
            }
        }
    }

    /// Removes a life, resets power-ups and credits the killer (if any).
    pub fn kill_player(&mut self, index: usize, killer_id: Option<u32>, current_beat: u64) {
        let victim_id = self.players[index].id;
        {
            let victim = &mut self.players[index];
            victim.is_alive = false;
            victim.lives = victim.lives.saturating_sub(1);
            if victim.lives == 0 && victim.death_beat.is_none() {
                victim.death_beat = Some(current_beat);
            }
            victim.death_pos = Some((victim.sub_x, victim.sub_y));
            victim.respawn_timer = Some(
                std::time::Instant::now() + std::time::Duration::from_secs(RESPAWN_DELAY_SECS),
            );
            victim.combo = 0;
            victim.reset_power_ups();
        }

        match killer_id {
            Some(killer) if killer == victim_id => {
                let victim = &mut self.players[index];
                victim.score = victim.score.saturating_sub(SELF_KILL_PENALTY);
            }
            Some(killer) => {
                if let Some(p) = self.players.iter_mut().find(|p| p.id == killer) {
                    p.kills += 1;
                    p.score = p.score.saturating_add(KILL_POINTS);
                }
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::models::Player;

    fn player_at(id: u32, x: i32, y: i32) -> Player {
        let mut player = Player::new(id, format!("Player {}", id), "🤖", 3);
        player.sub_x = x * 2;
        player.sub_y = y;
        player
    }

    fn fire(owner_id: u32) -> Cell {
        Cell::Explosion {
            ticks_left: 1,
            owner_id,
        }
    }

    #[test]
    fn check_deaths_kills_player_standing_in_fire() {
        let mut state = GameState::new(5, 5);
        state.players = vec![player_at(1, 1, 1)];
        state.grid[1 * 5 + 1] = fire(1);

        state.check_deaths(10);

        assert!(!state.players[0].is_alive);
        assert_eq!(state.players[0].lives, 2);
    }

    #[test]
    fn check_deaths_spares_player_with_active_shield_for_several_beats() {
        let mut state = GameState::new(5, 5);
        let mut player = player_at(1, 1, 1);
        player.shield_until_beat = Some(12);
        state.players = vec![player];
        state.grid[1 * 5 + 1] = fire(1);

        state.check_deaths(12);

        assert!(state.players[0].is_alive);
    }

    #[test]
    fn check_deaths_kills_player_after_shield_expired() {
        let mut state = GameState::new(5, 5);
        let mut player = player_at(1, 1, 1);
        player.shield_until_beat = Some(12);
        state.players = vec![player];
        state.grid[1 * 5 + 1] = fire(1);

        state.check_deaths(13);

        assert!(!state.players[0].is_alive);
    }

    #[test]
    fn check_deaths_credits_kill_to_bomb_owner() {
        let mut state = GameState::new(7, 5);
        state.players = vec![player_at(1, 1, 1), player_at(2, 3, 1)];
        state.grid[1 * 7 + 3] = fire(1);

        state.check_deaths(10);

        assert_eq!(state.players[0].kills, 1);
        assert_eq!(state.players[0].score, KILL_POINTS);
    }

    #[test]
    fn check_deaths_penalizes_self_kill() {
        let mut state = GameState::new(5, 5);
        let mut player = player_at(1, 1, 1);
        player.score = 500;
        state.players = vec![player];
        state.grid[1 * 5 + 1] = fire(1);

        state.check_deaths(10);

        assert_eq!(state.players[0].kills, 0);
        assert_eq!(state.players[0].score, 500 - SELF_KILL_PENALTY);
    }
}
