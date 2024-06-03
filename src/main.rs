use moobius::{Moobius, Config, HTTPAPIWrapper};
use reqwest::Error;

#[tokio::main]
async fn main() {
    let config = Config {
        http_server_uri: "https://api.moobius.net/".to_string(),
        ws_server_uri: "wss://ws.moobius.net/".to_string(),
        email: "".to_string(),
        password: "".to_string(),
        service_id: Some("".to_string()),
        channels: vec!["".to_string()],
    };
    
    let mut moobius_client = Moobius::new(config.clone()).await.unwrap();
    let (access_token, refresh_token) = moobius_client.http_client.authenticate().unwrap();
    let _ = moobius_client.ws_client.service_login(config.service_id.as_ref().unwrap(), &access_token).await.unwrap();    
    moobius_client.listen().await.unwrap();
}