use std::time::SystemTime;

/// Returns the current Unix timestamp in seconds.
///
/// # Panics
///
/// This function will panic if the system time is before the Unix epoch.
///
/// # Returns
///
/// A `u64` value representing the current Unix timestamp in seconds since the Unix epoch.
pub fn get_unix_time_in_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time is before the Unix epoch.")
        .as_secs()
}
