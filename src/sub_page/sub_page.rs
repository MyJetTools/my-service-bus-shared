use std::collections::{BTreeMap, HashMap};

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};

use crate::{MessageId, MySbMessageContent};

use super::{SizeAndAmount, SubPageId};

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<i64, MySbMessageContent>,
    pub gced: HashMap<i64, ()>,
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

    pub fn restored(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbMessageContent>) -> Self {
        let size_and_amount = calculate_size_and_messages_amount(&messages);

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

    pub fn add_messages(&mut self, new_messages: Vec<MySbMessageContent>) {
        for message in new_messages {
            self.size_and_amount.added(message.content.len());
            self.gced.remove(&message.id);
            if let Some(old_message) = self.messages.insert(message.id, message) {
                self.size_and_amount.removed(old_message.content.len());
            }
        }
    }

    pub fn get_message(&self, message_id: MessageId) -> Option<&MySbMessageContent> {
        self.messages.get(&message_id)
    }

    pub fn get_messages(
        &self,
        from_id: MessageId,
        to_id: MessageId,
    ) -> Option<Vec<&MySbMessageContent>> {
        let mut result = LazyVec::new();

        for message_id in from_id..=to_id {
            if let Some(msg) = self.messages.get(&message_id) {
                result.add(msg);
            }
        }

        result.get_result()
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

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        let mut messages_to_gc = None;

        for msg_id in self.messages.keys() {
            if *msg_id >= min_message_id {
                break;
            }

            messages_to_gc = Some(Vec::new());
            messages_to_gc.as_mut().unwrap().push(*msg_id);
        }

        if let Some(messages_to_gc) = messages_to_gc {
            for message_to_gc in messages_to_gc {
                if let Some(message) = self.messages.remove(&message_to_gc) {
                    self.size_and_amount.removed(message.content.len());
                    self.gced.insert(message_to_gc, ());
                }
            }
        }
    }
}

fn calculate_size_and_messages_amount(msgs: &BTreeMap<i64, MySbMessageContent>) -> SizeAndAmount {
    let mut result = SizeAndAmount::new();

    for msg in msgs.values() {
        result.added(msg.content.len());
    }

    result
}
