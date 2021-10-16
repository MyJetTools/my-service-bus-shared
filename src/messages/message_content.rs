use crate::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Debug)]
pub struct MySbMessageContent {
    pub id: MessageId,
    pub content: Vec<u8>,
    pub time: DateTimeAsMicroseconds,
}

impl MySbMessageContent {
    pub fn new(id: MessageId, content: Vec<u8>, time: DateTimeAsMicroseconds) -> Self {
        Self {
            id,
            content: content,
            time,
        }
    }
}
