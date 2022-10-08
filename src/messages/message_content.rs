use std::collections::HashMap;

use crate::protobuf_models::MessageProtobufModel;
use my_service_bus_abstractions::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Debug, Clone)]
pub struct MySbMessageContent {
    pub id: MessageId,
    pub content: Vec<u8>,
    pub time: DateTimeAsMicroseconds,
    pub headers: Option<HashMap<String, String>>,
}

impl MySbMessageContent {
    pub fn new(
        id: MessageId,
        content: Vec<u8>,
        headers: Option<HashMap<String, String>>,
        time: DateTimeAsMicroseconds,
    ) -> Self {
        Self {
            id,
            content,
            time,
            headers,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            id: self.id,
            content: self.content.clone(),
            time: self.time,
            headers: self.headers.clone(),
        }
    }
}

impl From<MessageProtobufModel> for MySbMessageContent {
    fn from(src: MessageProtobufModel) -> Self {
        Self {
            id: src.message_id,
            time: DateTimeAsMicroseconds::new(src.created),
            content: src.data,
            headers: convert_headers(src.headers),
        }
    }
}

fn convert_headers(
    src: Vec<crate::protobuf_models::MessageMetaDataProtobufModel>,
) -> Option<HashMap<String, String>> {
    if src.len() == 0 {
        return None;
    }

    let mut result = HashMap::new();

    for header in src {
        result.insert(header.key, header.value);
    }

    Some(result)
}
