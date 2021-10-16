use crate::{MessageId, MySbMessageContent};

#[derive(Debug)]
pub enum MySbMessage {
    Loaded(MySbMessageContent),
    Missing { id: MessageId },
    NotLoaded { id: MessageId },
}

impl MySbMessage {
    pub fn content_size(&self) -> usize {
        match self {
            MySbMessage::Loaded(msg) => msg.content.len(),
            MySbMessage::Missing { id: _ } => 0,
            MySbMessage::NotLoaded { id: _ } => 0,
        }
    }

    pub fn get_id(&self) -> MessageId {
        match self {
            MySbMessage::Loaded(msg) => msg.id,
            MySbMessage::Missing { id } => *id,
            MySbMessage::NotLoaded { id } => *id,
        }
    }
}
