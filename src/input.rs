pub mod keyboard;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorEvent {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub state: MouseState,
    pub button: MouseButton,
}

impl MouseEvent {
    pub(crate) fn new(state: impl Into<MouseState>, button: impl Into<MouseButton>) -> Self {
        Self {
            state: state.into(),
            button: button.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseState {
    Pressed,
    Released,
}

impl From<winit::event::ElementState> for MouseState {
    fn from(value: winit::event::ElementState) -> Self {
        match value {
            winit::event::ElementState::Pressed => MouseState::Pressed,
            winit::event::ElementState::Released => MouseState::Released,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(other) => MouseButton::Other(other),
        }
    }
}
