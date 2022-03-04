use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct LogoutAutMessage {
    login: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogoutReqMessage {
    login: String,
    hash: String,
    num: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogoutSuccMessage {}
