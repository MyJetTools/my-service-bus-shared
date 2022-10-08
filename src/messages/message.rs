use my_service_bus_abstractions::MessageId;

use super::MySbMessageContent;

#[derive(Debug, Clone)]
pub enum MySbMessage {
    Loaded(MySbMessageContent),
    Gced(MessageId),
}

impl MySbMessage {
    pub fn content_size(&self) -> usize {
        match self {
            MySbMessage::Loaded(msg) => msg.content.len(),
            MySbMessage::Gced(_) => 0,
        }
    }

    pub fn get_id(&self) -> MessageId {
        match self {
            MySbMessage::Loaded(msg) => msg.id,
            MySbMessage::Gced(id) => *id,
        }
    }

    pub fn is_gced(&self) -> bool {
        match self {
            MySbMessage::Loaded(_) => false,
            MySbMessage::Gced(_) => true,
        }
    }
}
