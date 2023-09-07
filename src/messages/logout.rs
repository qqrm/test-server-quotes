use serde::{Deserialize, Serialize};

/// Represents a request to log out from the server.
///
/// Allows a user to signal their intent to end their session.
///
/// # Fields
///
/// * `login` - The login identifier of the user intending to log out.
/// * `hash` - The session hash provided by the server during a successful login.
#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutReqMessage {
    pub login: String,
    pub hash: String,
}

/// Represents the server's acknowledgment to a logout request.
#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutSuccMessage {}
