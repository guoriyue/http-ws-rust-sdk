#![feature(async_await, async_closure)]
use crate::types::{Config};

use failure::{err_msg, Error};
use futures::Stream as _;
use futures3::compat::{Future01CompatExt, Sink01CompatExt, Stream01CompatExt};
use futures3::{Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Borrow;
use std::pin::Pin;
use tokio_tungstenite::connect_async;
use tungstenite::error::Error as WsError;
use tungstenite::Message;
use url::Url;
use log::{info, error};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Protocol {
    fn serialize(&self, obj: &impl Serialize) -> Result<Vec<u8>, Error>;
    fn deserialize<T: for <'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, Error>;
}

pub struct JsonProtocol;

impl Protocol for JsonProtocol {
    fn serialize(&self, obj: &impl Serialize) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(obj).map_err(Error::from)
    }

    fn deserialize<T: for <'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, Error> {
        serde_json::from_slice(data).map_err(Error::from)
    }
}

pub struct WebSocket<T: Protocol> {
    protocol: T,
    sink: Pin<Box<dyn Sink<Message, Error = WsError> + Send>>,
    stream: Pin<Box<dyn Stream<Item = Result<Message, WsError>> + Send>>
}

impl<T: Protocol> WebSocket<T> {
    pub async fn connect(protocol: T, url: impl AsRef<str>) -> Result<Self, Error> {
        let url = Url::parse(url.as_ref())?;
        let (ws_stream, _) = connect_async(url).compat().await?;
        let (sink, stream) = ws_stream.split();
        let (sink, stream) = (sink.sink_compat(), stream.compat());
        let (sink, stream) = (Box::pin(sink), Box::pin(stream));
        Ok(Self { protocol, sink, stream })
    }

    pub async fn send<REQ: Serialize>(&mut self, value: impl Borrow<REQ>) -> Result<(), Error> {
        let data = self.protocol.serialize(value.borrow())?;
        // Convert the byte vector to a UTF-8 string
        let message_string = String::from_utf8(data).map_err(Error::from)?;
        // Create a text WebSocket message
        let msg = Message::Text(message_string);
        // Send the text message through the WebSocket
        self.sink.send(msg).await?;
        Ok(())
    }

    pub async fn recv<RESP: for <'de> Deserialize<'de>>(&mut self) -> Result<RESP, Error> {
        loop {
            let msg = self.stream.next().await;
            let msg = msg.ok_or_else(|| err_msg("websocket stream ended"))??;
            match msg {
                Message::Text(text) => {
                    let value = self.protocol.deserialize(text.as_bytes())?;
                    return Ok(value);
                }
                Message::Binary(data) => {
                    let value = self.protocol.deserialize(&data)?;
                    return Ok(value);
                }
                Message::Ping(_) | Message::Pong(_) => {}
                Message::Close(_) => {
                    return Err(err_msg("wsbsocket closed"));
                }
            }
        }
    }

    pub async fn service_login(&mut self, service_id: &str, access_token: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let message = json!({
            "type": "service_login",
            "request_id": Uuid::new_v4().to_string(),
            "auth_origin": "cognito",
            "access_token": access_token,
            "service_id": service_id,
        });

        println!("service_login message: {:?}", message);
        self.send(message.clone()).await?;

        Ok(message)
    }

    pub async fn update_character_list(
        &mut self, 
        service_id: &str, 
        channel_id: &str, 
        characters: &str,
        recipients: &str
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let message = json!({
            "type": "update",
            "request_id": Uuid::new_v4().to_string(),
            "service_id": service_id,
            "body": {
                "subtype": "update_characters",
                "channel_id": channel_id,
                "recipients": recipients,
                "content": {"characters": characters}
            }
        });
        
        self.send(message.clone()).await?;

        Ok(message)
    }

    pub async fn update_buttons(
        &mut self,
        service_id: &str,
        channel_id: &str,
        buttons: Vec<Value>,
        recipients: &str
    ) -> Result<Value, Box<dyn std::error::Error>> {

        let button_dicts: Vec<Value> = buttons.iter().map(|b| b.clone()).collect();
        let message = json!({
            "type": "update",
            "request_id": Uuid::new_v4().to_string(),
            "service_id": service_id,
            "body": {
                "subtype": "update_buttons",
                "channel_id": channel_id,
                "recipients": recipients,
                "content": button_dicts,
                "group_id": "temp",
                "context": {}
            }
        });
        
        self.send(message.clone()).await?;
        Ok(message)
    }

    pub async fn message_up(
        &mut self,
        user_id: &str,
        service_id: &str,
        channel_id: &str,
        recipients: &[&str],
        subtype: &str,
        content: &Value
    ) -> Result<Value, Box<dyn std::error::Error>> {
        if recipients.is_empty() {
            return Ok(json!(null));
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let message = json!({
            "type": "message_up",
            "request_id": Uuid::new_v4().to_string(),
            "user_id": user_id,
            "service_id": service_id,
            "body": {
                "subtype": subtype,
                "channel_id": channel_id,
                "content": content,
                "recipients": recipients,
                "timestamp": timestamp,
                "context": {}
            }
        });
        
        self.send(message.clone()).await?;

        Ok(message)
    }

    pub async fn message_down(
        &mut self,
        service_id: &str,
        channel_id: &str,
        recipients: &str,
        subtype: &str,
        content: &str,
        sender: &str
    ) -> Result<Value, Box<dyn std::error::Error>> {
        if recipients.is_empty() {
            return Ok(json!(null));
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        
        let message = json!({
            "type": "message_down",
            "request_id": Uuid::new_v4().to_string(),
            "service_id": service_id,
            "body": {
                "subtype": subtype,
                "channel_id": channel_id,
                "content": json!({"text": content, "path": content}),
                "recipients": recipients,
                "timestamp": timestamp,
                "sender": sender,
                "context": {}
            }
        });


        self.send(message.clone()).await?;

        Ok(message)
    }

}