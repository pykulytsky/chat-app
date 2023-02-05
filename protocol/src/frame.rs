use colored::Colorize;
use std::fmt::Display;
use termion::{cursor, terminal_size};

use crate::{ConnectionError, ProtocolError, Result};
use bytes::{BufMut, Bytes};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

use chrono::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub color: Option<String>,
    pub avatar: Option<String>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.color.to_owned().unwrap_or(String::from(""));
        write!(f, "{}", self.username.color(color).bold())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Message {
    pub from: User,
    pub channel: String,
    pub body: String,
    pub created: DateTime<Utc>,
}

impl Message {
    pub fn new(from: User, channel: String, body: String) -> Self {
        Self {
            from,
            channel,
            body,
            created: Utc::now(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (columns, _) = terminal_size().unwrap();
        let msg = format!("{}: {}", self.from, self.body);
        let time = self.created.format("%H:%M").to_string();
        let offset: u16 = <usize as TryInto<u16>>::try_into(time.len()).unwrap() + 1;

        write!(
            f,
            "{}{}{}({})",
            msg,
            cursor::Right(columns),
            cursor::Left(offset),
            time
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Channel {
    pub name: String,
    pub messages: Vec<Message>,
    pub cover: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Frame {
    Authorize(User),
    Connect(Vec<Channel>),
    Message(Message),
    Bulk(Vec<Message>, Vec<Channel>),
    Channel(Channel),
    Ok,
    Error(String),
    Disconnect(User),
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

pub struct ChatCodec {}

impl ChatCodec {
    pub fn new() -> Self {
        ChatCodec {}
    }
}

impl Encoder<Frame> for ChatCodec {
    type Error = ProtocolError;
    fn encode(&mut self, item: Frame, dst: &mut bytes::BytesMut) -> Result<()> {
        let frame = bincode::serialize(&item)?;
        dst.reserve(frame.len());
        dst.put(&frame[..]);

        Ok(())
    }
}

impl Decoder for ChatCodec {
    type Item = Frame;
    type Error = ProtocolError;
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        let frame = bincode::deserialize(src)?;
        Ok(Some(frame))
    }
}

impl From<Message> for Frame {
    fn from(m: Message) -> Self {
        Self::Message(m)
    }
}

impl TryFrom<Frame> for Message {
    type Error = ConnectionError;
    fn try_from(frame: Frame) -> std::result::Result<Self, Self::Error> {
        match frame {
            Frame::Message(msg) => Ok(msg),
            _ => Err(ConnectionError::MessageParse),
        }
    }
}

impl TryFrom<Bytes> for Frame {
    type Error = ConnectionError;
    fn try_from(value: Bytes) -> std::result::Result<Self, ConnectionError> {
        match bincode::deserialize(&value) {
            Ok(frame) => Ok(frame),
            Err(_) => Err(ConnectionError::MessageParse),
        }
    }
}

impl TryInto<Bytes> for Frame {
    type Error = ConnectionError;
    fn try_into(self) -> std::result::Result<Bytes, ConnectionError> {
        match bincode::serialize(&self) {
            Ok(bytes) => Ok(Bytes::from(bytes)),
            Err(_) => Err(ConnectionError::MessageParse),
        }
    }
}
