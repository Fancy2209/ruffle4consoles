use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};

use egui_file_dialog::FileDialog;
use egui_glow::{glow::HasContext, Painter, ShaderVersion};
use egui_sdl2_event::EguiSDL2State;
use ruffle_core::{
    Player, PlayerBuilder, PlayerEvent, ViewportDimensions,
    config::Letterbox,
    events::{GamepadButton, KeyCode, MouseButton},
    limits::ExecutionLimit,
    tag_utils::SwfMovie,
};
use ruffle_render::quality::StageQuality;
use ruffle_render_glow::GlowRenderBackend;
use sdl2::controller::Axis;
use std::sync::Mutex;

use crate::app::log::ConsoleLogBackend;
use crate::app::{
    audio::SdlAudioBackend,
    input::{AxisState, sdl_gamepadbutton_to_ruffle, sdl_mousebutton_to_ruffle},
};

mod audio;
mod input;
mod log;

pub struct App {
    sdl2_context: sdl2::Sdl,
    sdl2_window: Option<sdl2::video::Window>,
    glow_context: Option<Arc<egui_glow::glow::Context>>,
    egui_sdl2_state: Option<EguiSDL2State>,
    egui_painter: Option<egui_glow::Painter>,
    egui_ctx: Option<egui::Context>,
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
    dimensions: ViewportDimensions,
    player: Option<Arc<Mutex<Player>>>,
    controllers: Vec<sdl2::controller::GameController>,
    axis_state: AxisState,
    last_frame_time: Instant,
    gamepad_button_mapping: HashMap<GamepadButton, KeyCode>,
}

impl App {
    pub fn new() -> Self {
        #[cfg(target_os = "vita")]
        let dimensions = ViewportDimensions {
            width: 940,
            height: 544,
            scale_factor: 1.0,
        };

        #[cfg(target_os = "horizon")]
        let (display_width, display_height) = get_default_display_resolution().unwrap();

        #[cfg(target_os = "horizon")]
        let dimensions = ViewportDimensions {
            width: display_width,
            height: display_height,
            scale_factor: 1.0,
        };

        #[cfg(not(any(target_os = "horizon", target_os = "vita")))]
        let dimensions = ViewportDimensions {
            width: 1280,
            height: 720,
            scale_factor: 1.0,
        };

        Self {
            sdl2_context: sdl2::init().unwrap(),
            sdl2_window: None,
            glow_context: None,
            egui_sdl2_state: None,
            egui_painter: None,
            egui_ctx: None,
            dimensions: dimensions,
            player: None,
            controllers: Vec::new(),
            gamepad_button_mapping: HashMap::new(),
            axis_state: AxisState::default(),
            last_frame_time: Instant::now(),
            file_dialog: FileDialog::new(),
            picked_file: None,
        }
    }

    pub fn create_window_and_gl_context(&mut self) {
        self.sdl2_window = Some(
            self.sdl2_context
                .video()
                .unwrap()
                .window(
                    "ruffle4consoles",
                    self.dimensions.width,
                    self.dimensions.height,
                )
                .opengl()
                .resizable()
                .position_centered()
                .build()
                .unwrap(),
        );
        let _ = self
            .sdl2_window
            .as_ref()
            .unwrap()
            .gl_create_context()
            .unwrap();
        self.glow_context = unsafe {Some(Arc::new(egui_glow::glow::Context::from_loader_function(|s|self.sdl2_context.video().unwrap().gl_get_proc_address(s) as *const _)))};
    }

    pub fn setup_egui(&mut self) {
        self.egui_ctx = Some(egui::Context::default());
        self.egui_painter = Some(Painter::new(self.glow_context.as_ref().unwrap().clone(), "", Some(ShaderVersion::Es100), false).unwrap());
        self.egui_sdl2_state = Some(EguiSDL2State::new(self.dimensions.width, self.dimensions.height, 1.0));
    }

    pub fn create_player(&mut self, movie_path: &str) {
        let movie =
            SwfMovie::from_path(movie_path, None).map_err(|e| anyhow::anyhow!(e.to_string()));
        self.player = Some(
            PlayerBuilder::new()
                .with_log(ConsoleLogBackend::new())
                .with_renderer(
                    GlowRenderBackend::new(
                        self.glow_context.as_ref().unwrap().clone(),
                        false,
                        StageQuality::High,
                    )
                    .unwrap(),
                )
                .with_audio(SdlAudioBackend::new(self.sdl2_context.audio().unwrap()).unwrap())
                .with_movie(movie.unwrap())
                .with_viewport_dimensions(
                    self.dimensions.width,
                    self.dimensions.height,
                    self.dimensions.scale_factor,
                )
                .with_fullscreen(true)
                .with_letterbox(Letterbox::On)
                .with_gamepad_button_mapping(self.gamepad_button_mapping.clone())
                .with_autoplay(true)
                .build(),
        );
        self.player
            .as_mut()
            .unwrap()
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::none());
    }

    pub fn loop_player(&mut self) {
        let sdl2_game_controller = self.sdl2_context.game_controller().unwrap();
        let mut event_pump = self.sdl2_context.event_pump().unwrap();
        'main: loop {
            #[cfg(target_os = "horizon")]
            {
                let (nx_width, nx_height) = sdl2_window.drawable_size();
                if nx_width != dimensions.width && nx_height != dimensions.height {
                    dimensions.width = nx_width;
                    dimensions.height = nx_height;
                    self.player
                        .as_ref()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .set_viewport_dimensions(dimensions);
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
                            self.dimensions.width = w as u32;
                            self.dimensions.height = h as u32;
                            self.player
                                .as_ref()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .set_viewport_dimensions(self.dimensions);
                        }
                    }
                    sdl2::event::Event::ControllerDeviceAdded {
                        timestamp: _,
                        which,
                    } => {
                        self.controllers
                            .push(sdl2_game_controller.open(which).unwrap());
                    }
                    sdl2::event::Event::ControllerDeviceRemoved {
                        timestamp: _,
                        which,
                    } => {
                        if let Some(pos) = self
                            .controllers
                            .iter()
                            .position(|c| c.instance_id() == which)
                        {
                            self.controllers.remove(pos); // drops the controller -> SDL closes it
                        }
                    }
                    sdl2::event::Event::ControllerButtonDown {
                        timestamp: _,
                        which: _,
                        button,
                    } => {
                        let ruffle_button = sdl_gamepadbutton_to_ruffle(button);
                        if let Some(ruffle_button) = ruffle_button {
                            self.player.as_ref().unwrap().lock().unwrap().handle_event(
                                PlayerEvent::GamepadButtonDown {
                                    button: ruffle_button,
                                },
                            );
                        }
                    }
                    sdl2::event::Event::ControllerButtonUp {
                        timestamp: _,
                        which: _,
                        button,
                    } => {
                        let ruffle_button = sdl_gamepadbutton_to_ruffle(button);
                        if let Some(ruffle_button) = ruffle_button {
                            self.player.as_ref().unwrap().lock().unwrap().handle_event(
                                PlayerEvent::GamepadButtonUp {
                                    button: ruffle_button,
                                },
                            );
                        }
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
                            self.axis_state.left
                        };
                        let right = if x_axis {
                            value > deadzone
                        } else {
                            self.axis_state.right
                        };
                        let up = if y_axis {
                            value < -deadzone
                        } else {
                            self.axis_state.up
                        };
                        let down = if y_axis {
                            value > deadzone
                        } else {
                            self.axis_state.down
                        };

                        if up != self.axis_state.up {
                            let event_up = if up {
                                PlayerEvent::GamepadButtonDown {
                                    button: GamepadButton::DPadUp,
                                }
                            } else {
                                PlayerEvent::GamepadButtonUp {
                                    button: GamepadButton::DPadUp,
                                }
                            };
                            self.axis_state.up = up;
                            self.player
                                .as_ref()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .handle_event(event_up);
                        }
                        if down != self.axis_state.down {
                            let event_down = if down {
                                PlayerEvent::GamepadButtonDown {
                                    button: GamepadButton::DPadDown,
                                }
                            } else {
                                PlayerEvent::GamepadButtonUp {
                                    button: GamepadButton::DPadDown,
                                }
                            };
                            self.axis_state.down = down;
                            self.player
                                .as_ref()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .handle_event(event_down);
                        }
                        if left != self.axis_state.left {
                            let event_left = if left {
                                PlayerEvent::GamepadButtonDown {
                                    button: GamepadButton::DPadLeft,
                                }
                            } else {
                                PlayerEvent::GamepadButtonUp {
                                    button: GamepadButton::DPadLeft,
                                }
                            };
                            self.axis_state.left = left;
                            self.player
                                .as_ref()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .handle_event(event_left);
                        }
                        if right != self.axis_state.right {
                            let event_right = if right {
                                PlayerEvent::GamepadButtonDown {
                                    button: GamepadButton::DPadRight,
                                }
                            } else {
                                PlayerEvent::GamepadButtonUp {
                                    button: GamepadButton::DPadRight,
                                }
                            };
                            println!("{} {}", right, value);
                            self.axis_state.right = right;
                            self.player
                                .as_ref()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .handle_event(event_right);
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
                            self.player.as_ref().unwrap().lock().unwrap().handle_event(
                                PlayerEvent::MouseDown {
                                    x: x.into(),
                                    y: y.into(),
                                    button: ruffle_button,
                                    index: None,
                                },
                            );
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
                            self.player.as_ref().unwrap().lock().unwrap().handle_event(
                                PlayerEvent::MouseUp {
                                    x: x.into(),
                                    y: y.into(),
                                    button: ruffle_button,
                                },
                            );
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
                        self.player.as_ref().unwrap().lock().unwrap().handle_event(
                            PlayerEvent::MouseDown {
                                x: x as f64 * self.dimensions.width as f64,
                                y: y as f64 * self.dimensions.height as f64,
                                button: MouseButton::Left,
                                index: None,
                            },
                        );
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
                        self.player.as_ref().unwrap().lock().unwrap().handle_event(
                            PlayerEvent::MouseUp {
                                x: x as f64 * self.dimensions.width as f64,
                                y: y as f64 * self.dimensions.height as f64,
                                button: MouseButton::Left,
                            },
                        );
                    }
                    _ => {}
                }
            }
            let new_time = Instant::now();
            let dt = new_time.duration_since(self.last_frame_time).as_micros();
            if dt > 0 {
                self.last_frame_time = new_time;
                self.player
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .tick(dt as f64 / 1000.0);
                if self.player.as_ref().unwrap().lock().unwrap().needs_render() {
                    self.player.as_ref().unwrap().lock().unwrap().render();
                    self.sdl2_window.as_ref().unwrap().gl_swap_window();
                }
            }
        }
    }

    pub fn loop_egui(&mut self) -> Option<PathBuf> {
        'running: loop {
        
        // --- Handle events
        for event in self.sdl2_context.event_pump().unwrap().poll_iter() {
            match &event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Escape), .. } => break 'running,

                sdl2::event::Event::Window { window_id, win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::SizeChanged(w, h) | sdl2::event::WindowEvent::Resized(w, h) => {
                        if *window_id == self.sdl2_window.as_ref().unwrap().id() {
                            unsafe { self.glow_context.as_ref().unwrap().viewport(0, 0, *w, *h) };
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            self.egui_sdl2_state.as_mut().unwrap().sdl2_input_to_egui(self.sdl2_window.as_ref().unwrap(), &event);
        }   

        // --- Run egui
        let full_output = self.egui_ctx.as_ref().unwrap().run(self.egui_sdl2_state.as_mut().unwrap().raw_input.take(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.file_dialog.pick_file();
            });
        });
        self.egui_sdl2_state.as_mut().unwrap().process_output(self.sdl2_window.as_ref().unwrap(), &full_output.platform_output);
        let clipped_primitives = self.egui_ctx.as_ref().unwrap().tessellate(full_output.shapes, 1.0);

        // --- Paint
                let(w,h) = self.sdl2_window.as_ref().unwrap().drawable_size();
        unsafe {
            self.glow_context.as_ref().unwrap().viewport(0, 0, w as i32, h as i32);
            self.glow_context.as_ref().unwrap().clear_color(0.1, 0.1, 0.1, 1.0);
            self.glow_context.as_ref().unwrap().clear(egui_glow::glow::COLOR_BUFFER_BIT);
        }
        
        self.egui_painter.as_mut().unwrap().paint_and_update_textures(
            [w, h],
            self.egui_sdl2_state.as_ref().unwrap().dpi_scaling,
            &clipped_primitives,
            &full_output.textures_delta
        );

        self.sdl2_window.as_ref().unwrap().gl_swap_window();
    }
    self.picked_file.clone()
    }
}
