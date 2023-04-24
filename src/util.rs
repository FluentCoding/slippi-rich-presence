use std::{time::{SystemTime, UNIX_EPOCH, Duration}, thread};

pub fn current_unix_time() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().try_into().unwrap()
}

pub fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

pub fn round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}
