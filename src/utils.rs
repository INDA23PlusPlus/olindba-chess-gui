use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_time() -> u64 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return since_epoch.as_secs() * 1000 + since_epoch.subsec_millis() as u64;
}