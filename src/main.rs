pub mod messages;
mod state;
mod utils;

use crate::{
    messages::{
        login::LoginSuccMessage, logout::LogoutSuccMessage, quote::QuoteRespMessage,
        time::RespTimeMessage,
    },
    state::UserState,
};
use actix_web::{
    error::{self},
    http::{header::ContentType, StatusCode},
    middleware, web, App, HttpRequest, HttpResponse, HttpServer,
};
use derive_more::{Display, Error};
use futures::lock::Mutex;
use messages::{
    login::LoginReqMessage, logout::LogoutReqMessage, quote::QuoteReqMessage, time::ReqTimeMessage,
};
use rand::prelude::SliceRandom;
use state::State;
use utils::get_unix_time_in_secs;

#[derive(Debug, Display, Error)]
enum GetTimeError {
    #[display(fmt = "user not exist")]
    UserNotExist,

    #[display(fmt = "not allowed")]
    NotAllowed,

    #[display(fmt = "json convertion failed")]
    JsonConvertionFailed,

    #[display(fmt = "internal state unavalible")]
    InternalStateUnavailible,
}

impl error::ResponseError for GetTimeError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            GetTimeError::UserNotExist => StatusCode::UNAUTHORIZED,
            GetTimeError::NotAllowed => StatusCode::UNAUTHORIZED,
            GetTimeError::JsonConvertionFailed => StatusCode::INTERNAL_SERVER_ERROR,
            GetTimeError::InternalStateUnavailible => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

async fn get_time(item: web::Json<ReqTimeMessage>, req: HttpRequest) -> HttpResponse {
    let req_time = item.0;

    if let Some(data) = req.app_data::<web::Data<Mutex<State>>>() {
        let mut state = data.as_ref().lock().await;

        if state.users.contains_key(&req_time.login) {
            let time = get_unix_time_in_secs();

            if let Some(user_state) = state.users.get(&req_time.login) {
                let hash = md5::compute(time.to_string() + user_state);
                state.authorized.insert(
                    req_time.login,
                    (format!("{:x}", hash), UserState::InProcess),
                );
            } else {
                return HttpResponse::from_error(GetTimeError::UserNotExist);
            }

            // GetTimeError::JsonConvertionFailed
            if let Ok(resp) = serde_json::to_string(&RespTimeMessage { time }) {
                return HttpResponse::Ok().json(resp);
            } else {
                return HttpResponse::from_error(GetTimeError::JsonConvertionFailed);
            }
        }

        return HttpResponse::from_error(GetTimeError::NotAllowed);
    }

    return HttpResponse::from_error(GetTimeError::InternalStateUnavailible);
}

async fn login(item: web::Json<LoginReqMessage>, req: HttpRequest) -> HttpResponse {
    let login_req = item.0;
    // dbg!(&login_req);

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

async fn get_quote(item: web::Json<QuoteReqMessage>, req: HttpRequest) -> HttpResponse {
    let quote_req = item.0;

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

    HttpResponse::Ok().json(resp)
}

async fn logout(item: web::Json<LogoutReqMessage>, req: HttpRequest) -> HttpResponse {
    let logout_req = item.0;

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

            let hash = md5::compute(data);
            let hash = format!("{:x}", hash);

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

    HttpResponse::Ok().json(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let data = web::Data::new(Mutex::new(state::State::new()));

    HttpServer::new(move || {
        App::new()
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
