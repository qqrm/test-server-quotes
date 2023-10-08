use serde::{Deserialize, Serialize};

/// Represents a request for login with the intention to obtain a quote.
///
/// Before accessing a quote, a user must provide proof of their identity and
/// their intent by submitting a valid `hash` that satisfies the server's
/// Proof of Work challenge.
///
/// # Fields
///
/// * `login` - The login identifier for the user.
/// * `hash` - The computed hash value which serves as the proof of work.
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginReqMessage {
    pub login: String,
    pub hash: String,
}

/// Represents the server's response to a successful login.
///
/// After the user's provided hash is verified, the server responds with a session `hash`
/// and the difficulty level for any subsequent Proof of Work challenge.
///
/// # Fields
///
/// * `hash` - A session hash which may be required for future interactions or to maintain session state.
/// * `difficulty` - The difficulty level for the next proof of work that the user might need to compute.
#[derive(Serialize, Deserialize, Debug)]
pub struct LoginSuccMessage {
    pub hash: String,
    pub difficulty: u64,
}
