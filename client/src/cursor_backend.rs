//! Backend wrapper that only sends cursor visibility changes.
//!
//! Ratatui hides the cursor on every frame. Some terminal layers (the Docker
//! CLI on Windows, old consoles) do not understand that sequence and print it
//! as text (`[?25l`) next to the last drawn cell, where the diff renderer never
//! erases it. Sending it only when the visibility actually changes removes the
//! artifacts and some useless output.

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};

pub struct CursorStateBackend<B: Backend> {
    inner: B,
    is_hidden: Option<bool>,
}

impl<B: Backend> CursorStateBackend<B> {
    pub fn new(inner: B) -> Self {
        Self { inner, is_hidden: None }
    }
}

impl<B: Backend> Backend for CursorStateBackend<B> {
    type Error = B::Error;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.inner.draw(content)
    }

    fn append_lines(&mut self, n: u16) -> Result<(), Self::Error> {
        self.inner.append_lines(n)
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        if needs_change(self.is_hidden, true) {
            self.inner.hide_cursor()?;
            self.is_hidden = Some(true);
        }
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        if needs_change(self.is_hidden, false) {
            self.inner.show_cursor()?;
            self.is_hidden = Some(false);
        }
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        self.inner.get_cursor_position()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        self.inner.set_cursor_position(position)
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.inner.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        self.inner.clear_region(clear_type)
    }

    fn size(&self) -> Result<Size, Self::Error> {
        self.inner.size()
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        self.inner.window_size()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush()
    }
}

/// A visibility command is sent only when the state is unknown or different.
fn needs_change(is_hidden: Option<bool>, hide: bool) -> bool {
    is_hidden != Some(hide)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_command_is_always_sent() {
        assert!(needs_change(None, true));
        assert!(needs_change(None, false));
    }

    #[test]
    fn repeated_hide_is_not_sent_again() {
        let after_hide = Some(true);

        let resend = needs_change(after_hide, true);

        assert!(!resend);
    }

    #[test]
    fn show_after_hide_is_sent() {
        assert!(needs_change(Some(true), false));
    }
}
