use crate::types::{Config, Character};
use reqwest::blocking::{Client};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::blocking::multipart::{Form, Part};
use serde_json::json;
use serde_json::Value;
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, error};
use std::fs::File;
use std::io::{self, ErrorKind, Read};


pub struct HTTPAPIWrapper {
    http_client: Client,
    http_server_uri: String,
    email: String,
    password: String,
    access_token: String,
    refresh_token: String,
    headers: HeaderMap,
}

impl HTTPAPIWrapper {
    pub fn new(config: Config) -> Self {
        let http_client = Client::new();
        let http_server_uri = config.http_server_uri;
        let email = config.email;
        let password = config.password;
        let access_token = String::new();
        let refresh_token = String::new();
        let headers = HeaderMap::new();
        Self {
            http_client,
            http_server_uri,
            email,
            password,
            access_token,
            refresh_token,
            headers,
        }
    }

    /// Authenticates the user with the Moobius HTTP API.
    /// This method must be called before any other API calls.
    /// It returns a tuple containing the access token and refresh token.
    pub fn authenticate(&mut self) -> Result<(String, String), Box<dyn std::error::Error>> {
        let url = format!("{}/auth/sign_in", self.http_server_uri);
        let request_body = json!({
            "username": self.email,
            "password": self.password
        });
        // This uses the blocking client's post method

        let response_body = self.http_client.post(&url)
            .json(&request_body)
            .send()?  // Sends the POST request
            .json::<Value>()?;  // Parses the response body as JSON
        
        self.access_token = response_body["data"]["AuthenticationResult"]["AccessToken"]
            .as_str()
            .ok_or("Access Token not found in the response")?
            .to_string();

        self.refresh_token = response_body["data"]["AuthenticationResult"]["RefreshToken"]
            .as_str()
            .ok_or("Refresh Token not found in the response")?
            .to_string();

        self.headers.insert("Auth-Origin", HeaderValue::from_static("cognito"));
        self.headers.insert("Authorization", HeaderValue::from_str(&("Bearer ".to_string() + &self.access_token))?);
        Ok((self.access_token.clone(), self.refresh_token.clone()))
    }

    pub fn create_character(&self, service_id: &str, name: &str, avatar: &str, description: &str) -> Result<Character, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!("{}/api/service/character/create", self.http_server_uri);
        // Prepare JSON payload
        let request_body = json!({
            "service_id": service_id,
            "context": {
                "name": name,
                "avatar": avatar,
                "description": description
            }
        });

        // Send POST request
        let response_body = self.http_client.post(&url)
            .headers(self.headers.clone())
            .json(&request_body)
            .send()?  // Sends the POST request
            .json::<Value>()?;  // Parses the response body as JSON
        
        println!("Response body: {:?}", response_body);
        let character = Character {
            character_id: response_body["data"]["character_id"]
                .as_str()
                .ok_or("Character ID not found in the response")?
                .to_string(),
            name: response_body["data"]["context"]["name"]
                .as_str()
                .ok_or("Character name not found in the response")?
                .to_string(),
            avatar: response_body["data"]["context"]["avatar"]
                .as_str()
                .ok_or("Character avatar not found in the response")?
                .to_string(),
            description: response_body["data"]["context"]["description"]
                .as_str()
                .ok_or("Character description not found in the response")?
                .to_string(),
            character_context: response_body["data"]["context"]
                .as_object()
                .ok_or("Character context not found in the response")?
                .clone(),
        };
        Ok(character)
    }

    pub fn upload_file(&self, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let extension = file_path.rsplit('.').next().ok_or("Failed to extract file extension")?;
        let (upload_url, upload_fields) = self.upload_with_extension(extension)?;
        self.do_upload_file(&upload_url, &upload_fields, file_path)
    }

    fn upload_with_extension(&self, extension: &str) -> Result<(String, Value), Box<dyn std::error::Error>> {
        let url = format!("{}/file/upload", self.http_server_uri);
        let params = [("extension", extension)];
        let response = self.http_client.get(&url)
            .headers(self.headers.clone())
            .query(&params)
            .send()?
            .json::<Value>()?;

        let upload_url = response["data"]["url"]
            .as_str()
            .ok_or("Upload URL not found in the response")?
            .to_string();
        let upload_fields = response["data"]["fields"].clone();

        Ok((upload_url, upload_fields))
    }

    fn do_upload_file(&self, upload_url: &str, upload_fields: &serde_json::Value, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(buffer.as_mut())?;

        let full_url = format!("{}{}", upload_url, upload_fields["key"].as_str().unwrap_or_default());
        println!("full_url: {:?}", full_url);

        let form = Form::new().file("image", file_path)?;

        println!("upload_fields: {:?}", upload_fields);
        // Create a client and make a POST request
        let response = self.http_client.post(upload_url)
            .json(&upload_fields)
            .send()?;
        println!("do_upload_file Response: {:?}", response);
        // let response = self.http_client.post(full_url)
        //     .body(buffer)
        //     .send()?
        //     .json::<Value>()?;

        println!("do_upload_file Response: {:?}", response);
        Ok("".to_string())
        // let file_url = response["data"]["url"]
        //     .as_str()
        //     .ok_or("File URL not found in the response")?
        //     .to_string();
        // Ok(file_url)
    }

    pub fn fetch_real_characters(&self, channel_id: &str, service_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("{}/channel/userlist", self.http_server_uri);
        let params = [("channel_id", channel_id), ("service_id", service_id)];
        let response = self.http_client.get(&url)
            .headers(self.headers.clone())
            .query(&params)
            .send()?
            .json::<Value>()?;
    
        let user_ids: Vec<String> = response["data"]["userlist"]
            .as_array() // Ensure that the userlist is an array
            .unwrap_or(&Vec::new()) // Provide a default empty vector if not an array
            .iter() // Iterate over the array
            .filter_map(|user| {
                // Extract the `user_id` from each object, handling possible issues gracefully
                user["user_id"].as_str().map(|s| s.to_owned())
            })
            .collect();
        Ok(user_ids)
    }
}

