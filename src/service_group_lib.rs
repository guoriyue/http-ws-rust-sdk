use crate::http_api_wrapper::HTTPAPIWrapper;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ServiceGroupLib {
    id2ids_mdown: Arc<Mutex<HashMap<String, Vec<String>>>>,
    ids2id_mdown: Arc<Mutex<HashMap<String, String>>>,
    id2ids_mup: Arc<Mutex<HashMap<String, Vec<String>>>>,
    ids2id_mup: Arc<Mutex<HashMap<String, String>>>,
}

impl ServiceGroupLib {
    pub fn new() -> Self {
        Self {
            id2ids_mdown: Arc::new(Mutex::new(HashMap::new())),
            ids2id_mdown: Arc::new(Mutex::new(HashMap::new())),
            id2ids_mup: Arc::new(Mutex::new(HashMap::new())),
            ids2id_mup: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn convert_list(
        &self,
        http_api: &HTTPAPIWrapper,
        character_ids: Vec<String>,
        is_message_down: bool,
        channel_id: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let (ids2id_lock, id2ids_lock) = if is_message_down {
            (&self.ids2id_mdown, &self.id2ids_mdown)
        } else {
            (&self.ids2id_mup, &self.id2ids_mup)
        };

        if character_ids.is_empty() {
            return Ok("".to_string());
        }

        let massive_str = character_ids.join("_");
        let mut ids2id = ids2id_lock.lock().await;
        let mut id2ids = id2ids_lock.lock().await;
        if !ids2id.contains_key(&massive_str) {
            let group_id = if is_message_down {
                http_api.create_service_group(character_ids.clone()).await?
            } else {
                let channel_id = channel_id.ok_or("A channel_id must be specified when is_message_down is False")?;
                http_api.create_channel_group(&channel_id, "A_message_up_group", character_ids.clone()).await?
            };

            ids2id.insert(massive_str.clone(), group_id.clone());
            id2ids.insert(group_id.clone(), character_ids);
            println!("Converted recipient list (is_mdown={}) to group id {} on process {}. Created new service group.", is_message_down, group_id, std::process::id());
            Ok(group_id)
        } else {
            let out = ids2id.get(&massive_str).unwrap().clone();
            println!("Converted recipient list (is_mdown={}) to group id {} on process {}. Group already exists.", is_message_down, out, std::process::id());
            Ok(out)
        }
    }
}

// #[tokio::main]
// async fn main() {
//     // Example usage
//     // let lib = ServiceGroupLib::new();
//     // let http_api = MockHttpApi {}; // Assuming a mock implementation of HttpApi trait
//     // let character_ids = vec!["id1".to_string(), "id2".to_string()];
//     // match lib.convert_list(&http_api, character_ids, true, None).await {
//     //     Ok(group_id) => println!("Group ID: {}", group_id),
//     //     Err(e) => eprintln!("Error: {}", e),
//     // }
// }
