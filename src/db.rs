use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct MoobiusDatabase {
    pub data: HashMap<String, Value>,
}

impl MoobiusDatabase {
    pub fn new() -> Self {
        MoobiusDatabase {
            data: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
    }

    pub fn get_field(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn remove_field(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn has_field(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn all_fields(&self) -> &HashMap<String, Value> {
        &self.data
    }

    pub fn add_to_list(&mut self, key: &str, item: Value) {
        if let Some(Value::Array(ref mut arr)) = self.data.get_mut(key) {
            arr.push(item);
        } else {
            println!("Field {} is not a list or does not exist.", key);
        }
    }
}