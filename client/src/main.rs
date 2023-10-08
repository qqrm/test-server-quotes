use serde_json::json;
use std::collections::HashMap;
use tokio::runtime::Runtime;

use messages::time::RespTimeMessage;

async fn get_time(client: &reqwest::Client, username: &str) -> Result<String, reqwest::Error> {
    let resp = client
        .post("http://localhost:9999/time")
        .json(&json!({"login": username}))
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 200);
    dbg!(&resp);
    let qwe = resp.json::<RespTimeMessage>().await?;
    dbg!(&qwe);
    // let json_resp: HashMap<String, u64> = resp.json().await?;
    // assert!(json_resp["time"] > 0);
    // Ok(json_resp["time"].to_string())
    unimplemented!()
}

async fn login(
    client: &reqwest::Client,
    time: &str,
    username: &str,
    password: &str,
) -> Result<(String, u64), reqwest::Error> {
    let hash = format!("{:x}", md5::compute(format!("{}{}", time, password)));
    let resp = client
        .post("http://localhost:9999/auth")
        .json(&json!({
            "login": username,
            "hash": hash
        }))
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 200);
    let json_resp: HashMap<String, serde_json::Value> = resp.json().await?;
    let server_hash: String = json_resp["hash"].as_str().unwrap().to_string();
    let difficulty: u64 = json_resp["difficulty"].as_u64().unwrap();

    assert_eq!(
        format!("{:x}", md5::compute(server_hash.clone())),
        server_hash
    );
    Ok((server_hash, difficulty))
}

fn calc_pow(difficulty: u64, hash: &str) -> u64 {
    let mut pow = 0;
    let mut new_hash = String::from("    ");

    while &new_hash[..4] != &"0".repeat(difficulty as usize) {
        pow += 1;
        new_hash = format!("{:x}", md5::compute(format!("{}{}", hash, pow)));
    }

    pow
}

async fn logout(
    client: &reqwest::Client,
    hash: &str,
    username: &str,
    password: &str,
) -> Result<(), reqwest::Error> {
    let new_hash = format!("{:x}", md5::compute(format!("{}{}", hash, password)));
    let resp = client
        .post("http://localhost:9999/logout")
        .json(&json!({
            "login": username,
            "hash": new_hash
        }))
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 200);
    Ok(())
}

async fn simple_integration_test(
    client: &reqwest::Client,
    username: &str,
    password: &str,
) -> Result<(), reqwest::Error> {
    let time = get_time(client, username).await?;
    let (hash, difficulty) = login(client, &time, username, password).await?;

    let pow = calc_pow(difficulty, &hash);
    let resp = client
        .post("http://localhost:9999/quote")
        .json(&json!({
            "login": username,
            "pow": pow
        }))
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 200);
    let json_resp: HashMap<String, serde_json::Value> = resp.json().await?;
    let hash = json_resp["hash"].as_str().unwrap().to_string();
    let difficulty = json_resp["difficulty"].as_u64().unwrap();

    let pow = calc_pow(difficulty, &hash);
    let resp = client
        .post("http://localhost:9999/quote")
        .json(&json!({
            "login": username,
            "pow": pow
        }))
        .send()
        .await?;

    assert_eq!(resp.status().as_u16(), 200);
    logout(client, &hash, username, password).await?;
    Ok(())
}

async fn simple_multiclient_test(client: &reqwest::Client) -> Result<(), reqwest::Error> {
    let users = vec![("one", "pass1"), ("two", "pass2"), ("three", "pass3")];

    for user in &users {
        simple_integration_test(client, user.0, user.1).await?;
    }
    Ok(())
}

async fn tests(client: &reqwest::Client) -> Result<(), reqwest::Error> {
    simple_integration_test(client, "one", "pass1").await?;
    simple_multiclient_test(client).await?;
    Ok(())
}

fn main() {
    let client = reqwest::Client::new();
    let rt = Runtime::new().unwrap();
    rt.block_on(tests(&client)).unwrap();
}
