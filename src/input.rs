//! HarbourOS system input API.
//!
//! SDL events are translated into small OS-level input records before apps or
//! shell surfaces consume them. Pointer coordinates are integer pixels only.

use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyInput {
    pub key: Keycode,
    pub modifiers: Mod,
}

impl KeyInput {
    pub fn new(key: Keycode, modifiers: Mod) -> Self {
        Self { key, modifiers }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
}

impl PointerButton {
    pub fn from_sdl(button: MouseButton) -> Option<Self> {
        match button {
            MouseButton::Left => Some(Self::Left),
            MouseButton::Middle => Some(Self::Middle),
            MouseButton::Right => Some(Self::Right),
            _ => None,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Middle => 1,
            Self::Right => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerAction {
    Move,
    ButtonDown,
    ButtonUp,
    Wheel,
    Enter,
    Leave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerInput {
    pub action: PointerAction,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub button: Option<PointerButton>,
    pub clicks: u8,
    pub wheel_x: i32,
    pub wheel_y: i32,
}

impl PointerInput {
    pub fn move_to(x: i32, y: i32, dx: i32, dy: i32) -> Self {
        Self {
            action: PointerAction::Move,
            x,
            y,
            dx,
            dy,
            button: None,
            clicks: 0,
            wheel_x: 0,
            wheel_y: 0,
        }
    }

    pub fn button(
        action: PointerAction,
        x: i32,
        y: i32,
        button: PointerButton,
        clicks: u8,
    ) -> Self {
        Self {
            action,
            x,
            y,
            dx: 0,
            dy: 0,
            button: Some(button),
            clicks,
            wheel_x: 0,
            wheel_y: 0,
        }
    }

    pub fn wheel(x: i32, y: i32, wheel_x: i32, wheel_y: i32) -> Self {
        Self {
            action: PointerAction::Wheel,
            x,
            y,
            dx: 0,
            dy: 0,
            button: None,
            clicks: 0,
            wheel_x,
            wheel_y,
        }
    }

    pub fn boundary(action: PointerAction, x: i32, y: i32) -> Self {
        Self {
            action,
            x,
            y,
            dx: 0,
            dy: 0,
            button: None,
            clicks: 0,
            wheel_x: 0,
            wheel_y: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerState {
    pub x: i32,
    pub y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub inside_window: bool,
    buttons: [bool; 3],
}

impl PointerState {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            prev_x: x,
            prev_y: y,
            inside_window: true,
            buttons: [false; 3],
        }
    }

    pub fn apply(&mut self, input: PointerInput) {
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.x = input.x;
        self.y = input.y;

        match input.action {
            PointerAction::ButtonDown => {
                if let Some(button) = input.button {
                    self.buttons[button.index()] = true;
                }
            }
            PointerAction::ButtonUp => {
                if let Some(button) = input.button {
                    self.buttons[button.index()] = false;
                }
            }
            PointerAction::Enter => self.inside_window = true,
            PointerAction::Leave => self.inside_window = false,
            PointerAction::Move | PointerAction::Wheel => {}
        }
    }

    pub fn clamp_to(&mut self, width: u32, height: u32) {
        let max_x = (width as i32 - 1).max(0);
        let max_y = (height as i32 - 1).max(0);
        self.x = self.x.clamp(0, max_x);
        self.y = self.y.clamp(0, max_y);
        self.prev_x = self.prev_x.clamp(0, max_x);
        self.prev_y = self.prev_y.clamp(0, max_y);
    }

    pub fn is_down(&self, button: PointerButton) -> bool {
        self.buttons[button.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputState {
    pub pointer: PointerState,
    pub last_key: Option<KeyInput>,
    pub last_pointer: Option<PointerInput>,
}

impl InputState {
    pub fn new(pointer_x: i32, pointer_y: i32) -> Self {
        Self {
            pointer: PointerState::new(pointer_x, pointer_y),
            last_key: None,
            last_pointer: None,
        }
    }

    pub fn record_key(&mut self, input: KeyInput) {
        self.last_key = Some(input);
    }

    pub fn record_pointer(&mut self, input: PointerInput) {
        self.pointer.apply(input);
        self.last_pointer = Some(input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_state_tracks_motion_buttons_and_clamp() {
        let mut input = InputState::new(4, 5);
        input.record_pointer(PointerInput::move_to(20, 30, 16, 25));
        input.record_pointer(PointerInput::button(
            PointerAction::ButtonDown,
            20,
            30,
            PointerButton::Left,
            1,
        ));

        assert_eq!(input.pointer.x, 20);
        assert_eq!(input.pointer.y, 30);
        assert!(input.pointer.is_down(PointerButton::Left));

        input.pointer.clamp_to(10, 12);
        assert_eq!(input.pointer.x, 9);
        assert_eq!(input.pointer.y, 11);

        input.record_pointer(PointerInput::button(
            PointerAction::ButtonUp,
            9,
            11,
            PointerButton::Left,
            1,
        ));
        assert!(!input.pointer.is_down(PointerButton::Left));
    }
}
