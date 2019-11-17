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
    static ref START_TIME: time_t = {
        if let Ok(s) = std::env::var("START_TIME") {
            match s.parse::<time_t>() {
                Ok(t) => return t,
                Err(e) => eprintln!("Failed to parse START_TIME: {}", e),
            }
        }
        unsafe { redhook::real!(time)(ptr::null_mut()) }
    };
    static ref WARP_AMOUNT: f64 = {
        if let Ok(s) = std::env::var("TIME_WARP") {
            match s.parse::<f64>() {
                Ok(x) if x.is_negative() => eprintln!("Cannot do negative TIME_WARP"),
                Err(e) => eprintln!("Failed to parse TIME_WARP: {}", e),
                Ok(x) => return x,
            }
        }
        1.
    };
}

fn warp_time(real: time_t) -> time_t {
    let start_time = *START_TIME;
    start_time + ((real - start_time) as f64 * *WARP_AMOUNT) as time_t
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
