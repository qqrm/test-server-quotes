use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteReqMessage {
    pub login: String,
    pub pow: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteRespMessage {
    pub quote: String,
    pub hash: String,
    pub difficulty: u64,
}
