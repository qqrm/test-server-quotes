use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutReqMessage {
    pub login: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutSuccMessage {}
