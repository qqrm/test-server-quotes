pub mod messages;
mod state;
mod utils;

use std::borrow::BorrowMut;

use actix_web::{
    dev::AppConfig, error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use futures::{lock::Mutex, StreamExt};
use json::JsonValue;
use messages::{
    login::LoginReqMessage, logout::LogoutReqMessage, quote::QuoteReqMessage, time::ReqTimeMessage,
};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use state::State;
use utils::get_unix_time_in_secs;

use crate::{
    messages::{
        login::LoginSuccMessage, logout::LogoutSuccMessage, quote::QuoteRespMessage,
        time::RespTimeMessage,
    },
    state::UserState,
};

async fn get_time(item: web::Json<ReqTimeMessage>, req: HttpRequest) -> HttpResponse {
    let req_time = item.0;
    dbg!(&req_time);

    let data = req.app_data::<web::Data<Mutex<State>>>().unwrap();
    let mut state = data.as_ref().lock().await;

    let resp = if state.users.contains_key(&req_time.login) {
        let time = get_unix_time_in_secs();
        let hash = md5::compute(
            time.to_string() + state.users.get(&req_time.login).expect("user not exist"),
        );
        state.authorized.insert(
            req_time.login,
            (format!("{:x}", hash), UserState::InProcess),
        );

        serde_json::to_string(&RespTimeMessage { time }).expect("json login cucces conv failed")
    } else {
        "{\"not allowed\"}".to_string()
    };
    HttpResponse::Ok().json(resp)
}

async fn login(item: web::Json<LoginReqMessage>, req: HttpRequest) -> HttpResponse {
    let login_req = item.0;
    dbg!(&login_req);

    let data = req.app_data::<web::Data<Mutex<State>>>().unwrap();
    let mut state = data.as_ref().lock().await;

    let resp = if state.authorized.contains_key(&login_req.login) {
        let user_info = state.authorized.get(&login_req.login);

        match user_info {
            Some((last_hash, user_state)) => {
                if UserState::InProcess == *user_state && login_req.hash == *last_hash {
                    let new_hash = md5::compute(last_hash.clone());

                    let login_succ_mess = LoginSuccMessage {
                        hash: format!("{:x}", new_hash),
                        difficulty: state.difficulty,
                    };

                    state.authorized.insert(
                        login_req.login,
                        (login_succ_mess.hash.clone(), UserState::Auth),
                    );
                    serde_json::to_string(&login_succ_mess).expect("json login cucces conv failed")
                } else {
                    "{\"smth wrong\"}".to_string()
                }
            }
            None => "{\"auth not started\"}".to_string(),
        }
    } else {
        "{\"auth not started\"}".to_string()
    };

    HttpResponse::Ok().json(resp)
}

async fn get_quote(
    item: web::Json<QuoteReqMessage>,
    req: HttpRequest,
    state: web::Data<Mutex<State>>,
) -> HttpResponse {
    let quote_req = item.0;
    dbg!(&quote_req);

    let data = req.app_data::<web::Data<Mutex<State>>>().unwrap();
    let mut state = data.as_ref().lock().await;

    let user_info = state.authorized.get(&quote_req.login);

    let resp = match user_info {
        Some((last_hash, user_state)) => {
            let data = last_hash.clone() + &quote_req.pow.to_string();
            let pow_hash = md5::compute(data);
            let pow_hash = format!("{:x}", pow_hash);

            if UserState::Auth == *user_state
                && pow_hash[..state.difficulty as usize]
                    == "0".to_string().repeat(state.difficulty as usize)
            {
                let hash = md5::compute(last_hash);

                let quote_resp_mess = QuoteRespMessage {
                    quote: state
                        .quotes
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone(),
                    hash: format!("{:x}", hash),
                    difficulty: state.difficulty,
                };

                state.authorized.insert(
                    quote_req.login,
                    (quote_resp_mess.hash.clone(), UserState::Auth),
                );
                serde_json::to_string(&quote_resp_mess).expect("json quote resp conv failed")
            } else {
                "{\"pow failed\"}".to_string()
            }
        }
        None => "{\"user not auth\"}".to_string(),
    };

    HttpResponse::Ok().json(resp.to_string())
}

async fn logout(item: web::Json<LogoutReqMessage>, req: HttpRequest) -> HttpResponse {
    let logout_req = item.0;
    dbg!(&logout_req);

    let data = req.app_data::<web::Data<Mutex<State>>>().unwrap();
    let mut state = data.as_ref().lock().await;

    let user_info = state.authorized.get(&logout_req.login);

    let resp = match user_info {
        Some((last_hash, user_state)) => {
            let data = last_hash.clone()
                + state
                    .users
                    .get(&logout_req.login)
                    .expect("user not register");

            dbg!(&data);

            let hash = md5::compute(data);
            let hash = format!("{:x}", hash);
            dbg!(&hash);

            if UserState::Auth == *user_state && logout_req.hash == hash {
                let logout_resp_mess = LogoutSuccMessage {};

                state.authorized.remove(&logout_req.login);
                serde_json::to_string(&logout_resp_mess).expect("json quote resp conv failed")
            } else {
                "{\"access denied\"}".to_string()
            }
        }
        None => "{\"user not auth\"}".to_string(),
    };

    HttpResponse::Ok().json(resp.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let data = web::Data::new(Mutex::new(state::State::new()));

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(data.clone())
            .service(web::resource("/time").route(web::post().to(get_time)))
            .service(web::resource("/auth").route(web::post().to(login)))
            .service(web::resource("/quote").route(web::post().to(get_quote)))
            .service(web::resource("/logout").route(web::post().to(logout)))
    })
    .bind(("127.0.0.1", 9999))?
    .run()
    .await
}
