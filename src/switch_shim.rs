#[cfg(target_os = "horizon")]
use sdl2::libc;

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