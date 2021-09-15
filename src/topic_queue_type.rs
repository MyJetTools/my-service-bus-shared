#[derive(Debug, Clone, Copy)]
pub enum TopicQueueType {
    Permanent = 0,
    DeleteOnDisconnect = 1,
    PermanentWithSingleConnection = 2,
}

impl Into<TopicQueueType> for u8 {
    fn into(self) -> TopicQueueType {
        match self {
            0 => TopicQueueType::Permanent,
            1 => TopicQueueType::DeleteOnDisconnect,
            2 => TopicQueueType::PermanentWithSingleConnection,
            _ => TopicQueueType::DeleteOnDisconnect,
        }
    }
}

impl Into<u8> for TopicQueueType {
    fn into(self) -> u8 {
        self as u8
    }
}
