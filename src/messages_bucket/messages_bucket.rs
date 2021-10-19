use std::{collections::HashMap, sync::Arc};

use crate::{messages_page::MessagesPage, queue_with_intervals::QueueWithIntervals, MessageId};

use super::MessageToSendModel;

pub struct MessagesBucket {
    pub page: Arc<MessagesPage>,
    pub messages: HashMap<MessageId, MessageToSendModel>,
    pub messages_size: usize,
    pub ids: QueueWithIntervals,
    pub intermediary_confirmed: QueueWithIntervals,
}

impl MessagesBucket {
    pub fn new(page: Arc<MessagesPage>) -> Self {
        Self {
            page,
            messages: HashMap::new(),
            messages_size: 0,
            ids: QueueWithIntervals::new(),
            intermediary_confirmed: QueueWithIntervals::new(),
        }
    }

    pub fn add(&mut self, msg_id: MessageId, attempt_no: i32, msg_size: usize) {
        let message = MessageToSendModel {
            msg_id,
            attempt_no,
            msg_size,
        };

        self.messages.insert(message.msg_id, message);
        self.messages_size += msg_size;

        self.ids.enqueue(msg_id);
    }

    pub fn messages_count(&self) -> usize {
        return self.messages.len();
    }

    pub fn messages_count_with_intermediary_confirmed(&self) -> usize {
        return self.messages.len() + (self.intermediary_confirmed.len() as usize);
    }

    pub fn intermediary_confirmed(&mut self, message_id: MessageId) {
        self.intermediary_confirmed.enqueue(message_id);
        self.remove(message_id);
    }

    pub fn remove(&mut self, message_id: MessageId) -> Option<MessageToSendModel> {
        let removed_message = self.messages.remove(&message_id)?;

        self.messages_size -= removed_message.msg_size;

        self.ids.remove(message_id);

        return Some(removed_message);
    }
}
