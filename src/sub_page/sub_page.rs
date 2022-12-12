use std::collections::{BTreeMap, HashMap};

use my_service_bus_abstractions::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::messages::MySbMessageContent;

use super::{SizeAndAmount, SubPageId};

pub enum GetMessageResult<'s> {
    Ok(&'s MySbMessageContent),
    Missing(MessageId),
    Gced(MessageId),
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<MessageId, MySbMessageContent>,
    pub gced: HashMap<MessageId, ()>,
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
            gced: HashMap::new(),
        }
    }

    pub fn restored(sub_page_id: SubPageId, src: Option<Vec<MySbMessageContent>>) -> Self {
        let mut messages = BTreeMap::new();

        let mut size_and_amount = SizeAndAmount::new();

        if let Some(src) = src {
            for msg in src {
                size_and_amount.added(msg.content.len());
                messages.insert(msg.id, msg);
            }
        }

        Self {
            sub_page_id,
            messages,
            created: DateTimeAsMicroseconds::now(),
            size_and_amount,
            gced: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, message: MySbMessageContent) {
        self.size_and_amount.added(message.content.len());
        self.gced.remove(&message.id);

        if let Some(old_message) = self.messages.insert(message.id, message) {
            self.size_and_amount.removed(old_message.content.len());
        }
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        if let Some(result) = self.messages.get(&msg_id) {
            return GetMessageResult::Ok(result);
        }
        if self.gced.contains_key(&msg_id) {
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
        !self.gced.is_empty()
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
                self.gced.insert(msg_id, ());
            }
        }
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
