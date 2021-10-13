use crate::MessageId;

#[derive(Debug)]
pub struct MessageToSendModel {
    pub msg_id: MessageId,
    pub attempt_no: i32,
    pub msg_size: usize,
}
