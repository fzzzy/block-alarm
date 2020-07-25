use std::{thread, time};

use futures::future::{ Future, FutureExt };
use tokio::time::{delay_for, Duration};

#[tokio::main]
async fn main() {
    std::thread::spawn(|| {
        match block_alarm::background_thread() {
            Ok(_) => {},
            Err(e) => { 
                println!("background thread had an error {:?}", e);
            }
        };
    });
    println!("Setting up the alarm");
    // one hundred milliseconds in usec
    let a = block_alarm::Alarm::new(1e5 as i64);
    a.start();
    println!("Alarm started.");
    println!("Blocking");
    let one_second = Duration::from_millis(1000);
    thread::sleep(one_second);
    println!("Blocked.");
    println!("Cooperatively delaying");
    delay_for(one_second).await;
    println!("Delayed.");
    println!("Blocking again");
    thread::sleep(one_second);
    println!("Done blocking.");
}
