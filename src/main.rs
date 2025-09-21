mod audio;
mod log;

use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use crate::audio::SdlAudioBackend;
use crate::log::ConsoleLogBackend;
use anyhow::anyhow;
use ron::de::from_reader;
use ron::from_str;
use ruffle_core::config::Letterbox;
use ruffle_core::events::{GamepadButton, KeyCode, MouseButton, ParseEnumError};
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{PlayerBuilder, PlayerEvent, ViewportDimensions};
use ruffle_render::quality::StageQuality;
use ruffle_render_glow::GlowRenderBackend;
use sdl2::controller::Axis;
use serde::Deserialize;

//#[cfg(any(target_os = "vita", target_os = "horizon"))]
#[cfg(target_os = "horizon")]
use core::ffi::c_void;

#[cfg(target_os = "vita")]
type SceGxmMultisampleMode = u32;
#[cfg(target_os = "vita")]
pub const SCE_GXM_MULTISAMPLE_NONE: SceGxmMultisampleMode = 0;
#[cfg(target_os = "vita")]
pub const SCE_GXM_MULTISAMPLE_2X: SceGxmMultisampleMode = 1;
#[cfg(target_os = "vita")]
pub const SCE_GXM_MULTISAMPLE_4X: SceGxmMultisampleMode = 2;

//#[cfg(target_os = "vita")]
//static VGL_MODE_SHADER_PAIR:u32 = 0;
//#[cfg(target_os = "vita")]
//static VGL_MODE_GLOBAL:u32 = 1;
#[cfg(target_os = "vita")]
static VGL_MODE_POSTPONED: u32 = 2;

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
unsafe extern "C" {
    pub fn vglInitWithCustomThreshold(
        pool_size: i32,
        width: i32,
        height: i32,
        ram_reteshold: i32,
        cdram_threshold: i32,
        phycont_threshold: i32,
        cdlg_threshold: i32,
        msaa: SceGxmMultisampleMode,
    ) -> bool;
    pub fn vglSetSemanticBindingMode(mode: u32);
    pub fn vglSetParamBufferSize(size: u32);
    pub fn vglUseCachedMem(r#use: bool);
    pub fn vglUseTripleBuffering(usage: bool);
    //pub fn vglCalloc(nobj: usize, size: usize) -> *mut c_void;
    //pub fn vglFree(p: *mut c_void);
    //pub fn vglMalloc(size: usize) -> *mut c_void;
    //pub fn vglMemalign(align: usize, size: usize) -> *mut c_void;
    //pub fn vglRealloc(p: *mut c_void, size: usize) -> *mut c_void;
    //pub fn sceClibMemcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    //pub fn sceClibMemset(dest: *mut c_void, c: i32, n: usize) -> *mut c_void;
}

/*
#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_calloc(nobj: usize, size: usize) -> *mut c_void {
    unsafe { vglCalloc(nobj, size) }
}

#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_free(p: *mut c_void) {
    unsafe { vglFree(p) };
}

#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_malloc(size: usize) -> *mut c_void {
    unsafe { vglMalloc(size) }
}

#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_memalign(align: usize, size: usize) -> *mut c_void {
    unsafe { vglMemalign(align, size) }
}

#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_realloc(p: *mut c_void, size: usize) -> *mut c_void {
    unsafe { vglRealloc(p, size) }
}

#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_memcpy(
    dest: *mut c_void,
    src: *const c_void,
    n: usize,
) -> *mut c_void {
    unsafe { sceClibMemcpy(dest, src, n) }
}

#[cfg(target_os = "vita")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __wrap_memset(dest: *mut c_void, c: i32, n: usize) -> *mut c_void {
    unsafe { sceClibMemset(dest, c, n) }
}*/

//#[cfg(target_os = "vita")]
//#[used]
//#[unsafe(export_name = "sceUserMainThreadStackSize")]
//pub static SCE_USER_MAIN_THREAD_STACK_SIZE: u32 = 1 * 1024 * 1024; // 1 MiB

//#[cfg(target_os = "vita")]
//#[used]
//#[unsafe(export_name = "sceLibcHeapSize")]
//pub static SCE_LIBC_HEAP_SIZE: u32 = 10 * 1024 * 1024; // 10 MiB

//#[cfg(target_os = "vita")]
//#[used]
//#[unsafe(export_name = "_newlib_heap_size_user")]
//pub static _NEWLIB_HEAP_SIZE_USER: u32 = 246 * 1024 * 1024;

#[cfg(target_os = "horizon")]
unsafe extern "C" {
    pub fn randomGet(buf: *mut c_void, len: usize);
    pub fn appletGetDefaultDisplayResolution(width: *mut i32, height: *mut i32) -> u32;
}

#[cfg(target_os = "horizon")]
static _SC_PAGESIZE: i32 = 30;

#[cfg(target_os = "horizon")]
static GRND_RANDOM: u32 = 0x2;

#[cfg(target_os = "horizon")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrandom(
    buf: *mut c_void,
    mut buflen: usize,
    flags: u32,
) -> isize {
    let maxlen = if flags & GRND_RANDOM != 0 {
        512
    } else {
        0x1FF_FFFF
    };
    buflen = buflen.min(maxlen);
    unsafe {
        randomGet(buf, buflen);
    }
    buflen as isize
}

#[cfg(target_os = "horizon")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sysconf(name: i32) -> i64 {
    if name == _SC_PAGESIZE {
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

#[cfg(target_os = "vita")]
const BASE_PATH: &str = "ux0:data/ruffle";

#[cfg(target_os = "horizon")]
const BASE_PATH: &str = "/switch/ruffle";

#[cfg(not(any(target_os = "horizon", target_os = "vita")))]
const BASE_PATH: &str = "./ruffle";

const CONFIG: &str = "
Config(
    gamepad_config: {},
)";

#[derive(Debug, Deserialize)]
struct Config {
    gamepad_config: HashMap<String, u32>,
    swf_url: Option<String>,
    swf_name: Option<String>,
}

fn load_config() -> Result<
    (
        HashMap<GamepadButton, KeyCode>,
        Option<String>,
        Option<String>,
    ),
    ParseEnumError,
> {
    let config_file = format!("{}/config.ron", BASE_PATH);
    let config_file_clone = config_file.clone();
    let f = File::open(config_file);
    if f.is_ok() {
        let config: Config = match from_reader(f.unwrap()) {
            Ok(x) => x,
            Err(e) => {
                println!("Couldn't load config file:{}", config_file_clone);
                println!("{}", e);
                from_str(CONFIG).unwrap()
            }
        };
        let mut gamepad_button_mapping: HashMap<GamepadButton, KeyCode> = HashMap::new();
        for (button, key) in config.gamepad_config.into_iter() {
            gamepad_button_mapping
                .insert(GamepadButton::from_str(&button)?, KeyCode::from_code(key));
        }
        Ok((gamepad_button_mapping, config.swf_name, config.swf_url))
    } else {
        println!("Couldn't load config file:{}", config_file_clone);
        let config: Config = from_str(CONFIG).unwrap();
        let mut gamepad_button_mapping: HashMap<GamepadButton, KeyCode> = HashMap::new();
        for (button, key) in config.gamepad_config.into_iter() {
            gamepad_button_mapping
                .insert(GamepadButton::from_str(&button)?, KeyCode::from_code(key));
        }
        Ok((gamepad_button_mapping, config.swf_name, config.swf_url))
    }
}

fn main() {
    sdl2::hint::set("SDL_TOUCH_MOUSE_EVENTS", "0");

    let mut axis_state = AxisState::default();
    let sdl2_context = sdl2::init().unwrap();
    let sdl2_video = sdl2_context.video().unwrap();
    let sdl2_game_controller = sdl2_context.game_controller().unwrap();
    let sdl2_joystick = sdl2_context.joystick().unwrap();

    // SDL2's default vitaGL config isn't ideal, so we gotta get a little unsafe
    #[cfg(target_os = "vita")]
    unsafe {
        vglSetParamBufferSize(4 * 1024 * 1024);
        vglUseCachedMem(true);
        vglUseTripleBuffering(false);
        vglSetSemanticBindingMode(VGL_MODE_POSTPONED);
        vglInitWithCustomThreshold(
            0,
            960,
            544,
            4 * 1024 * 1024,
            0,
            0,
            0,
            SCE_GXM_MULTISAMPLE_2X,
        );
    }

    let gl_attr = sdl2_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_version(2, 0);
    let _ = sdl2_video.gl_set_swap_interval(0);

    let config = match load_config() {
        Ok(x) => x,
        Err(_e) => {
            println!("Couldn't load default config");
            std::process::exit(1);
        }
    };

    let (gamepad_button_mapping, swf_name, swf_url) = config;

    let mut controllers: Vec<sdl2::controller::GameController> = Vec::new();
    for i in 0..sdl2_joystick.num_joysticks().unwrap() {
        if sdl2_game_controller.is_game_controller(i) {
            controllers.push(sdl2_game_controller.open(i).unwrap());
        }
    }

    let mut last_frame_time: Instant;

    #[cfg(target_os = "vita")]
    let mut dimensions = ViewportDimensions {
        width: 960,
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
    let swf_name = if swf_name.is_some() {
        swf_name.unwrap()
    } else {
        "movie.swf".into()
    };
    let swf_url = if swf_url.is_some() {
        swf_url.unwrap()
    } else {
        "file:///movie.swf".into()
    };

    let swf_data = std::fs::read(format!("{}/{}", BASE_PATH, swf_name));
    let movie = SwfMovie::from_data(&swf_data.unwrap(), swf_url.into(), None)
        .map_err(|e| anyhow!(e.to_string()));

    if movie.is_err() {
        println!("Couldn't load {}", format!("{}/{}", BASE_PATH, swf_name));
        std::process::exit(1);
    }
    let log = ConsoleLogBackend::default();

    // Glow can only realistically be used in vita and horizon, need
    let context = Arc::new(unsafe {
        glow::Context::from_loader_function(|s| sdl2_video.gl_get_proc_address(s) as *const _)
    });
    let renderer = GlowRenderBackend::new(context, false, StageQuality::High).unwrap();
    let audio = SdlAudioBackend::new(sdl2_context.audio().unwrap()).unwrap();

    let player = PlayerBuilder::new()
        .with_log(log.clone())
        .with_renderer(renderer)
        .with_audio(audio)
        .with_movie(movie.unwrap())
        .with_viewport_dimensions(dimensions.width, dimensions.height, dimensions.scale_factor)
        .with_scale_mode(ruffle_core::StageScaleMode::ShowAll, true)
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
                sdl2::event::Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => {
                    let x_axis = axis == Axis::LeftX;
                    let y_axis = axis == Axis::LeftY;
                    let deadzone = 8000;
                    let left = if x_axis {
                        value < -deadzone
                    } else {
                        axis_state.left
                    };
                    let right = if x_axis {
                        value > deadzone
                    } else {
                        axis_state.right
                    };
                    let up = if y_axis {
                        value < -deadzone
                    } else {
                        axis_state.up
                    };
                    let down = if y_axis {
                        value > deadzone
                    } else {
                        axis_state.down
                    };

                    if up != axis_state.up {
                        let event_up = if up {
                            PlayerEvent::GamepadButtonDown {
                                button: GamepadButton::DPadUp,
                            }
                        } else {
                            PlayerEvent::GamepadButtonUp {
                                button: GamepadButton::DPadUp,
                            }
                        };
                        axis_state.up = up;
                        player.lock().unwrap().handle_event(event_up);
                    }
                    if down != axis_state.down {
                        let event_down = if down {
                            PlayerEvent::GamepadButtonDown {
                                button: GamepadButton::DPadDown,
                            }
                        } else {
                            PlayerEvent::GamepadButtonUp {
                                button: GamepadButton::DPadDown,
                            }
                        };
                        axis_state.down = down;
                        player.lock().unwrap().handle_event(event_down);
                    }
                    if left != axis_state.left {
                        let event_left = if left {
                            PlayerEvent::GamepadButtonDown {
                                button: GamepadButton::DPadLeft,
                            }
                        } else {
                            PlayerEvent::GamepadButtonUp {
                                button: GamepadButton::DPadLeft,
                            }
                        };
                        axis_state.left = left;
                        player.lock().unwrap().handle_event(event_left);
                    }
                    if right != axis_state.right {
                        let event_right = if right {
                            PlayerEvent::GamepadButtonDown {
                                button: GamepadButton::DPadRight,
                            }
                        } else {
                            PlayerEvent::GamepadButtonUp {
                                button: GamepadButton::DPadRight,
                            }
                        };
                        axis_state.right = right;
                        player.lock().unwrap().handle_event(event_right);
                    }
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
