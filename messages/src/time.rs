use serde::{Deserialize, Serialize};

/// Represents a request to retrieve server time.
///
/// Users can request the current server time using this structure. This might be useful
/// for synchronization purposes, calculating offsets, or other time-related operations.
///
/// # Fields
///
/// * `login` - The unique identifier for the user requesting the server time. This can
///             be used by the server to verify the legitimacy of the request.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReqTimeMessage {
    pub login: String,
}

/// Represents the server's response containing the current server time.
///
/// After a user sends a `ReqTimeMessage`, the server responds with the current time
/// represented in seconds (or another time unit if needed).
///
/// # Fields
///
/// * `time` - Current server time in seconds since the Unix epoch. The actual time
///            unit and reference can be adjusted based on server configuration.
#[derive(Serialize, Deserialize, Debug)]
pub struct RespTimeMessage {
    pub time: u64,
}
