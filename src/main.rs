
use std::path::{Path, PathBuf};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::{Player, PlayerBuilder};
use ruffle_render::quality::StageQuality;
use ruffle_render_gles::GlesRenderBackend;
use anyhow::{anyhow, Error};

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
        .position_centered()
        .build()
        .unwrap();

    let bytes = include_bytes!("test.swf");
    let movie = SwfMovie::from_data(bytes, "./movie.swf".to_string(), None).map_err(|e| anyhow!(e.to_string()));

    let player = PlayerBuilder::new()
        .with_renderer(create_renderer(&sdl2_context, &sdl2_window))
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

fn create_renderer(sdl_context: &sdl2::Sdl, sdl_window: &sdl2::video::Window) -> GlesRenderBackend {
    return ruffle_render_gles::GlesRenderBackend::new(sdl_context, sdl_window, false, StageQuality::High).unwrap();
}
