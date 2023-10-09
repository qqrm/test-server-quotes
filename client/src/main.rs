//! A client module to interact with a server for authentication, quote retrieval, and testing.
//!
//! This module encapsulates the core client logic to communicate with a server,
//! including sending HTTP requests and processing server responses.
//! It demonstrates a simple client-server interaction model using RESTful APIs.

use futures::stream::{self, StreamExt};
use messages::{
    LoginReqMessage, LoginSuccMessage, LogoutReqMessage, QuoteReqMessage, QuoteRespMessage,
    ReqTimeMessage, RespTimeMessage,
};
use thiserror::Error;
use tokio::runtime::Runtime;

/// An enumeration of client-side errors.
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(reqwest::Error),
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        ClientError::HttpRequest(error)
    }
}

// Define constants for the URLs.
const URL_TIME: &str = "http://server-container/time";
const URL_AUTH: &str = "http://server-container/auth";
const URL_LOGOUT: &str = "http://server-container/logout";
const URL_QUOTE: &str = "http://server-container/quote";

/// Asynchronously fetches the server time.
async fn get_time(client: &reqwest::Client, username: &str) -> Result<u64, ClientError> {
    let resp = client
        .post(URL_TIME)
        .json(&ReqTimeMessage {
            login: username.to_string(),
        })
        .send()
        .await?;

    if resp.status().is_success() {
        let time_resp = resp.json::<RespTimeMessage>().await?;
        Ok(time_resp.time)
    } else {
        Err(ClientError::UnexpectedResponse(format!(
            "Unexpected status code: {}",
            resp.status()
        )))
    }
}

/// Asynchronously performs login operation.
async fn login(
    client: &reqwest::Client,
    time: u64,
    username: &str,
    password: &str,
) -> Result<(String, u64), ClientError> {
    let hash = format!("{:x}", md5::compute(format!("{}{}", time, password)));
    let resp = client
        .post(URL_AUTH)
        .json(&LoginReqMessage {
            login: username.to_string(),
            hash,
        })
        .send()
        .await?;

    if resp.status().is_success() {
        let login_resp = resp.json::<LoginSuccMessage>().await?;
        Ok((login_resp.hash, login_resp.difficulty))
    } else {
        Err(ClientError::UnexpectedResponse(format!(
            "Unexpected status code: {}",
            resp.status()
        )))
    }
}

/// Computes the proof of work based on the difficulty and hash.
fn calc_pow(difficulty: u64, hash: &str) -> u64 {
    let mut pow = 0;
    let mut new_hash = String::new();

    while !new_hash.starts_with(&"0".repeat(difficulty as usize)) {
        pow += 1;
        new_hash = format!("{:x}", md5::compute(format!("{}{}", hash, pow)));
    }

    pow
}

/// Asynchronously performs logout operation.
async fn logout(
    client: &reqwest::Client,
    hash: &str,
    username: &str,
    password: &str,
) -> Result<(), ClientError> {
    let new_hash = format!("{:x}", md5::compute(format!("{}{}", hash, password)));
    let resp = client
        .post(URL_LOGOUT)
        .json(&LogoutReqMessage {
            login: username.to_string(),
            hash: new_hash,
        })
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(ClientError::UnexpectedResponse(format!(
            "Unexpected status code: {}",
            resp.status()
        )))
    }
}

/// Executes a simple integration test for a single user asynchronously.
async fn simple_integration_test(
    client: &reqwest::Client,
    username: &str,
    password: &str,
) -> Result<(), ClientError> {
    let time = get_time(client, username).await?;
    let (hash, difficulty) = login(client, time, username, password).await?;

    let pow = calc_pow(difficulty, &hash);
    let quote_req_message = QuoteReqMessage {
        login: username.to_string(),
        pow,
    };
    let resp = client
        .post(URL_QUOTE)
        .json(&quote_req_message)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(ClientError::UnexpectedResponse(format!(
            "Unexpected status code: {}",
            resp.status()
        )));
    }

    let quote_resp: QuoteRespMessage = resp.json().await?;
    let pow = calc_pow(quote_resp.difficulty, &quote_resp.hash);
    let quote_req_message = QuoteReqMessage {
        login: username.to_string(),
        pow,
    };

    let resp = client
        .post(URL_QUOTE)
        .json(&quote_req_message)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(ClientError::UnexpectedResponse(format!(
            "Unexpected status code: {}",
            resp.status()
        )));
    }

    let _quote_resp: QuoteRespMessage = resp.json().await?;
    logout(client, &_quote_resp.hash, username, password).await?;
    Ok(())
}

/// Executes integration tests for multiple users concurrently.
async fn simple_multiclient_test(client: &reqwest::Client) -> Result<(), ClientError> {
    let users = [("one", "pass1"), ("two", "pass2"), ("three", "pass3")];

    let futures = users
        .iter()
        .map(|user| simple_integration_test(client, user.0, user.1));

    let results: Vec<Result<(), ClientError>> = stream::iter(futures)
        .buffer_unordered(users.len())
        .collect()
        .await;

    for result in results {
        result?;
    }

    Ok(())
}

/// Executes a set of predefined tests asynchronously.
async fn tests(client: &reqwest::Client) -> Result<(), ClientError> {
    simple_integration_test(client, "one", "pass1").await?;
    simple_multiclient_test(client).await?;
    Ok(())
}

/// Entry point for running client operations and tests.
fn main() {
    let client = reqwest::Client::new();
    let rt = Runtime::new().unwrap();
    rt.block_on(tests(&client))
        .unwrap_or_else(|e| eprintln!("Tests failed with error: {:?}", e));
}
