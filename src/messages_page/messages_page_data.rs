use std::collections::HashMap;

use crate::{
    messages::{MySbMessage, MySbMessageContent},
    queue_with_intervals::QueueWithIntervals,
    MessageId,
};

pub struct MessagesPageData {
    pub to_be_persisted: QueueWithIntervals,
    full_loaded_messages: QueueWithIntervals,
    pub messages: HashMap<MessageId, MySbMessage>,
    pub size: usize,
    pub is_being_persisted: bool,
}

impl MessagesPageData {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            size: 0,
            to_be_persisted: QueueWithIntervals::new(),
            is_being_persisted: false,
            full_loaded_messages: QueueWithIntervals::new(),
        }
    }

    pub fn new_messages(&mut self, msgs: Vec<MySbMessageContent>) {
        for msg in msgs {
            self.size += msg.content.len();

            self.to_be_persisted.enqueue(msg.id);
            self.full_loaded_messages.enqueue(msg.id);

            let old = self.messages.insert(msg.id, MySbMessage::Loaded(msg));

            if let Some(old) = old {
                self.size -= old.content_size();
            }
        }
    }

    pub fn restore(&mut self, msgs: Vec<MySbMessage>) {
        for msg in msgs {
            self.size += msg.content_size();

            if let MySbMessage::Loaded(full_message) = &msg {
                self.full_loaded_messages.enqueue(full_message.id);
            }

            let old = self.messages.insert(msg.get_id(), msg);

            if let Some(old) = old {
                self.size -= old.content_size();
            }
        }
    }

    fn gc(&mut self, messages_to_gc: Vec<MySbMessage>) {
        for msg_to_gc in &messages_to_gc {
            self.full_loaded_messages.remove(msg_to_gc.get_id());
        }

        self.restore(messages_to_gc);
    }

    pub fn gc_messages(&mut self, up_to_message_id: MessageId) {
        let mut messages_to_gc_result = None;

        for msg_id in &self.full_loaded_messages {
            if msg_id >= up_to_message_id {
                break;
            }

            if messages_to_gc_result.is_none() {
                messages_to_gc_result = Some(Vec::new())
            }

            if let Some(vec) = &mut messages_to_gc_result {
                vec.push(MySbMessage::NotLoaded { id: msg_id });
            }
        }

        if let Some(messages_to_gc) = messages_to_gc_result {
            self.gc(messages_to_gc);
        }
    }
}
