use std::vec;

use crate::service_group_lib::{ServiceGroupLib};
use crate::db::{MoobiusDatabase};
use crate::socket::{WebSocket, Protocol, JsonProtocol};
use crate::http_api_wrapper::{HTTPAPIWrapper};
use crate::types::{Config, MessageContent};
use crate::Character;

use serde_json::{json, Value};
use serde_derive::{Serialize, Deserialize};
use failure::{err_msg, Error};
use tungstenite::http::response;


pub struct Moobius {
    pub config: Config,
    pub http_client: HTTPAPIWrapper,
    pub ws_client: WebSocket<JsonProtocol>,
    pub service_group_lib: ServiceGroupLib,
    pub db: MoobiusDatabase,
}


impl Moobius {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let http_client = HTTPAPIWrapper::new(config.clone());
        let protocol = JsonProtocol;
        let ws_client = WebSocket::connect(protocol, &config.ws_server_uri).await?;
        let service_group_lib = ServiceGroupLib::new();
        let db = MoobiusDatabase::new();
        Ok(Self {
            config,
            http_client,
            ws_client,
            service_group_lib,
            db,
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
    async fn create_character(&mut self, file_path: &str, name: &str, description: &str) {
        let avatar_url = self.http_client.upload_file(file_path);
        if !self.db.has_field("virtual_characters") {
            self.db.add_field("virtual_characters", Value::Array(vec![]));
        }
        // Example of calling create_character synchronously within an async context
        if let Ok(character) = self.http_client.create_character(self.config.service_id.as_ref().unwrap(), name, avatar_url.unwrap().as_str(), description) {
            println!("Character created: {:?}", character);
            self.db.add_to_list("virtual_characters", serde_json::to_value(character).unwrap());
        } else {
            println!("Failed to create character");
        }
    }

    async fn on_fetch_characters(&mut self, body: &Value) {
        let real_character_ids: Result<Vec<String>, Box<dyn std::error::Error>> = self.http_client.fetch_real_characters(self.config.clone().channels[0].as_str(), self.config.clone().service_id.as_ref().unwrap());
        if self.db.has_field("virtual_characters") {
            let virtual_character_ids: Vec<Value> = self.db.get_field("virtual_characters").unwrap().as_array().unwrap().clone();
            let mut virtual_character_ids: Vec<String> = virtual_character_ids.into_iter()
                .filter_map(|v| v.get("character_id").and_then(|id| id.as_str()).map(|s| s.to_string()))
                .collect();
            let mut total_character_list: Vec<String> = real_character_ids.unwrap();
            total_character_list.append(&mut virtual_character_ids);
            let group_character_ids = self.service_group_lib.convert_list(&self.http_client, total_character_list, true, None).await.unwrap();
            let response = self.ws_client.update_character_list(self.config.service_id.as_ref().unwrap(), self.config.channels[0].as_str(), group_character_ids.as_str(), group_character_ids.as_str()).await;

        } else {
            let total_character_list: Vec<String> = real_character_ids.unwrap();
            let group_character_ids = self.service_group_lib.convert_list(&self.http_client, total_character_list, true, None).await.unwrap();
            let response = self.ws_client.update_character_list(self.config.service_id.as_ref().unwrap(), self.config.channels[0].as_str(), group_character_ids.as_str(), group_character_ids.as_str()).await;
        }
    }

    async fn on_fetch_buttons(&mut self, body: &Value) {
        let button_list_str = std::fs::read_to_string("src/buttons.json").unwrap();
        let button_list: Vec<Value> = serde_json::from_str(&button_list_str).unwrap();
        let channel_id = body["channel_id"].as_str().unwrap();
        let sender = body["sender"].as_str().unwrap();
        let group_recipients = self.service_group_lib.convert_list(&self.http_client, vec![sender.to_string()], true, None).await.unwrap();
        let response = self.ws_client.update_buttons(self.config.service_id.as_ref().unwrap(), channel_id, button_list, group_recipients.as_str()).await;
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
            Some("fetch_characters") => self.on_fetch_characters(&body).await,
            Some("fetch_buttons") => self.on_fetch_buttons(&body).await,
            Some("fetch_canvas") => self.on_fetch_canvas(&body).await,
            Some("fetch_context_menu") => self.on_fetch_context_menu(&body).await,
            _ => {
                println!("Unknown action subtype: {:?}", body["subtype"]);
            },
        }
    }
    async fn on_button_click(&mut self, body: &Value) {
        // Check and extract fields safely
        let channel_id = match body["channel_id"].as_str() {
            Some(id) => id.to_string(),
            None => {
                println!("Error: 'channel_id' not found in the body");
                return;
            }
        };
        
        let button_id = match body["button_id"].as_str() {
            Some(id) => id.to_string(),
            None => {
                println!("Error: 'button_id' not found in the body");
                return;
            }
        };
    
        let who_clicked = match body["sender"].as_str() {
            Some(id) => id.to_string(),
            None => {
                println!("Error: 'sender' not found in the body");
                return;
            }
        };
    
        let to_whom = match self.http_client.fetch_real_characters(&channel_id, self.config.service_id.as_ref().unwrap()) {
            Ok(result) => result,
            Err(e) => {
                println!("Error fetching real characters: {:?}", e);
                return;
            }
        };
        
        let value = body["arguments"].get(0)
                                     .and_then(|arg| arg["value"].as_str())
                                     .map(|v| v.to_lowercase());
    
        // Handle button click based on button_id
        match button_id.as_str() {
            "message_btn" => {
                match value.as_deref() {
                    Some("text") => {
                        let some_text: String = "Hello, World!".to_string();
                        self.send_text_message(some_text, &channel_id, &who_clicked, to_whom, 1000).await;
                    },
                    Some("image") => {
                        let cat_in_plastic_bag = "src/cat_plastic_bag.png";
                        self.send_image_message(cat_in_plastic_bag, &channel_id, &who_clicked, to_whom).await;
                    },
                    _ => {
                        println!("Unknown value message_btn: {:?}", value);
                    }
                }
            },
            "user_btn" => {
                match value.as_deref() {
                    Some("make mickey") => {
                        let new_mickey = self.create_character("src/mickey.png", "Mickey", "A friendly mouse").await;
                        let empty_body = json!({});
                        self.on_fetch_characters(&empty_body).await;
                    },
                    Some("mickey talk") => {
                        let last_mickey_id = self.db.get_field("virtual_characters").unwrap().as_array().unwrap().last().unwrap().get("character_id").unwrap().as_str().unwrap().to_string();
                        self.send_text_message("M-I-C-K-E-Y M-O-U-S-E!".to_string(), &channel_id, &last_mickey_id, vec![who_clicked], 1000).await;
                    },
                    _ => {
                        println!("Unknown value user_btn: {:?}", value);
                    }
                }
            },
            "command_btn" => {
                let cmds = "
                \"show\" (send to service): Show buttons and canvas.
                \"hide\" (send to service): Hide buttons and canvas.
                \"reset\" (send to service): Reset Mickeys and refresh buttons.
                ".trim().replace('\n', "\n\n");
                self.send_text_message(cmds, &channel_id, &who_clicked, vec![who_clicked.clone()], 1000).await;
            },
            _ => {
                println!("Unknown button_id: {}", button_id);
            }
        }
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
                "message_up" => self.on_message_up(&payload["body"]).await,
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
    async fn send_text_message(
        &mut self,
        the_message: String,
        channel_id: &str,
        sender: &str,
        recipients: Vec<String>,
        len_limit: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = the_message;

        if content.len() > len_limit {
            content = self.limit_len(&content, len_limit);
        }

        let group_recipients = self.service_group_lib.convert_list(&self.http_client, recipients, true, None).await?;
        let response = self.ws_client.message_down( self.config.service_id.as_ref().unwrap(), &channel_id, &group_recipients, "text", &content, &sender).await?;
        Ok(())
    }

    async fn send_image_message(
        &mut self,
        file_path: &str,
        channel_id: &str,
        sender: &str,
        recipients: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let avatar_url = self.http_client.upload_file(file_path);
        let group_recipients = self.service_group_lib.convert_list(&self.http_client, recipients, true, None).await?;
        let response = self.ws_client.message_down( self.config.service_id.as_ref().unwrap(), &channel_id, &group_recipients, "image", avatar_url.unwrap().as_str(), &sender).await?;
        Ok(())
    }

    fn limit_len(&self, text: &str, len: usize) -> String {
        if text.len() > len {
            text[..len].to_string()
        } else {
            text.to_string()
        }
    }

}