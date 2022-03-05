use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginReqMessage {
    pub login: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginSuccMessage {
    pub hash: String,
    pub difficulty: u64,
}
