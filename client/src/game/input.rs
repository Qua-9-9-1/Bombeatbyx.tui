#[derive(Default)]
pub struct InputState {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
}

impl InputState {
    pub fn tick_decay(&mut self) {
        if self.up > 0 { self.up -= 1; }
        if self.down > 0 { self.down -= 1; }
        if self.left > 0 { self.left -= 1; }
        if self.right > 0 { self.right -= 1; }
    }
}