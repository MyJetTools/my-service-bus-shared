use std::collections::BTreeMap;

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};

use crate::{MessageId, MySbMessage, MySbMessageContent};

use super::{SizeAndAmount, SubPageId};

pub enum SubPageMessages {
    AllAreMissing,
    Messages(BTreeMap<i64, MySbMessage>),
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: SubPageMessages,
    pub created: DateTimeAsMicroseconds,
    size_and_amount: SizeAndAmount,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: SubPageMessages::Messages(BTreeMap::new()),
            created: DateTimeAsMicroseconds::now(),
            size_and_amount: SizeAndAmount::new(),
        }
    }

    pub fn restored(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbMessage>) -> Self {
        let size_and_amount = calculate_size_and_messages_amount(&messages);

        Self {
            sub_page_id,
            messages: SubPageMessages::Messages(messages),
            created: DateTimeAsMicroseconds::now(),
            size_and_amount,
        }
    }

    pub fn create_with_all_missing(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: SubPageMessages::AllAreMissing,
            created: DateTimeAsMicroseconds::now(),
            size_and_amount: SizeAndAmount::new(),
        }
    }

    pub fn add_message(&mut self, message: MySbMessageContent) {
        match &mut self.messages {
            SubPageMessages::AllAreMissing => {
                panic!("You can not insert message into sub page with all messages missing");
            }
            SubPageMessages::Messages(messages) => {
                self.size_and_amount.added(message.content.len());
                if let Some(old_message) = messages.insert(message.id, MySbMessage::Loaded(message))
                {
                    if let MySbMessage::Loaded(old_message) = old_message {
                        self.size_and_amount.removed(old_message.content.len());
                    }
                }
            }
        }
    }

    pub fn add_messages(&mut self, new_messages: Vec<MySbMessageContent>) {
        match &mut self.messages {
            SubPageMessages::AllAreMissing => {
                panic!("You can not insert message into sub page with all messages missing");
            }
            SubPageMessages::Messages(messages) => {
                for message in new_messages {
                    self.size_and_amount.added(message.content.len());
                    if let Some(old_message) =
                        messages.insert(message.id, MySbMessage::Loaded(message))
                    {
                        if let MySbMessage::Loaded(old_message) = old_message {
                            self.size_and_amount.removed(old_message.content.len());
                        }
                    }
                }
            }
        }
    }

    pub fn get_message(&self, message_id: MessageId) -> Option<&MySbMessage> {
        match &self.messages {
            SubPageMessages::AllAreMissing => None,
            SubPageMessages::Messages(messages) => messages.get(&message_id),
        }
    }

    pub fn get_content(&self, message_id: MessageId) -> Option<&MySbMessageContent> {
        match self.get_message(message_id)? {
            MySbMessage::Loaded(content) => Some(content),
            MySbMessage::Missing { id: _ } => None,
        }
    }
    pub fn get_messages(&self, from_id: MessageId, to_id: MessageId) -> Option<Vec<&MySbMessage>> {
        match &self.messages {
            SubPageMessages::AllAreMissing => None,
            SubPageMessages::Messages(messages) => {
                let mut result = LazyVec::new();

                for message_id in from_id..=to_id {
                    if let Some(msg) = messages.get(&message_id) {
                        result.add(msg);
                    }
                }

                result.get_result()
            }
        }
    }

    pub fn get_size_and_amount(&self) -> &SizeAndAmount {
        &self.size_and_amount
    }

    pub fn get_messages_amount(&self) -> usize {
        match &self.messages {
            SubPageMessages::AllAreMissing => 0,
            SubPageMessages::Messages(messages) => messages.len(),
        }
    }
}

fn calculate_size_and_messages_amount(msgs: &BTreeMap<i64, MySbMessage>) -> SizeAndAmount {
    let mut result = SizeAndAmount::new();

    for msg in msgs.values() {
        if let MySbMessage::Loaded(msg) = msg {
            result.added(msg.content.len());
        }
    }

    result
}
