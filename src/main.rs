
use std::error::Error;
use std::os::raw::c_int;
use std::ptr::null_mut;

const ITIMER_PROF: c_int = 2;

#[repr(C)]
#[derive(Clone)]
struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

#[repr(C)]
#[derive(Clone)]
struct Itimerval {
    pub it_interval: Timeval,
    pub it_value: Timeval,
}

extern "C" {
    fn setitimer(which: c_int, new_value: *mut Itimerval, old_value: *mut Itimerval) -> c_int;
}

pub struct Alarm {
    timeout: c_int
}

impl Alarm {
    pub fn new(timeout: c_int) -> Self {
        let mut me = Alarm { timeout };
        me.retrigger();
        me
    }

    pub fn retrigger(&mut self) {
        let it_interval = Timeval {
            tv_sec: self.timeout as i64 / 1e6 as i64,
            tv_usec: self.timeout as i64 % 1e6 as i64,
        };
        let it_value = it_interval.clone();

        unsafe {
            setitimer(
                ITIMER_PROF,
                &mut Itimerval {
                    it_interval,
                    it_value,
                },
                null_mut(),
            )
        };
    }
}

impl Drop for Alarm {
    fn drop(&mut self) {
        let it_interval = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let it_value = it_interval.clone();
        unsafe {
            setitimer(
                ITIMER_PROF,
                &mut Itimerval {
                    it_interval,
                    it_value,
                },
                null_mut(),
            )
        };
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
