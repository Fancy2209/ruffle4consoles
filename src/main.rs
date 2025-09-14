mod log;

use anyhow::anyhow;
use log::VitaLogBackend;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{PlayerBuilder, ViewportDimensions};
use ruffle_render::quality::StageQuality;
use ruffle_render_glow::GlowRenderBackend;

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

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let sdl2_video = sdl2_context.video().unwrap();
    let gl_attr = sdl2_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_version(2, 0);

    let sdl2_window = sdl2_video
        .window("ruffle4vita", 940, 544)
        .opengl()
        //.position_centered()
        .build()
        .unwrap();

    let gl_context = sdl2_window.gl_create_context().unwrap();
    let _ = sdl2_window.gl_make_current(&gl_context);

    let bytes = include_bytes!("movie.swf");

    let movie = SwfMovie::from_data(bytes, "./movie.swf".to_string(), None)
        .map_err(|e| anyhow!(e.to_string()));
    let log = VitaLogBackend::default();

    let context: glow::Context;
    unsafe {
        context =
            glow::Context::from_loader_function(|s| sdl2_video.gl_get_proc_address(s) as *const _);
    }
    let renderer = GlowRenderBackend::new(
        context,
        ViewportDimensions {
            width: 940,
            height: 544,
            scale_factor: 1.0,
        },
        false,
        StageQuality::High,
    )
    .unwrap();

    let player = PlayerBuilder::new()
        .with_log(log.clone())
        .with_renderer(renderer)
        .with_movie(movie.unwrap())
        .with_viewport_dimensions(940, 544, 1.0)
        .build();
    player.lock().unwrap().preload(&mut ExecutionLimit::none());

    let mut event_pump = sdl2_context.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }
        player.lock().unwrap().run_frame();
        player.lock().unwrap().render();

        sdl2_window.gl_swap_window();
    }
}
