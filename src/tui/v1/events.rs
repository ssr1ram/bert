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
    if event::poll(Duration::from_millis(100))? {
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

                Ok(Some(AppEvent::Key(key)))
            }
            Event::Mouse(mouse) => {
                // Only process mouse button press events
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    Ok(Some(AppEvent::Mouse(mouse)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    } else {
        Ok(Some(AppEvent::Tick))
    }
}
