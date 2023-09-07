use serde::{Deserialize, Serialize};

/// Represents a request message for a "Word of Wisdom" quote.
///
/// After the successful login and PoW verification, users can use this structure
/// to request a quote from the server. Users must provide their `login` details
/// and the proof of work computed as `pow`.
///
/// # Fields
///
/// * `login` - The unique identifier for the user requesting the quote.
/// * `pow` - Proof of work value computed by the user. This serves as a mitigation
///           against potential DDoS attacks by ensuring that a user performs some
///           computational work before making a request.
#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteReqMessage {
    pub login: String,
    pub pow: u64,
}

/// Represents the server's response containing the requested quote.
///
/// After validating the user's proof of work and ensuring the user has
/// access rights, the server responds with a quote from the "Word of Wisdom"
/// or another wisdom collection. The response also contains a `hash` to
/// track session state and a `difficulty` value which informs the user about
/// the level of complexity for the next proof of work.
///
/// # Fields
///
/// * `quote` - The provided quote from the "Word of Wisdom" or another collection.
/// * `hash` - A session hash that may be required for future interactions or
///            for maintaining session state.
/// * `difficulty` - The difficulty level for the next proof of work challenge
///                  that the user might need to compute. This might change based
///                  on server conditions or other metrics.
#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteRespMessage {
    pub quote: String,
    pub hash: String,
    pub difficulty: u64,
}
