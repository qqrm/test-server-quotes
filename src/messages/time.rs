use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ReqTimeMessage {
    pub login: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RespTimeMessage {
    pub time: u64,
}
