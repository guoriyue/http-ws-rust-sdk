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
    stream: Pin<Box<dyn Stream<Item = Result<Message, WsError>> + Send>>,
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

    pub async fn update_userlist(
        &mut self, 
        client_id: &str, 
        channel_id: &str, 
        user_list: Vec<&str>, 
        recipients: Vec<&str>
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // Ensure all items in user_list are strings (Rust's type system handles this).
        for ul in &user_list {
            if !ul.is_ascii() {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "User list must be a list of string ids.")));
            }
        }

        let message = json!({
            "type": "update",
            "request_id": Uuid::new_v4().to_string(),
            "client_id": client_id,
            "body": {
                "subtype": "update_userlist",
                "channel_id": channel_id,
                "recipients": recipients,
                "userlist": user_list,
                "context": {}
            }
        });

        println!("Constructed update_userlist message: {:?}", message);
        self.send(message.clone()).await?;

        Ok(message)
    }
}