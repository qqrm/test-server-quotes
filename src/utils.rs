use std::time::SystemTime;

pub fn get_unix_time_in_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Unixtime getting error")
        .as_secs()
}
