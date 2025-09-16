#![allow(clippy::redundant_field_names)]

extern crate sdl2;
extern crate egui;

use egui::OutputCommand;
use egui::{Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect};
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use sdl2::mouse::{Cursor, MouseButton, SystemCursor};
use sdl2::video::Window;

pub struct FusedCursor {
    pub cursor: sdl2::mouse::Cursor,
    pub icon: sdl2::mouse::SystemCursor,
}

impl FusedCursor {
    pub fn new() -> Self {
        Self {
            cursor: sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::Arrow).unwrap(),
            icon: sdl2::mouse::SystemCursor::Arrow,
        }
    }
}

impl Default for FusedCursor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn translate_virtual_key_code(key: Keycode) -> Option<egui::Key> {
    Some(match key {
        Keycode::Left => Key::ArrowLeft,
        Keycode::Up => Key::ArrowUp,
        Keycode::Right => Key::ArrowRight,
        Keycode::Down => Key::ArrowDown,

        Keycode::Escape => Key::Escape,
        Keycode::Tab => Key::Tab,
        Keycode::Backspace => Key::Backspace,
        Keycode::Space => Key::Space,
        Keycode::Return => Key::Enter,

        Keycode::Insert => Key::Insert,
        Keycode::Home => Key::Home,
        Keycode::Delete => Key::Delete,
        Keycode::End => Key::End,
        Keycode::PageDown => Key::PageDown,
        Keycode::PageUp => Key::PageUp,

        Keycode::Kp0 | Keycode::Num0 => Key::Num0,
        Keycode::Kp1 | Keycode::Num1 => Key::Num1,
        Keycode::Kp2 | Keycode::Num2 => Key::Num2,
        Keycode::Kp3 | Keycode::Num3 => Key::Num3,
        Keycode::Kp4 | Keycode::Num4 => Key::Num4,
        Keycode::Kp5 | Keycode::Num5 => Key::Num5,
        Keycode::Kp6 | Keycode::Num6 => Key::Num6,
        Keycode::Kp7 | Keycode::Num7 => Key::Num7,
        Keycode::Kp8 | Keycode::Num8 => Key::Num8,
        Keycode::Kp9 | Keycode::Num9 => Key::Num9,

        Keycode::A => Key::A,
        Keycode::B => Key::B,
        Keycode::C => Key::C,
        Keycode::D => Key::D,
        Keycode::E => Key::E,
        Keycode::F => Key::F,
        Keycode::G => Key::G,
        Keycode::H => Key::H,
        Keycode::I => Key::I,
        Keycode::J => Key::J,
        Keycode::K => Key::K,
        Keycode::L => Key::L,
        Keycode::M => Key::M,
        Keycode::N => Key::N,
        Keycode::O => Key::O,
        Keycode::P => Key::P,
        Keycode::Q => Key::Q,
        Keycode::R => Key::R,
        Keycode::S => Key::S,
        Keycode::T => Key::T,
        Keycode::U => Key::U,
        Keycode::V => Key::V,
        Keycode::W => Key::W,
        Keycode::X => Key::X,
        Keycode::Y => Key::Y,
        Keycode::Z => Key::Z,

        _ => {
            return None;
        }
    })
}

pub struct EguiSDL2State {
    pub raw_input: RawInput,
    pub modifiers: Modifiers,
    pub dpi_scaling: f32,
    pub mouse_pointer_position: egui::Pos2,
    pub fused_cursor: FusedCursor,
}

impl EguiSDL2State {
    pub fn sdl2_input_to_egui(&mut self, window: &sdl2::video::Window, event: &sdl2::event::Event) {
        fn sdl_button_to_egui(btn: &MouseButton) -> Option<PointerButton> {
            match btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            }
        }

        use sdl2::event::Event::*;
        let pixels_per_point = self.dpi_scaling;
        if event.get_window_id() != Some(window.id()) {
            return;
        }
        match event {
            // handle when window Resized and SizeChanged.
            Window { win_event, .. } => match win_event {
                WindowEvent::Resized(x, y) | sdl2::event::WindowEvent::SizeChanged(x, y) => {
                    self.update_screen_rect(*x as u32, *y as u32);
                }
                _ => (),
            },
            MouseButtonDown { mouse_btn, .. } => {
                if let Some(pressed) = sdl_button_to_egui(mouse_btn) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.mouse_pointer_position,
                        button: pressed,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
            }
            MouseButtonUp { mouse_btn, .. } => {
                if let Some(released) = sdl_button_to_egui(mouse_btn) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.mouse_pointer_position,
                        button: released,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
            }

            MouseMotion { x, y, .. } => {
                self.mouse_pointer_position =
                    egui::pos2(*x as f32 / pixels_per_point, *y as f32 / pixels_per_point);
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.mouse_pointer_position));
            }

            KeyUp {
                keycode, keymod, ..
            } => {
                let key_code = match keycode {
                    Some(key_code) => key_code,
                    _ => return,
                };
                let key = match translate_virtual_key_code(*key_code) {
                    Some(key) => key,
                    _ => return,
                };
                self.modifiers = Modifiers {
                    alt: (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (*keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: *keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                self.raw_input.events.push(egui::Event::Key {
                    key: key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: self.modifiers,
                });
            }

            KeyDown {
                keycode, keymod, ..
            } => {
                let key_code = match keycode {
                    Some(key_code) => key_code,
                    _ => return,
                };

                let key = match translate_virtual_key_code(*key_code) {
                    Some(key) => key,
                    _ => return,
                };
                self.modifiers = Modifiers {
                    alt: (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (*keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: *keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                self.raw_input.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: self.modifiers,
                });

                if self.modifiers.command && key == Key::C {
                    // println!("copy event");
                    self.raw_input.events.push(egui::Event::Copy);
                } else if self.modifiers.command && key == Key::X {
                    // println!("cut event");
                    self.raw_input.events.push(egui::Event::Cut);
                } else if self.modifiers.command && key == Key::V {
                    // println!("paste");
                    if let Ok(contents) = window.subsystem().clipboard().clipboard_text() {
                        self.raw_input.events.push(egui::Event::Text(contents));
                    }
                }
            }

            TextInput { text, .. } => {
                self.raw_input.events.push(egui::Event::Text(text.clone()));
            }
            MouseWheel { x, y, .. } => {
                let delta = egui::vec2(*x as f32 * 8.0, *y as f32 * 8.0);
                let sdl = window.subsystem().sdl();
                // zoom:
                if sdl.keyboard().mod_state() & Mod::LCTRLMOD == Mod::LCTRLMOD
                    || sdl.keyboard().mod_state() & Mod::RCTRLMOD == Mod::RCTRLMOD
                {
                    let zoom_delta = (delta.y / 125.0).exp();
                    self.raw_input.events.push(egui::Event::Zoom(zoom_delta));
                }
                // horizontal scroll:
                else if sdl.keyboard().mod_state() & Mod::LSHIFTMOD == Mod::LSHIFTMOD
                    || sdl.keyboard().mod_state() & Mod::RSHIFTMOD == Mod::RSHIFTMOD
                {
                    let e = egui::Event::MouseWheel {
                        unit: MouseWheelUnit::Point,
                        delta: egui::vec2(delta.x + delta.y, 0.0),
                        modifiers: Default::default(),
                    };
                    self.raw_input.events.push(e);
                    // regular scroll:
                } else {
                    let e = egui::Event::MouseWheel {
                        unit: MouseWheelUnit::Point,
                        delta: egui::vec2(delta.x, delta.y),
                        modifiers: Default::default(),
                    };
                    self.raw_input.events.push(e)
                }
            }
            _ => {}
        }
    }

    pub fn update_screen_rect(&mut self, width: u32, height: u32) {
        let inv_scale = 1.0 / self.dpi_scaling;
        let rect = egui::vec2(width as f32 * inv_scale, height as f32 * inv_scale);
        self.raw_input.screen_rect = Some(Rect::from_min_size(Pos2::new(0f32, 0f32), rect));
    }

    pub fn update_time(&mut self, running_time: Option<f64>, delta: f32) {
        self.raw_input.time = running_time;
        self.raw_input.predicted_dt = delta;
    }

    pub fn new(width: u32, height: u32, dpi_scaling: f32) -> Self {
        let inv_scale = 1.0 / dpi_scaling;
        let rect = egui::vec2(width as f32 * inv_scale, height as f32 * inv_scale);
        let screen_rect = Rect::from_min_size(Pos2::new(0f32, 0f32), rect);
        let raw_input = RawInput {
            screen_rect: Some(screen_rect),
            ..RawInput::default()
        };
        let modifiers = Modifiers::default();
        EguiSDL2State {
            raw_input,
            modifiers,
            dpi_scaling,
            mouse_pointer_position: egui::Pos2::new(0.0, 0.0),
            fused_cursor: FusedCursor::new(),
        }
    }

    pub fn process_output(&mut self, window: &Window, egui_output: &egui::PlatformOutput) {
        for command in &egui_output.commands {
            if let OutputCommand::CopyText(copied_text) = command {
                let result = window
                    .subsystem()
                    .clipboard()
                    .set_clipboard_text(&copied_text);
                if result.is_err() {
                    dbg!("Unable to set clipboard content to SDL clipboard.");
                }
            }
        }
        EguiSDL2State::translate_cursor(&mut self.fused_cursor, egui_output.cursor_icon);
    }

    fn translate_cursor(fused: &mut FusedCursor, cursor_icon: egui::CursorIcon) {
        let tmp_icon = match cursor_icon {
            egui::CursorIcon::Crosshair => SystemCursor::Crosshair,
            egui::CursorIcon::Default => SystemCursor::Arrow,
            egui::CursorIcon::Grab => SystemCursor::Hand,
            egui::CursorIcon::Grabbing => SystemCursor::SizeAll,
            egui::CursorIcon::Move => SystemCursor::SizeAll,
            egui::CursorIcon::PointingHand => SystemCursor::Hand,
            egui::CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
            egui::CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
            egui::CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
            egui::CursorIcon::ResizeVertical => SystemCursor::SizeNS,
            egui::CursorIcon::Text => SystemCursor::IBeam,
            egui::CursorIcon::NotAllowed | egui::CursorIcon::NoDrop => SystemCursor::No,
            egui::CursorIcon::Wait => SystemCursor::Wait,
            //There doesn't seem to be a suitable SDL equivalent...
            _ => SystemCursor::Arrow,
        };

        if tmp_icon != fused.icon {
            fused.cursor = Cursor::from_system(tmp_icon).unwrap();
            fused.icon = tmp_icon;
            fused.cursor.set();
        }
    }
}
