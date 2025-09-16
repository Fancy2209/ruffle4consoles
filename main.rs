use std::sync::Arc;
use egui::TexturesDelta;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::GLProfile,
};
use egui_sdl2_event::EguiSDL2State;
use egui_glow::{glow::{self, HasContext}, ShaderVersion};
use egui_glow::Painter;

const INITIAL_WIDTH: u32 = 960;
const INITIAL_HEIGHT: u32 = 544;

fn main() {
    // SDL2 setup
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::GLES);
        gl_attr.set_context_version(2, 0);
    }

    let window = video
        .window("egui-glow + SDL2", INITIAL_WIDTH, INITIAL_HEIGHT)
        .opengl()
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let gl:Arc<glow::Context> = unsafe {
        Arc::new(glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _))
    };

    // egui + glow painter
    let egui_ctx = egui::Context::default();
    let mut painter = Painter::new(gl.clone(), "", Some(ShaderVersion::Es100), false).unwrap();
    let mut egui_sdl2 = EguiSDL2State::new(INITIAL_WIDTH, INITIAL_HEIGHT, 1.0);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut checkbox1_checked = false;

    'running: loop {
        // --- Handle events
        for event in event_pump.poll_iter() {
            match &event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                Event::Window { window_id, win_event, .. } => match win_event {
                    WindowEvent::SizeChanged(w, h) | WindowEvent::Resized(w, h) => {
                        if *window_id == window.id() {
                            unsafe { gl.viewport(0, 0, *w, *h) };
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            egui_sdl2.sdl2_input_to_egui(&window, &event);
        }   

        // --- Run egui
        let full_output = egui_ctx.run(egui_sdl2.raw_input.take(), |ctx| {
            egui::Window::new("Settings").show(ctx, |ui| {
                ui.label("Hello from egui_glow!");
                if ui.button("Press me").clicked() {
                    println!("Pressed!");
                }
                ui.checkbox(&mut checkbox1_checked, "checkbox1");
            });
        });

        egui_sdl2.process_output(&window, &full_output.platform_output);
        let clipped_primitives = egui_ctx.tessellate(full_output.shapes, egui_sdl2.dpi_scaling);

        // --- Paint
                let(w,h) = window.drawable_size();
        unsafe {
            gl.viewport(0, 0, w as i32, h as i32);
            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        
        painter.paint_and_update_textures(
            [w, h],
            egui_sdl2.dpi_scaling,
            &clipped_primitives,
            &full_output.textures_delta
        );

        window.gl_swap_window();
    }
}
