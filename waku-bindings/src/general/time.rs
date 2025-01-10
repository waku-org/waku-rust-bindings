use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_now_in_nanosecs() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_secs() * 1_000_000_000 + since_epoch.subsec_nanos() as u64
}
