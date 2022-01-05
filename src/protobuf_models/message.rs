use prost::{DecodeError, EncodeError};

use crate::bcl::BclDateTime;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageProtobufModel {
    #[prost(int64, tag = "1")]
    pub message_id: i64,
    #[prost(message, tag = "2")]
    pub created: Option<BclDateTime>,
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
