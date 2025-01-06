use std::time::{SystemTime, UNIX_EPOCH};

pub fn get() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
