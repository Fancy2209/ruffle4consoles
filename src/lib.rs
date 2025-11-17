#![allow(unused_variables)]
#![allow(dead_code)]

mod backends;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::anyhow;
use ruffle_core::config::Letterbox;
use ruffle_core::events::MouseButton;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent, ViewportDimensions};
use ruffle_render::quality::StageQuality;
use ruffle_render_glow::GlowRenderBackend;

use backends::audio::SdlAudioBackend;
use ruffle_frontend_utils::backends::storage::DiskStorageBackend;

struct ActivePlayer {
    player: Arc<Mutex<Player>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn SDL_main(_argc: i32, _argv: *const *const i8) -> i32 {
    main();
    return 0;
}

pub fn main() {
    let mut last_frame_time: Instant;

    sdl2::hint::set("SDL_IOS_ORIENTATIONS", "LandscapeLeft LandscapeRight");
    //sdl2::hint::set("SDL_ANDROID_BLOCK_ON_PAUSE", "1");
    sdl2::hint::set("SDL_MOUSE_TOUCH_EVENTS", "0");
    sdl2::hint::set("SDL_TOUCH_MOUSE_EVENTS", "0");
    let sdl2_context = sdl2::init().unwrap();
    let sdl2_video = sdl2_context.video().unwrap();

    let gl_attr = sdl2_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_version(2, 0);
    let _ = sdl2_video.gl_set_swap_interval(0);

    let mut dimensions = ViewportDimensions {
        width: 1024,
        height: 768,
        scale_factor: 1.0,
    };

    let sdl2_window = sdl2_video
        .window("Matt's Hidden Cats", dimensions.width, dimensions.height)
        .opengl()
        .resizable()
        .fullscreen_desktop()
        .borderless()
        .position_centered()
        .build()
        .unwrap();

    (dimensions.width, dimensions.height) = sdl2_window.size();

    let gl_context = sdl2_window.gl_create_context().unwrap();
    let _ = sdl2_window.gl_make_current(&gl_context);

    let swf_url = "file:///movie.swf";

    let swf_data = include_bytes!("Matts Hidden Cats.swf");
    let movie =
        SwfMovie::from_data(swf_data, swf_url.to_string(), None).map_err(|e| anyhow!(e.to_string()));

    let context = Arc::new(unsafe {
        glow::Context::from_loader_function(|s| sdl2_video.gl_get_proc_address(s) as *const _)
    });
    let renderer = GlowRenderBackend::new(context, false, StageQuality::High).unwrap();
    let audio = SdlAudioBackend::new(sdl2_context.audio().unwrap()).unwrap();

    let base_path = sdl2::filesystem::pref_path("KupoGames", "MattsHiddenCats");
    let storage_path = format!("{}/{}", base_path.unwrap(), "Saves");
    println!("{}", storage_path);
    let _ = std::fs::create_dir_all(storage_path.clone());

    let player = PlayerBuilder::new()
        .with_renderer(renderer)
        .with_audio(audio)
        .with_storage(Box::new(DiskStorageBackend::new(std::path::PathBuf::from(
            storage_path,
        ))))
        .with_movie(movie.unwrap())
        .with_viewport_dimensions(dimensions.width, dimensions.height, dimensions.scale_factor)
        .with_fullscreen(false)
        .with_letterbox(Letterbox::Off)
        .with_autoplay(true)
        .build();
    last_frame_time = Instant::now();
    player.lock().unwrap().preload(&mut ExecutionLimit::none());

    let mut event_pump = sdl2_context.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                
                // Prevent issues
                sdl2::event::Event::AppWillEnterBackground { .. } => {
                    player.lock().unwrap().handle_event(PlayerEvent::FocusGained);
                },
                sdl2::event::Event::AppWillEnterForeground { .. } => {
                    player.lock().unwrap().handle_event(PlayerEvent::FocusLost);
                },
                
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
                sdl2::event::Event::MouseMotion {
                    x,
                    y,
                    ..
                } => {
                    player.lock().unwrap().handle_event(PlayerEvent::MouseMove {
                        x: x.into(),
                        y: y.into(),
                    });
                }
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn,
                    x,
                    y,
                    ..
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
                    mouse_btn,    
                    x,
                    y,
                    ..
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
                // TODO: Implement sdl2::event::Event::TextInput and UI Backend
                sdl2::event::Event::FingerMotion {
                  x,
                  y,
                  ..
                } => {
                     player.lock().unwrap().handle_event(PlayerEvent::MouseMove {
                            x: x as f64 * dimensions.width as f64,
                            y: y as f64 * dimensions.height as f64
                        });
                }
                sdl2::event::Event::FingerDown {
                    x,
                    y,
                    ..
                } => {
                    player.lock().unwrap().handle_event(PlayerEvent::MouseDown {
                        x: x as f64 * dimensions.width as f64,
                        y: y as f64 * dimensions.height as f64,
                        button: MouseButton::Left,
                        index: None,
                    });
                }
                sdl2::event::Event::FingerUp {
                    x,
                    y,
                    ..
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
}


#[cfg(not(any(target_os = "horizon", target_os = "vita")))]
fn sdl_mousebutton_to_ruffle(button: sdl2::mouse::MouseButton) -> Option<MouseButton> {
    match button {
        sdl2::mouse::MouseButton::Left => Some(MouseButton::Left),
        sdl2::mouse::MouseButton::Right => Some(MouseButton::Right),
        sdl2::mouse::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}
