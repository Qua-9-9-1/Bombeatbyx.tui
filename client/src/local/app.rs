use crate::local::settings::{ClientSettings, GaugeSkin};
use crate::tui::Tui;
use crate::ui;
use common::game::{BeatAccuracy, GameAction, GameContext};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::{Duration, Instant};


pub const CELL_W: u16 = 2;
pub const CELL_H: u16 = 1;

pub struct App {
    pub game_ctx: GameContext,
    pub current_player_id: u32,
    pub game_run: bool,
    pub settings: ClientSettings,
}

impl App {
    pub fn new() -> Self {
        let mut game_ctx = GameContext::new(15, 15, 80.0);
        let current_player_id = 1;

        game_ctx.state.players.push(common::game::Player {
            id: current_player_id,
            sub_x: 2,
            sub_y: 2,
            is_alive: true,
            max_bombs: 3,
            active_bombs: 0,
            bomb_range: 2,
            score: 0,
            combo: 0,
            last_acted_beat: None,
            last_accuracy: BeatAccuracy::Waiting,
            last_action_time: None,
            spam_lockout_until: None,
        });

        Self {
            game_ctx,
            current_player_id,
            game_run: true,
            settings: ClientSettings {
                player_name: "Newbie".to_string(),
                key_up: 'z',
                key_down: 's',
                key_left: 'q',
                key_right: 'd',
                gauge_skin: GaugeSkin::Undertale,
            },
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> std::io::Result<()> {
        let _ = tui.init();
        let mut last_time = Instant::now();
        let mut last_render = Instant::now();
        let render_rate = Duration::from_millis(16);
        let tick_rate = Duration::from_millis(16);
        let mut lag = Duration::ZERO;

        while self.game_run {
            let current_time = Instant::now();
            lag += current_time.duration_since(last_time);
            last_time = current_time;

            self.game_ctx.tick_game_logic();
            self.handle_inputs()?;
            self.update_physics(tick_rate, &mut lag);

            if current_time.duration_since(last_render) >= render_rate {
                tui.draw(|frame| ui::draw(frame, self))?;
                last_render = current_time;
            }

            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn handle_inputs(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                if key.code == KeyCode::Esc {
                    self.game_run = false;
                    return Ok(());
                }
                if let Some(action) = self.map_key_to_action(key.code) {
                    self.game_ctx.process_player_action(self.current_player_id, action);
                }
            }
        }
        Ok(())
    }

    fn map_key_to_action(&self, code: KeyCode) -> Option<GameAction> {
        match code {
            KeyCode::Left | KeyCode::Char('q') => Some(GameAction::MoveLeft),
            KeyCode::Right | KeyCode::Char('d') => Some(GameAction::MoveRight),
            KeyCode::Up | KeyCode::Char('z') => Some(GameAction::MoveUp),
            KeyCode::Down | KeyCode::Char('s') => Some(GameAction::MoveDown),
            KeyCode::Char(' ') => Some(GameAction::PlaceBomb),
            KeyCode::Char('e') => Some(GameAction::TriggerSpell),
            _ => None,
        }
    }

    fn update_physics(&mut self, tick_rate: Duration, lag: &mut Duration) {
        while *lag >= tick_rate {
            *lag -= tick_rate;
        }
    }

    pub fn get_feedback_and_combo(&self) -> (&'static str, u32) {
        if let Some(player) = self.game_ctx.state.players.iter().find(|p| p.id == 1) {
            if player.last_acted_beat == Some(self.game_ctx.rhythm.beat_count) {
                return (player.last_accuracy.as_str(), player.combo);
            }
            return ("WAITING...", player.combo);
        }
        ("WAITING...", 0)
    }
}
