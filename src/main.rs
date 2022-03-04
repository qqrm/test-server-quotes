pub mod messages;
mod state;
mod utils;

use std::{borrow::BorrowMut, fmt::Write};

use actix_web::{
    dev::AppConfig, error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use futures::{lock::Mutex, StreamExt};
use json::JsonValue;
use messages::{login::LoginReqMessage, time::ReqTimeMessage};
use serde::{Deserialize, Serialize};
use state::State;
use utils::get_unix_time_in_secs;

use crate::{messages::login::LoginSuccMessage, state::UserState};

async fn get_time(
    item: web::Json<ReqTimeMessage>,
    req: HttpRequest,
    state: web::Data<State>,
) -> HttpResponse {
    let req_time = item.0;
    dbg!(&req_time);

    let resp = if state.users.lock().unwrap().contains_key(&req_time.login) {
        let time = get_unix_time_in_secs().to_string();
        let hash = md5::compute(
            time.clone()
                + state
                    .users
                    .lock()
                    .unwrap()
                    .get(&req_time.login)
                    .expect("user not exist"),
        );
        state.authorized.lock().unwrap().insert(
            req_time.login,
            (format!("{:x}", hash), UserState::InProcess),
        );
        format!("{{\"time\" : \"{}\" }}", time)
    } else {
        "{\"not allowed\"}".to_string()
    };
    HttpResponse::Ok().json(resp)
}

async fn login(
    item: web::Json<LoginReqMessage>,
    req: HttpRequest,
    state: web::Data<State>,
) -> HttpResponse {
    let login_req = item.0;
    dbg!(&login_req);

    let resp = if state.users.lock().unwrap().contains_key(&login_req.login) {
        let mut authorized = state.authorized.lock().unwrap();
        let user_info = authorized.get(&login_req.login);

        match user_info {
            Some((last_hash, user_state)) => {
                if UserState::InProcess == *user_state && login_req.hash == *last_hash {
                    let new_hash = md5::compute(last_hash.clone());

                    let login_succ_mess = LoginSuccMessage {
                        hash: format!("{:x}", new_hash),
                    };

                    authorized.insert(
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let state = web::Data::new(state::State::new());

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(state.clone())
            .service(web::resource("/gettime").route(web::post().to(get_time)))
            .service(web::resource("/auth").route(web::post().to(login)))
        // .service(
        // web::resource("/extractor2")
        // .app_data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (resource level)
        // .route(web::post().to(extract_item)),
        // )
        // .service(web::resource("/manual").route(web::post().to(index_manual)))
        // .service(web::resource("/mjsonrust").route(web::post().to(index_mjsonrust)))
        // .service(web::resource("/").route(web::post().to(index)))
    })
    .bind(("127.0.0.1", 9999))?
    .run()
    .await
}
