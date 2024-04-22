// use async_trait::async_trait;
// use crate::socket::{Client, ClientExt, Error};
// // use ezsockets::Client;
// // use ezsockets::{ClientConfig, connect, ClientExt, Error};
use crate::socket::{WebSocket, Protocol, JsonProtocol};
use crate::http_api_wrapper::{HTTPAPIWrapper};
use crate::types::{Config};
// use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use serde_derive::{Serialize, Deserialize};
use failure::{err_msg, Error};
use tungstenite::http::response;
// use std::io::{self, BufRead};
// use log::{info, warn, error};
// use tokio::main;
// use url::Url;

pub struct Moobius {
    pub config: Config,
    pub http_client: HTTPAPIWrapper,
    pub ws_client: WebSocket<JsonProtocol>,
}


impl Moobius {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let http_client = HTTPAPIWrapper::new(config.clone());
        let protocol = JsonProtocol;
        let ws_client = WebSocket::connect(protocol, &config.ws_server_uri).await?;

        Ok(Self {
            config,
            http_client,
            ws_client,
        })
    }

    pub async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let message = self.ws_client.recv::<Value>().await?;
            self.handle_received_payload(message).await?;
        }
    }
    async fn on_update(&self, body: &Value) {
        println!("Received update: {:?}", body);
    }
    async fn on_message_up(&self, body: &Value) {
        println!("Received message_up: {:?}", body);
    }
    async fn on_fetch_playground(&self, body: &Value) {
        println!("Received fetch_playground: {:?}", body);
    }
    async fn on_fetch_channel_info(&self, body: &Value) {
        println!("Received fetch_channel_info: {:?}", body);
    }
    async fn on_fetch_features(&self, body: &Value) {
        println!("Received fetch_features: {:?}", body);
    }
    async fn on_fetch_userlist(&mut self, body: &Value) {
        println!("Received fetch_userlist: {:?}", body);
        let file_path = "src/cat_plastic_bag.png";
        let real_character_ids = self.http_client.fetch_real_characters(self.config.clone().channels[0].as_str(), self.config.clone().service_id.as_ref().unwrap());
        // let avatar_url = self.http_client.upload_file(file_path);
        // println!("Avatar URL: {:?}", avatar_url);
        
        // // Example of calling create_character synchronously within an async context
        // if let Ok(character) = self.http_client.create_character(self.config.service_id.as_ref().unwrap(), "Her", "Avatar URL", "Description") {
        //     println!("Character created: {:?}", character);
        // } else {
        //     println!("Failed to create character");
        // }

        match real_character_ids {
            Ok(vec) => {
                let vec_of_str: Vec<&str> = vec.iter().map(|s| s.as_str()).collect();
                let response = self.ws_client.update_userlist(vec.get(0).unwrap().as_str(), self.config.channels[0].as_str(), vec_of_str.clone(), vec_of_str.clone()).await;
                match response {
                    Ok(_) => {
                        println!("Userlist updated successfully");
                    },
                    Err(e) => {
                        println!("An error occurred: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("An error occurred: {}", e);
            }
        }

    }
    
    async fn on_action(&mut self, body: &Value) {
        println!("Received action: {:?}", body);
        match body["subtype"].as_str() {
            Some("fetch_playground") => self.on_fetch_playground(&body).await,
            Some("fetch_channel_info") => self.on_fetch_channel_info(&body).await,
            Some("fetch_features") => self.on_fetch_features(&body).await,
            Some("fetch_userlist") => self.on_fetch_userlist(&body).await,
            _ => {
                println!("Unknown action subtype: {:?}", body["subtype"]);
            },
        }
        // Received payload: Object {"body": Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_playground")}, "type": String("action")}
        // Received action: Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_playground")}
        // Received payload: Object {"body": Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_channel_info")}, "type": String("action")}
        // Received action: Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_channel_info")}
        // Received payload: Object {"body": Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_features")}, "type": String("action")}
        // Received action: Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_features")}
        // Received payload: Object {"body": Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_userlist")}, "type": String("action")}
        // Received action: Object {"channel_id": String("8503c190-aa5e-4938-941e-c31a6f70f5d3"), "context": Object {}, "sender": String("321e7409-e19a-4608-a623-2bae497568d0"), "subtype": String("fetch_userlist")}
    }
    async fn on_button_click(&self, body: &Value) {
        println!("Received button_click: {:?}", body);
    }
    async fn on_context_menu_click(&self, body: &Value) {
        println!("Received context_menu_click: {:?}", body);
    }
    async fn on_copy_client(&self, body: &Value) {
        println!("Received copy_client: {:?}", body);
    }
    async fn handle_received_payload(&mut self, payload: Value) -> Result<(), Box<dyn std::error::Error>> {
        // Handle the payload here
        println!("Received payload: {:?}", payload);
        match payload["type"].as_str() {
            Some("update") => self.on_update(&payload["body"]).await,
            Some("msg_up") => self.on_message_up(&payload["body"]).await,
            Some("action") => self.on_action(&payload["body"]).await,
            Some("button_click") => self.on_button_click(&payload["body"]).await,
            Some("menu_click") => self.on_context_menu_click(&payload["body"]).await,
            Some("copy_client") => self.on_copy_client(&payload["body"]).await,
            _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Unknown or missing type"))),
        }
        Ok(())
    }
}