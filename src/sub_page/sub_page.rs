use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};
use tokio::sync::Mutex;

use crate::{protobuf_models::MessageProtobufModel, MessageId};

use super::SubPageId;

#[derive(Debug, Clone)]
pub enum MessageStatus {
    Loaded(MessageProtobufModel),
    Missing,
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: Mutex<BTreeMap<i64, MessageStatus>>,
    pub created: DateTimeAsMicroseconds,
    size: AtomicUsize,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: Mutex::new(BTreeMap::new()),
            created: DateTimeAsMicroseconds::now(),
            size: AtomicUsize::new(0),
        }
    }

    pub fn restored(sub_page_id: SubPageId, messages: BTreeMap<i64, MessageStatus>) -> Self {
        let messages_size = calculate_size(&messages);

        Self {
            sub_page_id,
            messages: Mutex::new(messages),
            created: DateTimeAsMicroseconds::now(),
            size: AtomicUsize::new(messages_size),
        }
    }

    pub async fn add_messages(&self, new_messages: Vec<MessageProtobufModel>) {
        let mut messages = self.messages.lock().await;

        for message in new_messages {
            self.size.fetch_add(message.data.len(), Ordering::SeqCst);
            if let Some(old_message) =
                messages.insert(message.message_id, MessageStatus::Loaded(message))
            {
                if let MessageStatus::Loaded(old_message) = old_message {
                    self.size
                        .fetch_sub(old_message.data.len(), Ordering::SeqCst);
                }
            }
        }
    }

    pub async fn get_message(&self, message_id: MessageId) -> Option<MessageStatus> {
        let messages = self.messages.lock().await;
        let result = messages.get(&message_id)?;
        Some(result.clone())
    }
    pub async fn get_messages(
        &self,
        from_id: MessageId,
        to_id: MessageId,
    ) -> Option<Vec<MessageStatus>> {
        let mut result = LazyVec::new();
        let messages = self.messages.lock().await;

        for message_id in from_id..=to_id {
            if let Some(msg) = messages.get(&message_id) {
                result.add(msg.clone());
            }
        }

        result.get_result()
    }

    pub fn get_size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
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
