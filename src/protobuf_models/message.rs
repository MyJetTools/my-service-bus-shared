use prost::{DecodeError, EncodeError};

use crate::MySbMessageContent;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageProtobufModel {
    #[prost(int64, tag = "1")]
    pub message_id: i64,
    #[prost(int64, tag = "2")]
    pub created: i64,
    #[prost(bytes, tag = "3")]
    pub data: Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub headers: Vec<MessageMetaDataProtobufModel>,
}

impl MessageProtobufModel {
    pub fn parse(payload: &[u8]) -> Result<Self, DecodeError> {
        prost::Message::decode(payload)
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) -> Result<(), EncodeError> {
        prost::Message::encode(self, dest)
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessagesProtobufModel {
    #[prost(message, repeated, tag = "1")]
    pub messages: Vec<MessageProtobufModel>,
}

impl MessagesProtobufModel {
    pub fn parse(payload: &[u8]) -> Result<Self, DecodeError> {
        prost::Message::decode(payload)
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) -> Result<(), EncodeError> {
        prost::Message::encode(self, dest)
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageMetaDataProtobufModel {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}

impl From<&MySbMessageContent> for MessageProtobufModel {
    fn from(src: &MySbMessageContent) -> Self {
        Self {
            message_id: src.id,
            created: src.time.unix_microseconds,
            data: src.content.clone(),
            headers: convert_headers(src),
        }
    }
}

fn convert_headers(src: &MySbMessageContent) -> Vec<MessageMetaDataProtobufModel> {
    match &src.headers {
        Some(src) => {
            let mut result = Vec::with_capacity(src.len());

            for (key, value) in src {
                result.push(MessageMetaDataProtobufModel {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }

            return result;
        }
        None => return vec![],
    }
}
