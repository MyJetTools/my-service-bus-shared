use std::collections::BTreeMap;

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};
use tokio::sync::Mutex;

use crate::{protobuf_models::MessageProtobufModel, MessageId};

use super::SubPageId;

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: Mutex<BTreeMap<i64, MessageProtobufModel>>,
    pub created: DateTimeAsMicroseconds,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: Mutex::new(BTreeMap::new()),
            created: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn restored(sub_page_id: SubPageId, messages: BTreeMap<i64, MessageProtobufModel>) -> Self {
        Self {
            sub_page_id,
            messages: Mutex::new(messages),
            created: DateTimeAsMicroseconds::now(),
        }
    }

    pub async fn add_messages(&self, new_messages: Vec<MessageProtobufModel>) {
        let mut messages = self.messages.lock().await;

        for message in new_messages {
            messages.insert(message.message_id, message);
        }
    }

    pub async fn get_message(&self, message_id: MessageId) -> Option<MessageProtobufModel> {
        let messages = self.messages.lock().await;
        let result = messages.get(&message_id)?;
        Some(result.clone())
    }
    pub async fn get_messages(
        &self,
        from_id: MessageId,
        to_id: MessageId,
    ) -> Option<Vec<MessageProtobufModel>> {
        let mut result = LazyVec::new();
        let messages = self.messages.lock().await;

        for message_id in from_id..=to_id {
            if let Some(msg) = messages.get(&message_id) {
                result.add(msg.clone());
            }
        }

        result.get_result()
    }
}
