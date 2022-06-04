use std::collections::HashMap;

use crate::MessageId;
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
