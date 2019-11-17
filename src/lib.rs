#![allow(deprecated)] // redhook still uses ONCE_INIT

use libc::time_t;
use std::ptr;

// Not contained in libc as it's deprecated
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct timeb {
    time: time_t,
    millitm: libc::c_ushort,
    timezone: libc::c_short,
    dstflag: libc::c_short,
}

lazy_static::lazy_static! {
    static ref FIRST_TIME: time_t = unsafe { redhook::real!(time)(ptr::null_mut()) };
    static ref TIME_OFFSET: time_t = {
        if let Ok(s) = std::env::var("TIME_OFFSET") {
            match s.parse::<time_t>() {
                Ok(t) => return t,
                Err(e) => eprintln!("Failed to parse TIME_OFFSET: {}", e),
            }
        }
        if let Ok(s) = std::env::var("TIME_START") {
            match s.parse::<time_t>() {
                Ok(t) => return t - *FIRST_TIME,
                Err(e) => eprintln!("Failed to parse START_TIME: {}", e),
            }
        }
        0
    };
    static ref WARP_AMOUNT: f64 = {
        if let Ok(s) = std::env::var("TIME_WARP") {
            match s.parse::<f64>() {
                Err(e) => eprintln!("Failed to parse TIME_WARP: {}", e),
                Ok(x) => return x,
            }
        }
        1.
    };
}

fn warp_time(real: time_t) -> time_t {
    *FIRST_TIME
        + ((real - *FIRST_TIME) as f64 * *WARP_AMOUNT) as time_t
        + *TIME_OFFSET
}

redhook::hook! {
    unsafe fn time(ptr: *mut time_t) -> time_t => hook_time {
        let real = redhook::real!(time)(ptr::null_mut());
        let modified = warp_time(real);
        if !ptr.is_null() {
            *ptr = modified;
        }
        modified
    }
}

redhook::hook! {
    unsafe fn ftime(ptr: *mut timeb) -> libc::c_int => hook_ftime {
        let ret = redhook::real!(ftime)(ptr);
        // Should always be the case, but in theory could error.
        if ret == 0 {
            (*ptr).time = warp_time((*ptr).time);
        }
        ret
    }
}

redhook::hook! {
    unsafe fn clock_gettime(clk_id: libc::clockid_t, timespec: *mut libc::timespec)
        -> libc::c_int => hook_clock_gettime
    {
        let ret = redhook::real!(clock_gettime)(clk_id, timespec);
        if ret == 0 && clk_id == libc::CLOCK_REALTIME {
            (*timespec).tv_sec = warp_time((*timespec).tv_sec);
        }
        ret
    }
}

redhook::hook! {
    unsafe fn gettimeofday(tv: *mut libc::timeval, tz: *mut libc::timezone)
        -> libc::c_int => hook_gettimeofday
    {
        let ret = redhook::real!(gettimeofday)(tv, tz);
        if ret == 0 && !tv.is_null() {
            (*tv).tv_sec = warp_time((*tv).tv_sec);
        }
        ret
    }
}
