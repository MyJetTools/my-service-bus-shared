use std::collections::BTreeMap;

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};

use crate::{protobuf_models::MessageProtobufModel, MessageId};

use super::SubPageId;

#[derive(Debug, Clone)]
pub enum MessageStatus {
    Loaded(MessageProtobufModel),
    Missing,
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<i64, MessageStatus>,
    pub created: DateTimeAsMicroseconds,
    size: usize,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: BTreeMap::new(),
            created: DateTimeAsMicroseconds::now(),
            size: 0,
        }
    }

    pub fn restored(sub_page_id: SubPageId, messages: BTreeMap<i64, MessageStatus>) -> Self {
        let size = calculate_size(&messages);

        Self {
            sub_page_id,
            messages: messages,
            created: DateTimeAsMicroseconds::now(),
            size,
        }
    }

    pub fn add_message(&mut self, message: MessageProtobufModel) {
        self.size += message.data.len();
        if let Some(old_message) = self
            .messages
            .insert(message.message_id, MessageStatus::Loaded(message))
        {
            if let MessageStatus::Loaded(old_message) = old_message {
                self.size -= old_message.data.len();
            }
        }
    }

    pub fn add_messages(&mut self, new_messages: Vec<MessageProtobufModel>) {
        for message in new_messages {
            self.size += message.data.len();
            if let Some(old_message) = self
                .messages
                .insert(message.message_id, MessageStatus::Loaded(message))
            {
                if let MessageStatus::Loaded(old_message) = old_message {
                    self.size -= old_message.data.len();
                }
            }
        }
    }

    pub fn get_message(&self, message_id: MessageId) -> Option<&MessageStatus> {
        self.messages.get(&message_id)
    }
    pub fn get_messages(&self, from_id: MessageId, to_id: MessageId) -> Option<Vec<MessageStatus>> {
        let mut result = LazyVec::new();

        for message_id in from_id..=to_id {
            if let Some(msg) = self.messages.get(&message_id) {
                result.add(msg.clone());
            }
        }

        result.get_result()
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

fn calculate_size(msgs: &BTreeMap<i64, MessageStatus>) -> usize {
    let mut size = 0;

    for msg in msgs.values() {
        if let MessageStatus::Loaded(msg) = msg {
            size += msg.data.len();
        }
    }

    size
}
