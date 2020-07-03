
use futures::future::{ Future, FutureExt };
use std::error::Error;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::{ thread, time };

use signal_hook::iterator::Signals;

use tokio::time::{ delay_for, Duration };

const ITIMER_VIRTUAL: c_int = 1;
const SIGVTALRM: c_int = 26;

const BLOCK: bool = true;

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
    timeout: i64
}

impl Alarm {
    pub fn new(timeout: i64) -> Self {
        let mut me = Alarm { timeout };
        me.retrigger();
        me
    }

    pub fn start(self) {
        tokio::spawn(async move {
            safety(self);
        });
    }

    pub fn retrigger(&mut self) {
        let it_interval = Timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let it_value = Timeval {
            tv_sec: self.timeout / 1e6 as i64,
            tv_usec: self.timeout % 1e6 as i64,
        };

        unsafe {
            setitimer(
                ITIMER_VIRTUAL,
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
                ITIMER_VIRTUAL,
                &mut Itimerval {
                    it_interval,
                    it_value,
                },
                null_mut(),
            )
        };
    }
}

fn background_thread() -> Result<(), Box<dyn Error>> {
    let s = Signals::new(&[
            SIGVTALRM,
            signal_hook::SIGTERM |
            signal_hook::SIGINT |
            signal_hook::SIGQUIT
        ])?;
    'outer: loop {
        // Pick up signals that arrived since last time
        for signal in s.pending() {
            match signal as c_int {
                signal_hook::SIGTERM | signal_hook::SIGINT | signal_hook::SIGQUIT => {
                    break 'outer;
                },
                _ => {
                    println!("something blocked");
                }
            }
        }
    }
    Ok(())
}

fn safety(mut me: Alarm) {
    me.retrigger();
    tokio::spawn_front(async move {
        safety(me);
    });
}

#[tokio::main]
async fn main() {
    std::thread::spawn(|| {
        match background_thread() {
            Ok(_) => {},
            Err(e) => { 
                println!("background thread had an error {:?}", e);
            }
        };
    });
    println!("Setting up the alarm");
    // one hundred milliseconds in usec
    let a = Alarm::new(1e5 as i64);
    a.start();
    println!("Alarm started.");
    println!("Blocking");
    let one_second = time::Duration::from_millis(1000);
    if BLOCK {
        thread::sleep(one_second);
    }
    println!("Blocked.");
    println!("Cooperatively delaying");
    let two_seconds = Duration::new(2, 0);
    delay_for(two_seconds).await;
    println!("Delayed.");
    println!("Blocking again");
    thread::sleep(one_second);
    println!("Done blocking.");
}
