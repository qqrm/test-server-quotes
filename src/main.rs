mod errors;
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
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use errors::ApiError;
use futures::lock::Mutex;
use messages::{
    login::LoginReqMessage, logout::LogoutReqMessage, quote::QuoteReqMessage, time::ReqTimeMessage,
};
use rand::prelude::SliceRandom;
use state::State;
use utils::get_unix_time_in_secs;

async fn get_time(item: web::Json<ReqTimeMessage>, req: HttpRequest) -> HttpResponse {
    let req_time = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    let time = get_unix_time_in_secs();

    let Some(pass) = state.users.get(&req_time.login) else {
        return HttpResponse::from_error(ApiError::UserNotExist);
    };

    let hash = md5::compute(time.to_string() + pass);
    let hash = format!("{hash:x}");
    state
        .authorized
        .insert(req_time.login, (hash, UserState::InProcess));

    let Ok(resp) = serde_json::to_string(&RespTimeMessage { time }) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed)
    };
    HttpResponse::Ok().json(resp)
}

async fn auth(item: web::Json<LoginReqMessage>, req: HttpRequest) -> HttpResponse {
    let login_req = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    let Some((last_hash, user_state)) = state.authorized.get(&login_req.login) else {
       return HttpResponse::from_error(ApiError::AuthNotStarted);
    };

    if UserState::InProcess != *user_state {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    }

    if login_req.hash != *last_hash {
        return HttpResponse::from_error(ApiError::InvalidHash);
    }

    let new_hash = md5::compute(last_hash);

    let login_succ_mess = LoginSuccMessage {
        hash: format!("{new_hash:x}"),
        difficulty: state.difficulty,
    };

    state.authorized.insert(
        login_req.login,
        (login_succ_mess.hash.clone(), UserState::Auth),
    );

    let Ok(resp) = serde_json::to_string(&login_succ_mess) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed);
    };

    HttpResponse::Ok().json(resp)
}

async fn get_quote(item: web::Json<QuoteReqMessage>, req: HttpRequest) -> HttpResponse {
    let quote_req = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    let Some((last_hash, user_state)) = state.authorized.get(&quote_req.login) else {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    };

    let data = last_hash.clone() + &quote_req.pow.to_string();
    let pow_hash = md5::compute(data);
    let pow_hash = format!("{pow_hash:x}");

    if UserState::Auth != *user_state {
        return HttpResponse::from_error(ApiError::UserNotAuth);
    }

    if pow_hash[..state.difficulty as usize] != "0".to_string().repeat(state.difficulty as usize) {
        return HttpResponse::from_error(ApiError::PovCheckFailed);
    }

    let hash = md5::compute(last_hash);

    let quote_resp_mess = QuoteRespMessage {
        quote: state
            .quotes
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone(),
        hash: format!("{hash:x}"),
        difficulty: state.difficulty,
    };

    state.authorized.insert(
        quote_req.login,
        (quote_resp_mess.hash.clone(), UserState::Auth),
    );

    let Ok(resp) = serde_json::to_string(&quote_resp_mess) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed)
    };

    HttpResponse::Ok().json(resp)
}

async fn logout(item: web::Json<LogoutReqMessage>, req: HttpRequest) -> HttpResponse {
    let logout_req = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    let Some((last_hash, user_state)) = state.authorized.get(&logout_req.login) else {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    };

    let Some(pass) = state.users.get(&logout_req.login) else {
        return HttpResponse::from_error(ApiError::UserNotExist);
    };
    let hash = md5::compute(last_hash.to_owned() + pass);
    let hash = format!("{hash:x}");

    if UserState::Auth != *user_state {
        return HttpResponse::from_error(ApiError::UserNotAuth);
    }

    if logout_req.hash != hash {
        return HttpResponse::from_error(ApiError::InvalidHash);
    }

    let logout_resp_mess = LogoutSuccMessage {};
    state.authorized.remove(&logout_req.login);

    let Ok(resp) = serde_json::to_string(&logout_resp_mess) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed);
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
            .service(web::resource("/auth").route(web::post().to(auth)))
            .service(web::resource("/quote").route(web::post().to(get_quote)))
            .service(web::resource("/logout").route(web::post().to(logout)))
    })
    .bind(("127.0.0.1", 9999))?
    .run()
    .await
}
