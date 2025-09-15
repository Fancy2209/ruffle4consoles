mod log;

use std::collections::HashMap;
use std::time::Instant;

use anyhow::anyhow;
use log::VitaLogBackend;
use ruffle_core::config::Letterbox;
use ruffle_core::events::{GamepadButton, KeyCode, MouseButton};
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{PlayerBuilder, PlayerEvent, ViewportDimensions};
use ruffle_render::quality::StageQuality;
use ruffle_render_glow::GlowRenderBackend;

#[cfg(target_os = "horizon")]
use sdl2::libc;

#[cfg(target_os = "vita")]
#[link(name = "SDL2", kind = "static")]
#[link(name = "vitaGL", kind = "static")]
#[link(name = "stdc++", kind = "static")]
#[link(name = "vitashark", kind = "static")]
#[link(name = "SceShaccCg_stub", kind = "static")]
#[link(name = "mathneon", kind = "static")]
#[link(name = "SceShaccCgExt", kind = "static")]
#[link(name = "taihen_stub", kind = "static")]
#[link(name = "SceKernelDmacMgr_stub", kind = "static")]
#[link(name = "SceIme_stub", kind = "static")]
unsafe extern "C" {}

#[cfg(target_os = "horizon")]
unsafe extern "C" {
    pub fn randomGet(buf: *mut libc::c_void, len: libc::size_t);
    pub fn appletGetDefaultDisplayResolution(width: *mut i32, height: *mut i32) -> u32;
}

#[cfg(target_os = "horizon")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrandom(
    buf: *mut libc::c_void,
    mut buflen: libc::size_t,
    flags: libc::c_uint,
) -> libc::ssize_t {
    let maxlen = if flags & libc::GRND_RANDOM != 0 {
        512
    } else {
        0x1FF_FFFF
    };
    buflen = buflen.min(maxlen);
    unsafe {
        randomGet(buf, buflen);
    }
    buflen as libc::ssize_t
}

#[cfg(target_os = "horizon")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sysconf(name: i32) -> libc::c_long {
    if name == libc::_SC_PAGESIZE {
        return 4096;
    } else {
        return -1;
    }
}

#[cfg(target_os = "horizon")]
pub fn get_default_display_resolution() -> Result<(u32, u32), u32> {
    let mut width: i32 = 0;
    let mut height: i32 = 0;

    let rc = unsafe { appletGetDefaultDisplayResolution(&mut width, &mut height) };

    if rc == 0 {
        Ok((width as u32, height as u32))
    } else {
        Err(rc)
    }
}

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let sdl2_video = sdl2_context.video().unwrap();
    let sdl2_game_controller = sdl2_context.game_controller().unwrap();
    let sdl2_joystick = sdl2_context.joystick().unwrap();

    let gl_attr = sdl2_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_version(2, 0);
    
    let mut controllers: Vec<sdl2::controller::GameController> = Vec::new();
    println!("{}", sdl2_joystick.num_joysticks().unwrap());
    for i in 0..sdl2_joystick.num_joysticks().unwrap() {
        if sdl2_game_controller.is_game_controller(i) {
            controllers.push(sdl2_game_controller.open(i).unwrap());
        }
    }

    let mut last_frame_time: Instant;

    #[cfg(target_os = "vita")]
    let mut dimensions = ViewportDimensions {
        width: 940,
        height: 544,
        scale_factor: 1.0,
    };

    #[cfg(target_os = "horizon")]
    let (display_width, display_height) = get_default_display_resolution().unwrap();

    #[cfg(target_os = "horizon")]
    let mut dimensions = ViewportDimensions {
        width: display_width,
        height: display_height,
        scale_factor: 1.0,
    };

    #[cfg(not(any(target_os = "horizon", target_os = "vita")))]
    let mut dimensions = ViewportDimensions {
        width: 1280,
        height: 720,
        scale_factor: 1.0,
    };

    let sdl2_window = sdl2_video
        .window("ruffle4consoles", dimensions.width, dimensions.height)
        .opengl()
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let gl_context = sdl2_window.gl_create_context().unwrap();
    let _ = sdl2_window.gl_make_current(&gl_context);

    let bytes = include_bytes!("movie.swf");

    let movie = SwfMovie::from_data(bytes, "./movie.swf".to_string(), None)
        .map_err(|e| anyhow!(e.to_string()));
    let log = VitaLogBackend::default();

    // Glow can only realistically be used in vita and horizon, need
    let context: glow::Context;
    unsafe {
        context =
            glow::Context::from_loader_function(|s| sdl2_video.gl_get_proc_address(s) as *const _);
    }
    let renderer = GlowRenderBackend::new(context, false, StageQuality::High).unwrap();

    let mut gamepad_button_mapping: HashMap<GamepadButton, KeyCode> = HashMap::new();
    gamepad_button_mapping.insert(GamepadButton::DPadUp, KeyCode::UP);
    gamepad_button_mapping.insert(GamepadButton::DPadDown, KeyCode::DOWN);
    gamepad_button_mapping.insert(GamepadButton::DPadLeft, KeyCode::LEFT);
    gamepad_button_mapping.insert(GamepadButton::DPadRight, KeyCode::RIGHT);
    gamepad_button_mapping.insert(GamepadButton::South, KeyCode::S);
    gamepad_button_mapping.insert(GamepadButton::West, KeyCode::A);
    gamepad_button_mapping.insert(GamepadButton::East, KeyCode::UP);

    let player = PlayerBuilder::new()
        .with_log(log.clone())
        .with_renderer(renderer)
        .with_movie(movie.unwrap())
        .with_viewport_dimensions(dimensions.width, dimensions.height, dimensions.scale_factor)
        .with_fullscreen(true)
        .with_letterbox(Letterbox::On)
        .with_gamepad_button_mapping(gamepad_button_mapping)
        .with_autoplay(true)
        .build();
    last_frame_time = Instant::now();
    player.lock().unwrap().preload(&mut ExecutionLimit::none());

    let mut event_pump = sdl2_context.event_pump().unwrap();
    'main: loop {
        #[cfg(target_os = "horizon")]
        {
            let (nx_width, nx_height) = sdl2_window.drawable_size();
            if nx_width != dimensions.width && nx_height != dimensions.height {
                dimensions.width = nx_width;
                dimensions.height = nx_height;
                player.lock().unwrap().set_viewport_dimensions(dimensions);
            }
        }
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    if w > 0 && h > 0 {
                        dimensions.width = w as u32;
                        dimensions.height = h as u32;
                        player.lock().unwrap().set_viewport_dimensions(dimensions);
                    }
                }
                sdl2::event::Event::ControllerDeviceAdded {
                    timestamp: _,
                    which,
                } => {
                    controllers.push(sdl2_game_controller.open(which).unwrap());
                }
                sdl2::event::Event::ControllerDeviceRemoved {
                    timestamp: _,
                    which,
                } => {
                    if let Some(pos) = controllers.iter().position(|c| c.instance_id() == which) {
                        controllers.remove(pos); // drops the controller -> SDL closes it
                    }
                }
                sdl2::event::Event::ControllerButtonDown {
                    timestamp: _,
                    which: _,
                    button,
                } => {
                    println!("{}", button.string());
                    let ruffle_button = sdl_gamepadbutton_to_ruffle(button);
                    if let Some(ruffle_button) = ruffle_button {
                        player
                            .lock()
                            .unwrap()
                            .handle_event(PlayerEvent::GamepadButtonDown {
                                button: ruffle_button,
                            });
                    }
                }
                sdl2::event::Event::ControllerButtonUp {
                    timestamp: _,
                    which: _,
                    button,
                } => {
                    println!("{}", button.string());
                    let ruffle_button = sdl_gamepadbutton_to_ruffle(button);
                    if let Some(ruffle_button) = ruffle_button {
                        player
                            .lock()
                            .unwrap()
                            .handle_event(PlayerEvent::GamepadButtonUp {
                                button: ruffle_button,
                            });
                    }
                }
                sdl2::event::Event::MouseButtonDown {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x,
                    y,
                } => {
                    let ruffle_button = sdl_mousebutton_to_ruffle(mouse_btn);
                    if let Some(ruffle_button) = ruffle_button {
                        player.lock().unwrap().handle_event(PlayerEvent::MouseDown {
                            x: x.into(),
                            y: y.into(),
                            button: ruffle_button,
                            index: None,
                        });
                    }
                }
                sdl2::event::Event::MouseButtonUp {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x,
                    y,
                } => {
                    let ruffle_button = sdl_mousebutton_to_ruffle(mouse_btn);
                    if let Some(ruffle_button) = ruffle_button {
                        player.lock().unwrap().handle_event(PlayerEvent::MouseUp {
                            x: x.into(),
                            y: y.into(),
                            button: ruffle_button,
                        });
                    }
                }
                sdl2::event::Event::FingerDown {
                    timestamp: _,
                    touch_id: _,
                    finger_id: _,
                    x,
                    y,
                    dx: _,
                    dy: _,
                    pressure: _,
                } => {
                    player.lock().unwrap().handle_event(PlayerEvent::MouseDown {
                        x: x as f64 * dimensions.width as f64,
                        y: y as f64 * dimensions.height as f64,
                        button: MouseButton::Left,
                        index: None,
                    });
                }
                sdl2::event::Event::FingerUp {
                    timestamp: _,
                    touch_id: _,
                    finger_id: _,
                    x,
                    y,
                    dx: _,
                    dy: _,
                    pressure: _,
                } => {
                    player.lock().unwrap().handle_event(PlayerEvent::MouseUp {
                        x: x as f64 * dimensions.width as f64,
                        y: y as f64 * dimensions.height as f64,
                        button: MouseButton::Left,
                    });
                }
                _ => {}
            }
        }
        let new_time = Instant::now();
        let dt = new_time.duration_since(last_frame_time).as_micros();
        if dt > 0 {
            last_frame_time = new_time;
            if let Ok(mut player) = player.lock() {
                player.tick(dt as f64 / 1000.0);
                if player.needs_render() {
                    player.render();
                    sdl2_window.gl_swap_window();
                }
            }
        }
    }
    drop(controllers);
}

fn sdl_gamepadbutton_to_ruffle(button: sdl2::controller::Button) -> Option<GamepadButton> {
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

fn sdl_mousebutton_to_ruffle(button: sdl2::mouse::MouseButton) -> Option<MouseButton> {
    return match button {
        sdl2::mouse::MouseButton::Left => Some(MouseButton::Left),
        sdl2::mouse::MouseButton::Right => Some(MouseButton::Right),
        sdl2::mouse::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    };
}
