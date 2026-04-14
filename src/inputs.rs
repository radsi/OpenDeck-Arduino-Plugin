use crate::mappings::{COL_COUNT, KEY_COUNT, ROW_COUNT};

#[derive(Debug, Clone)]
pub enum InputEvent {
    ButtonDown(u8),
    ButtonUp(u8),
    ButtonState(Vec<bool>),
}

pub struct InputState {
    pub buttons: Vec<bool>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            buttons: vec![false; KEY_COUNT],
        }
    }
}

pub fn process_input(state: &mut InputState, line: &str) -> Option<InputEvent> {
    log::info!("Processing input: {}", line);

    if let Some((row, col, pressed)) = parse_matrix(line) {
        if row >= ROW_COUNT || col >= COL_COUNT {
            return None;
        }

        let idx = row * COL_COUNT + col;

        state.buttons[idx] = pressed;

        return Some(InputEvent::ButtonState(state.buttons.clone()));
    }

    None
}

fn parse_matrix(line: &str) -> Option<(usize, usize, bool)> {
    let parts: Vec<&str> = line.trim().split(',').collect();

    if parts.len() != 3 {
        return None;
    }

    let row = parts[0].parse::<usize>().ok()?;
    let col = parts[1].parse::<usize>().ok()?;
    let state = parts[2].parse::<u8>().ok()?;

    Some((row, col, state != 0))
}
