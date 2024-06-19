use serde_derive::{Serialize, Deserialize};
use serde_json::{Value, json, Map};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub http_server_uri: String,
    pub ws_server_uri: String,
    pub email: String,
    pub password: String,
    pub service_id: Option<String>,
    pub channels: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub character_id: String,
    pub name: String,
    pub avatar: String,
    pub description: String,
    pub character_context: Map<String, Value>, // Using a HashMap to represent arbitrary JSON data
}

#[derive(Debug)]
pub struct MessageContent {
    pub filename: String,
    pub size: Option<u64>,
    pub path: String,
    pub text: Option<String>,
}

