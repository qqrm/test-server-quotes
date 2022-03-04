use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

type Login = String;
type Pass = String;
type LastHash = String;

#[derive(PartialEq, Debug)]
pub enum UserState {
    UnAuth,
    Auth,
    InProcess,
}

#[derive(Debug)]
pub struct State {
    pub users: Arc<Mutex<HashMap<Login, Pass>>>,
    pub authorized: Arc<Mutex<HashMap<Login, (LastHash, UserState)>>>,
}

impl State {
    pub fn new() -> State {
        // users already registered (better way is using session id or smth)
        let users = Arc::new(Mutex::new(HashMap::from([(
            ("one".to_string()),
            "pass1".to_string(),
        )])));
        let authorized = Arc::new(Mutex::new(HashMap::new()));

        State { users, authorized }
    }
}
