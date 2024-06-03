use crate::service_group_lib::{ServiceGroupLib};
use crate::socket::{WebSocket, Protocol, JsonProtocol};
use crate::http_api_wrapper::{HTTPAPIWrapper};
use crate::types::{Config};

use serde_json::{json, Value};
use serde_derive::{Serialize, Deserialize};
use failure::{err_msg, Error};
use tungstenite::http::response;

pub struct Moobius {
    pub config: Config,
    pub http_client: HTTPAPIWrapper,
    pub ws_client: WebSocket<JsonProtocol>,
    pub service_group_lib: ServiceGroupLib,
}


impl Moobius {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let http_client = HTTPAPIWrapper::new(config.clone());
        let protocol = JsonProtocol;
        let ws_client = WebSocket::connect(protocol, &config.ws_server_uri).await?;
        let service_group_lib = ServiceGroupLib::new();
        Ok(Self {
            config,
            http_client,
            ws_client,
            service_group_lib,
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
    async fn create_character(&self, file_path: &str, name: &str, description: &str) {
        // let file_path = "src/cat_plastic_bag.png";
        let avatar_url = self.http_client.upload_file(file_path);
        
        // Example of calling create_character synchronously within an async context
        if let Ok(character) = self.http_client.create_character(self.config.service_id.as_ref().unwrap(), name, avatar_url.unwrap().as_str(), description) {
            println!("Character created: {:?}", character);
        } else {
            println!("Failed to create character");
        }
    }
    
    async fn on_fetch_characters(&mut self, body: &Value) {
        println!("Received fetch_characters: {:?}", body);
        let real_character_ids = self.http_client.fetch_real_characters(self.config.clone().channels[0].as_str(), self.config.clone().service_id.as_ref().unwrap());
        println!("real_character_ids: {:?}", real_character_ids);
        match real_character_ids {
            Ok(vec) => {
                // let vec_of_str: Vec<&str> = vec.iter().map(|s| s.as_str()).collect();
                let group_character_ids = self.service_group_lib.convert_list(&self.http_client, vec, true, None).await.unwrap();
                let response = self.ws_client.update_character_list(self.config.service_id.as_ref().unwrap(), self.config.channels[0].as_str(), group_character_ids.as_str(), group_character_ids.as_str()).await;
            },
            Err(e) => {
                println!("An error occurred: {}", e);
            }
        }
    }

    async fn on_fetch_buttons(&mut self, body: &Value) {
        println!("Received fetch_buttons: {:?}", body);
        // let sender = body["sender"].as_str().unwrap();
        // let to_whom = self.http_client.fetch_real_characters(body["channel_id"].as_str().unwrap(), self.config.service_id.as_ref().unwrap()).unwrap();
        // // send to everyone
        // for recipient in to_whom {
        //     let response = self.ws_client.send_buttons_from_database(body["channel_id"].as_str().unwrap(), recipient.as_str()).await;
        //     println!("response: {:?}", response);
        // }
    }
    
    async fn on_fetch_canvas(&mut self, body: &Value) {
        println!("Received fetch_canvas: {:?}", body);
    }

    async fn on_fetch_context_menu(&mut self, body: &Value) {
        println!("Received fetch_context_menu: {:?}", body);
    }

    async fn on_action(&mut self, body: &Value) {
        println!("Received action: {:?}", body);
        match body["subtype"].as_str() {
            Some("fetch_playground") => self.on_fetch_playground(&body).await,
            Some("fetch_channel_info") => self.on_fetch_channel_info(&body).await,
            Some("fetch_features") => self.on_fetch_features(&body).await,
            Some("fetch_characters") => self.on_fetch_characters(&body).await,
            Some("fetch_buttons") => self.on_fetch_buttons(&body).await,
            Some("fetch_canvas") => self.on_fetch_canvas(&body).await,
            Some("fetch_context_menu") => self.on_fetch_context_menu(&body).await,
            _ => {
                println!("Unknown action subtype: {:?}", body["subtype"]);
            },
        }
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
        if let Some(payload_type) = payload.get("type").and_then(|t| t.as_str()) {
            match payload_type {
                "copy" => self.on_copy_client(&payload["body"]).await,
                "update" => self.on_update(&payload["body"]).await,
                "msg_up" => self.on_message_up(&payload["body"]).await,
                "action" => self.on_action(&payload["body"]).await,
                "button_click" => self.on_button_click(&payload["body"]).await,
                "menu_click" => self.on_context_menu_click(&payload["body"]).await,
                _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Unknown type"))),
            }
        } else {
            println!("Missing type field in payload");
        }
        Ok(())
    }
}