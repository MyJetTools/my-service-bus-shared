use std::collections::BTreeMap;

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::messages::MySbMessageContent;

use super::{SizeAndAmount, SubPageId};

pub enum GetMessageResult<'s> {
    Message(&'s MySbMessageContent),
    Missing,
    GarbageCollected,
}

impl<'s> GetMessageResult<'s> {
    pub fn unwrap(&'s self) -> &'s MySbMessageContent {
        match self {
            GetMessageResult::Message(msg) => msg,
            GetMessageResult::Missing => panic!("Message is missing"),
            GetMessageResult::GarbageCollected => panic!("Message is garbage collected"),
        }
    }

    pub fn is_message_content(&self) -> bool {
        match self {
            GetMessageResult::Message(_) => true,
            _ => false,
        }
    }

    pub fn is_garbage_collected(&self) -> bool {
        match self {
            GetMessageResult::GarbageCollected => true,
            _ => false,
        }
    }
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<i64, MySbMessageContent>,
    pub to_persist: QueueWithIntervals,
    pub created: DateTimeAsMicroseconds,
    pub garbage_collected: QueueWithIntervals,
    pub loaded: QueueWithIntervals,
    size_and_amount: SizeAndAmount,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: BTreeMap::new(),
            created: DateTimeAsMicroseconds::now(),
            size_and_amount: SizeAndAmount::new(),
            to_persist: QueueWithIntervals::new(),
            garbage_collected: QueueWithIntervals::new(),
            loaded: QueueWithIntervals::new(),
        }
    }

    pub fn restore(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbMessageContent>) -> Self {
        let mut size_and_amount = SizeAndAmount::new();
        let mut loaded = QueueWithIntervals::new();

        for msg in messages.values() {
            size_and_amount.added(msg.content.len());
            loaded.enqueue(msg.id.get_value());
        }

        Self {
            sub_page_id,
            messages,
            created: DateTimeAsMicroseconds::now(),
            size_and_amount,
            to_persist: QueueWithIntervals::new(),
            garbage_collected: QueueWithIntervals::new(),
            loaded,
        }
    }

    pub fn add_message(&mut self, message: MySbMessageContent) -> Option<MySbMessageContent> {
        if !self.sub_page_id.is_my_message_id(message.id) {
            println!(
                "Somehow we are uploading message_id {} to sub_page {}. Skipping message...",
                message.id.get_value(),
                self.sub_page_id.get_value()
            );
            return None;
        }

        self.size_and_amount.added(message.content.len());
        self.to_persist.enqueue(message.id.get_value());
        self.loaded.enqueue(message.id.get_value());

        if let Some(old_message) = self.messages.insert(message.id.get_value(), message) {
            self.size_and_amount.removed(old_message.content.len());
            return Some(old_message);
        }

        None
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        if let Some(msg) = self.messages.get(msg_id.as_ref()) {
            return GetMessageResult::Message(msg);
        } else {
            if self.garbage_collected.has_message(msg_id.get_value()) {
                return GetMessageResult::GarbageCollected;
            }

            return GetMessageResult::Missing;
        }
    }

    pub fn get_size_and_amount(&self) -> &SizeAndAmount {
        &self.size_and_amount
    }

    pub fn get_messages_amount(&self) -> usize {
        self.messages.len()
    }

    fn get_first_message_id(&self) -> Option<MessageId> {
        for msg_id in self.messages.keys() {
            return Some(MessageId::new(*msg_id));
        }

        None
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) -> bool {
        let first_message_id_of_next_page =
            self.sub_page_id.get_first_message_id_of_next_sub_page();

        if min_message_id.get_value() >= first_message_id_of_next_page.get_value() {
            return true;
        }

        if min_message_id.get_value() < self.sub_page_id.get_first_message_id().get_value() {
            return false;
        }

        let first_message_id = self.get_first_message_id();

        if first_message_id.is_none() {
            return false;
        }

        let first_message_id = first_message_id.unwrap();

        for msg_id in first_message_id.get_value()..min_message_id.get_value() {
            if let Some(message) = self.messages.remove(&msg_id) {
                self.size_and_amount.removed(message.content.len());
                self.garbage_collected.enqueue(msg_id);
            }
        }

        false
    }

    pub fn persisted(&mut self, message_id: MessageId) {
        let _ = self.to_persist.remove(message_id.get_value());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::messages::MySbMessageContent;
    #[test]
    fn test_gc_messages() {
        let mut sub_page = SubPage::new(SubPageId::new(0));

        sub_page.add_message(MySbMessageContent {
            id: 0.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        sub_page.add_message(MySbMessageContent {
            id: 1.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        let gc_full_page = sub_page.gc_messages(1.into());

        assert!(!gc_full_page);

        let result = sub_page.get_message(0.into());
        assert!(result.is_garbage_collected());

        let result = sub_page.get_message(1.into());
        assert!(result.is_message_content());
    }

    #[test]
    pub fn test_gc_messages_prev_page() {
        let mut sub_page = SubPage::new(SubPageId::new(1));

        sub_page.add_message(MySbMessageContent {
            id: 1000.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        sub_page.add_message(MySbMessageContent {
            id: 1001.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        let gc_full_page = sub_page.gc_messages(5.into());

        assert!(!gc_full_page);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());
    }

    #[test]
    pub fn test_gc_messages_next_page() {
        let mut sub_page = SubPage::new(SubPageId::new(1));

        sub_page.add_message(MySbMessageContent {
            id: 1000.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        sub_page.add_message(MySbMessageContent {
            id: 1001.into(),
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        let gc_full_page = sub_page.gc_messages(9999.into());

        assert!(gc_full_page);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());

        let result = sub_page.get_message(1001.into());
        assert!(result.is_message_content());
    }
}
