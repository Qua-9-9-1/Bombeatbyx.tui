use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Game,
    Menu,
}

#[derive(Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub space_pressed: bool,

    pub timer_up: u8,
    pub timer_down: u8,
    pub timer_left: u8,
    pub timer_right: u8,

    pub release_up_pending: bool,
    pub release_down_pending: bool,
    pub release_left_pending: bool,
    pub release_right_pending: bool,
}

pub struct ControlsManager {
    pub mode: ControlMode,
    pub state: InputState,
    pub menu_cursor: usize,
    pub max_menu_items: usize,
    pub supports_release: bool,
}

impl ControlsManager {
    pub fn new() -> Self {
        Self {
            mode: ControlMode::Game,
            state: InputState::default(),
            menu_cursor: 0,
            max_menu_items: 3,
            supports_release: false,
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            self.supports_release = true;
        }

        match self.mode {
            ControlMode::Game => self.handle_game_input(key),
            ControlMode::Menu => self.handle_menu_input(key),
        }
    }

    fn handle_game_input(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('m') {
            if key.kind != KeyEventKind::Release {
                self.mode = ControlMode::Menu;
                self.state = InputState::default();
            }
            return;
        }

        let is_release = key.kind == KeyEventKind::Release;
        let active_frames = 8;

        match key.code {
            KeyCode::Up | KeyCode::Char('z') => {
                if is_release {
                    if self.supports_release {
                        self.state.release_up_pending = true;
                    }
                } else {
                    self.state.up = true;
                    self.state.down = false;
                    self.state.timer_up = active_frames;
                    self.state.release_up_pending = false;
                }
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if is_release {
                    if self.supports_release {
                        self.state.release_down_pending = true;
                    }
                } else {
                    self.state.down = true;
                    self.state.up = false;
                    self.state.timer_down = active_frames;
                    self.state.release_down_pending = false;
                }
            }
            KeyCode::Left | KeyCode::Char('q') | KeyCode::Char('a') => {
                if is_release {
                    if self.supports_release {
                        self.state.release_left_pending = true;
                    }
                } else {
                    self.state.left = true;
                    self.state.right = false;
                    self.state.timer_left = active_frames;
                    self.state.release_left_pending = false;
                }
            }
            KeyCode::Right | KeyCode::Char('d') => {
                if is_release {
                    if self.supports_release {
                        self.state.release_right_pending = true;
                    }
                } else {
                    self.state.right = true;
                    self.state.left = false;
                    self.state.timer_right = active_frames;
                    self.state.release_right_pending = false;
                }
            }
            KeyCode::Char(' ') => {
                if !is_release {
                    self.state.space_pressed = true;
                }
            }
            _ => {}
        }
    }

    fn handle_menu_input(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('m') => {
                self.mode = ControlMode::Game;
            }
            KeyCode::Up | KeyCode::Char('z') => {
                if self.menu_cursor > 0 {
                    self.menu_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if self.menu_cursor < self.max_menu_items - 1 {
                    self.menu_cursor += 1;
                }
            }
            KeyCode::Enter => {
                if self.menu_cursor == 0 {
                    self.mode = ControlMode::Game;
                }
            }
            _ => {}
        }
    }

    pub fn tick_input_decay(&mut self) {
        if self.supports_release {
            if self.state.release_up_pending {
                self.state.up = false;
                self.state.release_up_pending = false;
            }
            if self.state.release_down_pending {
                self.state.down = false;
                self.state.release_down_pending = false;
            }
            if self.state.release_left_pending {
                self.state.left = false;
                self.state.release_left_pending = false;
            }
            if self.state.release_right_pending {
                self.state.right = false;
                self.state.release_right_pending = false;
            }
            return;
        }

        if self.state.timer_up > 0 {
            self.state.timer_up -= 1;
            if self.state.timer_up == 0 {
                self.state.up = false;
            }
        }
        if self.state.timer_down > 0 {
            self.state.timer_down -= 1;
            if self.state.timer_down == 0 {
                self.state.down = false;
            }
        }
        if self.state.timer_left > 0 {
            self.state.timer_left -= 1;
            if self.state.timer_left == 0 {
                self.state.left = false;
            }
        }
        if self.state.timer_right > 0 {
            self.state.timer_right -= 1;
            if self.state.timer_right == 0 {
                self.state.right = false;
            }
        }
    }
}
