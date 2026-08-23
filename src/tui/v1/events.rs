//! Event handling for keyboard and mouse input

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use crate::errors::Result;
use std::time::Duration;

pub enum AppEvent {
    Quit,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
}

pub fn read_event() -> Result<Option<AppEvent>> {
    loop {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(Some(AppEvent::Tick));
        }

        match event::read()? {
            Event::Key(key) => {
                // Handle common quit keys
                if matches!(key.code, KeyCode::Char('c'))
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    return Ok(Some(AppEvent::Quit));
                }

                if matches!(key.code, KeyCode::Esc) {
                    return Ok(Some(AppEvent::Quit));
                }

                return Ok(Some(AppEvent::Key(key)));
            }
            Event::Mouse(mouse) => {
                // Only mouse button presses matter; drain everything else
                // (moves, drags, releases) instead of redrawing per event.
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    return Ok(Some(AppEvent::Mouse(mouse)));
                }
            }
            _ => {}
        }
    }
}
