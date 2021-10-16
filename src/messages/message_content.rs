use crate::{date_time::DateTimeAsMicroseconds, MessageId};

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
