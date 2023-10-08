mod errors;
mod state;
mod utils;

use crate::state::UserState;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use errors::ApiError;
use futures::lock::Mutex;
use messages::login::LoginSuccMessage;
use messages::logout::LogoutSuccMessage;
use messages::quote::QuoteRespMessage;
use messages::time::RespTimeMessage;
use messages::{
    login::LoginReqMessage, logout::LogoutReqMessage, quote::QuoteReqMessage, time::ReqTimeMessage,
};
use rand::prelude::SliceRandom;
use state::State;
use utils::get_unix_time_in_secs;
/// Retrieves the current Unix timestamp for a user and sets their authentication process.
///
/// This function handles a user's request to get the current Unix timestamp. It performs
/// the following steps:
/// 1. Verifies that the user exists.
/// 2. Computes a hash based on the current time and the user's password.
/// 3. Updates the user's status in the application state to `InProcess`.
/// 4. Returns the current Unix timestamp to the requester.
///
/// # Arguments
///
/// * `item` - The JSON payload of the user's request containing their login details.
/// * `req` - The HTTP request information.
///
/// # Returns
///
/// Returns a `HttpResponse` containing the current Unix timestamp or an error.
async fn get_time(item: web::Json<ReqTimeMessage>, req: HttpRequest) -> HttpResponse {
    let req_time = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    // Get the current Unix timestamp.
    let time = get_unix_time_in_secs();

    let Some(pass) = state.users.get(&req_time.login) else {
        return HttpResponse::from_error(ApiError::UserNotExist);
    };

    // Compute the authentication hash for the user.
    let hash = md5::compute(format!("{}{}", time, pass));
    let hash_str = format!("{:x}", hash);

    // Update user status in the application state.
    state
        .authorized
        .insert(req_time.login, (hash_str, UserState::InProcess));

    let Ok(resp) = serde_json::to_string(&RespTimeMessage { time }) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed);
    };

    HttpResponse::Ok().json(resp)
}

/// Authenticates a user based on the provided login details.
///
/// This function handles a user's authentication request by:
/// 1. Verifying that they have started the authentication process.
/// 2. Checking if the provided hash matches the expected value.
/// 3. Updating the user's state to `Auth` upon successful verification.
/// 4. Returning a new authentication hash for subsequent requests.
///
/// # Arguments
///
/// * `item` - The JSON payload of the user's request containing their login details.
/// * `req` - The HTTP request information.
///
/// # Returns
///
/// Returns a `HttpResponse` containing the new authentication hash or an error.
async fn auth(item: web::Json<LoginReqMessage>, req: HttpRequest) -> HttpResponse {
    let login_req = item.0;

    // Access the shared application state.
    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    // Check if user's authentication process has started.
    let Some((last_hash, user_state)) = state.authorized.get(&login_req.login) else {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    };

    // Ensure that the user is in the process of authenticating.
    if *user_state != UserState::InProcess {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    }

    // Verify the provided hash.
    if &login_req.hash != last_hash {
        return HttpResponse::from_error(ApiError::InvalidHash);
    }

    // Compute a new authentication hash for the user.
    let new_hash = md5::compute(last_hash);
    let new_hash_str = format!("{:x}", new_hash);

    // Update user's state in the application.
    state
        .authorized
        .insert(login_req.login, (new_hash_str.clone(), UserState::Auth));

    // Return the new authentication hash to the user.
    let Ok(resp) = serde_json::to_string(&LoginSuccMessage {
        hash: new_hash_str,
        difficulty: state.difficulty,
    }) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed);
    };

    HttpResponse::Ok().json(resp)
}

/// Handles the logic for getting a quote.
///
/// # Parameters
/// - `item`: Contains the `QuoteReqMessage` sent from the client.
/// - `req`: The HTTP request containing relevant data.
///
/// # Returns
/// - `HttpResponse`: Returns the response with a `QuoteRespMessage` or an error.
async fn get_quote(item: web::Json<QuoteReqMessage>, req: HttpRequest) -> HttpResponse {
    let quote_req = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    let Some((last_hash, user_state)) = state.authorized.get(&quote_req.login) else {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    };

    // Compute the proof of work hash
    let pow_data = format!("{}{}", last_hash, quote_req.pow);
    let pow_hash = format!("{:x}", md5::compute(&pow_data));

    // Ensure user is authenticated
    if UserState::Auth != *user_state {
        return HttpResponse::from_error(ApiError::UserNotAuth);
    }

    // Check if the computed hash passes the proof of work
    if !pow_hash.starts_with(&"0".repeat(state.difficulty as usize)) {
        return HttpResponse::from_error(ApiError::PovCheckFailed);
    }

    // Compute new hash for the quote response
    let new_hash = format!("{:x}", md5::compute(last_hash));

    // Create quote response message
    let quote_resp_mess = QuoteRespMessage {
        quote: state
            .quotes
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone(),
        hash: new_hash.clone(),
        difficulty: state.difficulty,
    };

    state
        .authorized
        .insert(quote_req.login, (new_hash, UserState::Auth));

    let Ok(resp) = serde_json::to_string(&quote_resp_mess) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed);
    };

    HttpResponse::Ok().json(resp)
}

/// Handles the logic for logging out a user.
///
/// # Parameters
/// - `item`: Contains the `LogoutReqMessage` sent from the client.
/// - `req`: The HTTP request containing relevant data.
///
/// # Returns
/// - `HttpResponse`: Returns the response with a `LogoutSuccMessage` or an error.
async fn logout(item: web::Json<LogoutReqMessage>, req: HttpRequest) -> HttpResponse {
    let logout_req = item.0;

    let Some(data) = req.app_data::<web::Data<Mutex<State>>>() else {
        return HttpResponse::from_error(ApiError::InternalStateUnavailable);
    };
    let mut state = data.as_ref().lock().await;

    // Verify that the user is already authorized.
    let Some((last_hash, user_state)) = state.authorized.get(&logout_req.login) else {
        return HttpResponse::from_error(ApiError::AuthNotStarted);
    };

    // Get the user's password from the state.
    let Some(pass) = state.users.get(&logout_req.login) else {
        return HttpResponse::from_error(ApiError::UserNotExist);
    };

    // Compute the expected hash.
    let expected_hash = format!("{:x}", md5::compute(format!("{}{}", last_hash, pass)));

    // Verify that the user is in the authorized state.
    if UserState::Auth != *user_state {
        return HttpResponse::from_error(ApiError::UserNotAuth);
    }

    // Verify that the provided hash matches the expected hash.
    if logout_req.hash != expected_hash {
        return HttpResponse::from_error(ApiError::InvalidHash);
    }

    // Create the logout success message and remove the user from the authorized list.
    let logout_resp_mess = LogoutSuccMessage {};
    state.authorized.remove(&logout_req.login);

    // Convert the message to JSON.
    let Ok(resp) = serde_json::to_string(&logout_resp_mess) else {
        return HttpResponse::from_error(ApiError::JsonConvertionFailed);
    };

    HttpResponse::Ok().json(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set the logging level and initialize the logger.
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // Create a new shared state instance.
    let data = web::Data::new(Mutex::new(state::State::new()));

    // Start the HTTP server.
    HttpServer::new(move || {
        App::new()
            // Add middleware for logging each request.
            .wrap(middleware::Logger::default())
            // Set the JSON payload limit.
            .app_data(web::JsonConfig::default().limit(4096))
            // Share the app state among the workers.
            .app_data(data.clone())
            // Define the endpoints and associate them with handler functions.
            .service(web::resource("/time").route(web::post().to(get_time)))
            .service(web::resource("/auth").route(web::post().to(auth)))
            .service(web::resource("/quote").route(web::post().to(get_quote)))
            .service(web::resource("/logout").route(web::post().to(logout)))
    })
    // Bind the server to a specific address and port.
    .bind(("127.0.0.1", 9999))?
    // Run the server.
    .run()
    .await
}
