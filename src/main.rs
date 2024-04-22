use moobius::{Moobius, Config, HTTPAPIWrapper};
use reqwest::Error;

#[tokio::main]
async fn main() {
    let config = Config {
        http_server_uri: "http://localhost:3000".to_string(),
        ws_server_uri: "ws://localhost:3000".to_string(),
        email: "aaa".to_string(),
        password: "aaa".to_string(),
        service_id: Some("aaa".to_string()),
        channels: vec!["aaa".to_string()],
    };
    
    let mut moobius_client = Moobius::new(config.clone()).await.unwrap();
    let (access_token, refresh_token) = moobius_client.http_client.authenticate().unwrap();
    let _ = moobius_client.ws_client.service_login(config.service_id.as_ref().unwrap(), &access_token).await.unwrap();    
    moobius_client.listen().await.unwrap();
}