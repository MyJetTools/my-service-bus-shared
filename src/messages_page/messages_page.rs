use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::{
    messages::{MySbMessage, MySbMessageContent},
    page_id::PageId,
    MessageId,
};

use super::MessagesPageData;

pub enum MessageSize {
    MessageIsReady(usize),
    NotLoaded,
    Missing,
}

pub struct MessagesPage {
    pub data: RwLock<MessagesPageData>,
    pub page_id: PageId,
    pub last_accessed: DateTimeAsMicroseconds,
}

impl MessagesPage {
    pub fn new(page_id: PageId) -> MessagesPage {
        MessagesPage {
            data: RwLock::new(MessagesPageData::new()),
            page_id,
            last_accessed: DateTimeAsMicroseconds::now(),
        }
    }

    pub async fn new_messages(&self, msgs: Vec<MySbMessageContent>) {
        let mut write_access = self.data.write().await;
        write_access.new_messages(msgs);
    }

    pub async fn restore(&self, msgs: Vec<MySbMessage>) {
        let mut write_access = self.data.write().await;
        write_access.restore(msgs);
    }

    pub async fn get_message_size(&self, message_id: &MessageId) -> MessageSize {
        let read_access = self.data.read().await;
        let msg = read_access.messages.get(message_id);

        if msg.is_none() {
            return MessageSize::Missing;
        }

        match msg.unwrap() {
            MySbMessage::Loaded(msg) => MessageSize::MessageIsReady(msg.content.len()),
            MySbMessage::Missing { id: _ } => MessageSize::Missing,
            MySbMessage::NotLoaded { id: _ } => MessageSize::NotLoaded,
        }
    }
}
