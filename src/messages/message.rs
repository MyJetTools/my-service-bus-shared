use my_service_bus_abstractions::MessageId;

use super::MySbMessageContent;

#[derive(Debug, Clone)]
pub enum MySbMessage {
    Loaded(MySbMessageContent),
    Missing(MessageId),
}

impl MySbMessage {
    pub fn content_size(&self) -> usize {
        match self {
            MySbMessage::Loaded(msg) => msg.content.len(),

            MySbMessage::Missing(_) => 0,
        }
    }

    pub fn get_id(&self) -> MessageId {
        match self {
            MySbMessage::Loaded(msg) => msg.id,
            MySbMessage::Missing(id) => *id,
        }
    }

    pub fn is_gced(&self) -> bool {
        match self {
            MySbMessage::Loaded(_) => false,
            MySbMessage::Missing(_) => false,
        }
    }

    pub fn is_missing(&self) -> bool {
        match self {
            MySbMessage::Loaded(_) => false,
            MySbMessage::Missing(_) => true,
        }
    }
}
