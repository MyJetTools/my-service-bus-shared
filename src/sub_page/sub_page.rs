use std::collections::BTreeMap;

use my_service_bus_abstractions::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{messages::MySbMessageContent, MySbMessage};

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
    pub messages: BTreeMap<i64, MySbMessage>,
    pub created: DateTimeAsMicroseconds,

    size_and_amount: SizeAndAmount,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: BTreeMap::new(),
            created: DateTimeAsMicroseconds::now(),
            size_and_amount: SizeAndAmount::new(),
        }
    }

    pub fn restore(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbMessage>) -> Self {
        let mut size_and_amount = SizeAndAmount::new();

        for msg in messages.values() {
            if let MySbMessage::Loaded(msg) = msg {
                size_and_amount.added(msg.content.len());
            }
        }

        Self {
            sub_page_id,
            messages,
            created: DateTimeAsMicroseconds::now(),
            size_and_amount,
        }
    }

    pub fn add_message(&mut self, message: MySbMessage) -> Option<MySbMessage> {
        let message_id = message.get_message_id();

        if !self.sub_page_id.is_my_message_id(message_id) {
            println!(
                "Somehow we are uploading message_id {} to sub_page {}. Skipping message...",
                message_id.get_value(),
                self.sub_page_id.get_value()
            );
            return None;
        }

        self.size_and_amount.added(message.get_content_size());

        if let Some(old_message) = self.messages.insert(message_id.get_value(), message) {
            self.size_and_amount.removed(old_message.get_content_size());
            return Some(old_message);
        }

        None
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        if let Some(msg) = self.messages.get(msg_id.as_ref()) {
            match msg {
                MySbMessage::Loaded(msg) => GetMessageResult::Message(msg),
                MySbMessage::Missing(_) => GetMessageResult::Missing,
            }
        } else {
            return GetMessageResult::GarbageCollected;
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
                self.size_and_amount.removed(message.get_content_size());
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::messages::MySbMessageContent;
    #[test]
    fn test_gc_messages() {
        let mut sub_page = SubPage::new(SubPageId::new(0));

        sub_page.add_message(
            MySbMessageContent {
                id: 0.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

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

        sub_page.add_message(
            MySbMessageContent {
                id: 1000.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1001.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        let gc_full_page = sub_page.gc_messages(5.into());

        assert!(!gc_full_page);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());
    }

    #[test]
    pub fn test_gc_messages_next_page() {
        let mut sub_page = SubPage::new(SubPageId::new(1));

        sub_page.add_message(
            MySbMessageContent {
                id: 1000.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1001.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        let gc_full_page = sub_page.gc_messages(9999.into());

        assert!(gc_full_page);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());

        let result = sub_page.get_message(1001.into());
        assert!(result.is_message_content());
    }
}
