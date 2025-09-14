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

#[cfg(target_os = "horizon")]
unsafe extern "C" {
    pub fn randomGet(buf: *mut libc::c_void, len: libc::size_t);
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

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let sdl2_video = sdl2_context.video().unwrap();
    let gl_attr = sdl2_video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
    gl_attr.set_context_version(2, 0);

    #[cfg(target_os = "vita")]
    let dimensions = ViewportDimensions {
        width: 940,
        height: 544,
        scale_factor: 1.0,
    };

    #[cfg(target_os = "horizon")]
    let dimensions = ViewportDimensions {
        width: 1280,
        height: 720,
        scale_factor: 1.0,
    };

    #[cfg(not(any(target_os = "horizon", target_os = "vita")))]
    let dimensions = ViewportDimensions {
        width: 1280,
        height: 720,
        scale_factor: 1.0,
    };

    let sdl2_window = sdl2_video
        .window("ruffle4consoles", dimensions.width, dimensions.height)
        .opengl()
        //.position_centered()
        .build()
        .unwrap();

    let gl_context = sdl2_window.gl_create_context().unwrap();
    let _ = sdl2_window.gl_make_current(&gl_context);

    let bytes = include_bytes!("test.swf");

    let movie = SwfMovie::from_data(bytes, "./test.swf".to_string(), None)
        .map_err(|e| anyhow!(e.to_string()));
    let log = VitaLogBackend::default();

    // Glow can only realistically be used in vita and horizon, need
    let context: glow::Context;
    unsafe {
        context =
            glow::Context::from_loader_function(|s| sdl2_video.gl_get_proc_address(s) as *const _);
    }
    let renderer = GlowRenderBackend::new(context, dimensions, false, StageQuality::High).unwrap();

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
