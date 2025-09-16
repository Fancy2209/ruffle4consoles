use ruffle_core::events::{GamepadButton, MouseButton};

pub struct AxisState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Default for AxisState {
    fn default() -> Self {
        AxisState {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

pub fn sdl_gamepadbutton_to_ruffle(button: sdl2::controller::Button) -> Option<GamepadButton> {
    return match button {
        sdl2::controller::Button::DPadUp => Some(GamepadButton::DPadUp),
        sdl2::controller::Button::DPadDown => Some(GamepadButton::DPadDown),
        sdl2::controller::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        sdl2::controller::Button::DPadRight => Some(GamepadButton::DPadRight),
        sdl2::controller::Button::A => Some(GamepadButton::South),
        sdl2::controller::Button::B => Some(GamepadButton::East),
        sdl2::controller::Button::X => Some(GamepadButton::West),
        sdl2::controller::Button::Y => Some(GamepadButton::North),
        sdl2::controller::Button::Start => Some(GamepadButton::Start),
        sdl2::controller::Button::Back => Some(GamepadButton::Select),
        sdl2::controller::Button::RightShoulder => Some(GamepadButton::RightTrigger),
        sdl2::controller::Button::LeftShoulder => Some(GamepadButton::LeftTrigger),
        _ => None,
    };
}

pub fn sdl_mousebutton_to_ruffle(button: sdl2::mouse::MouseButton) -> Option<MouseButton> {
    return match button {
        sdl2::mouse::MouseButton::Left => Some(MouseButton::Left),
        sdl2::mouse::MouseButton::Right => Some(MouseButton::Right),
        sdl2::mouse::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    };
}