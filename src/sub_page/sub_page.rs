use std::collections::BTreeMap;

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::messages::MySbMessageContent;

use super::{SizeAndAmount, SubPageId};

pub enum GetMessageResult<'s> {
    Ok(&'s MySbMessageContent),
    Missing(MessageId),
    Gced(MessageId),
}

impl<'s> GetMessageResult<'s> {
    pub fn unwrap_as_message(&self) -> &'s MySbMessageContent {
        match self {
            GetMessageResult::Ok(msg) => msg,
            GetMessageResult::Missing(id) => panic!("Message {} is missing", id),
            GetMessageResult::Gced(id) => panic!("Message {} is gced", id),
        }
    }
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<MessageId, MySbMessageContent>,
    pub gced: QueueWithIntervals,
    pub to_persist: QueueWithIntervals,
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
            gced: QueueWithIntervals::new(),
            to_persist: QueueWithIntervals::new(),
        }
    }

    pub fn restore(
        sub_page_id: SubPageId,
        messages: BTreeMap<MessageId, MySbMessageContent>,
    ) -> Self {
        let mut size_and_amount = SizeAndAmount::new();

        for msg in messages.values() {
            size_and_amount.added(msg.content.len());
        }

        Self {
            sub_page_id,
            messages,
            created: DateTimeAsMicroseconds::now(),
            size_and_amount,
            gced: QueueWithIntervals::new(),
            to_persist: QueueWithIntervals::new(),
        }
    }

    pub fn add_message(&mut self, message: MySbMessageContent) -> Option<MySbMessageContent> {
        self.size_and_amount.added(message.content.len());
        self.to_persist.enqueue(message.id);

        if let Some(old_message) = self.messages.insert(message.id, message) {
            self.size_and_amount.removed(old_message.content.len());
            return Some(old_message);
        }

        None
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        if let Some(result) = self.messages.get(&msg_id) {
            return GetMessageResult::Ok(result);
        }
        if self.gced.has_message(msg_id) {
            return GetMessageResult::Gced(msg_id);
        } else {
            return GetMessageResult::Missing(msg_id);
        }
    }

    pub fn get_size_and_amount(&self) -> &SizeAndAmount {
        &self.size_and_amount
    }

    pub fn get_messages_amount(&self) -> usize {
        self.messages.len()
    }

    pub fn has_gced_messages(&self) -> bool {
        self.gced.len() > 0
    }

    fn get_first_message_id(&self) -> Option<MessageId> {
        for msg_id in self.messages.keys() {
            return Some(*msg_id);
        }

        None
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        let first_message_id = self.get_first_message_id();

        if first_message_id.is_none() {
            return;
        }

        let first_message_id = first_message_id.unwrap();

        for msg_id in first_message_id..self.sub_page_id.get_first_message_id_of_next_sub_page() {
            if min_message_id <= msg_id {
                break;
            }

            if let Some(message) = self.messages.remove(&msg_id) {
                self.size_and_amount.removed(message.content.len());
                self.gced.enqueue(msg_id);
            }
        }
    }

    pub fn persisted(&mut self, message_id: MessageId) {
        let _ = self.to_persist.remove(message_id);
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
            id: 0,
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        sub_page.add_message(MySbMessageContent {
            id: 1,
            content: vec![],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });
    }
}
