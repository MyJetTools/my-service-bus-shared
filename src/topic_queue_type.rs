#[derive(Debug, Clone, Copy)]
pub enum TopicQueueType {
    Permanent = 0,
    DeleteOnDisconnect = 1,
    PermanentWithSingleConnection = 2,
}
