use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteReqMessage {
    pub login: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuoteRespMessage {
    pub quote: String,
    pub hash: String,
    pub num: u64,
}
