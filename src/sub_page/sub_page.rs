use std::collections::BTreeMap;

use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};

use crate::{MessageId, MySbMessage, MySbMessageContent};

use super::SubPageId;

pub enum SubPageMessages {
    AllAreMissing,
    Messages(BTreeMap<i64, MySbMessage>),
}

pub struct SubPage {
    pub sub_page_id: SubPageId,
    pub messages: SubPageMessages,
    pub created: DateTimeAsMicroseconds,
    size: usize,
}

impl SubPage {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: SubPageMessages::Messages(BTreeMap::new()),
            created: DateTimeAsMicroseconds::now(),
            size: 0,
        }
    }

    pub fn restored(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbMessage>) -> Self {
        let size = calculate_size(&messages);

        Self {
            sub_page_id,
            messages: SubPageMessages::Messages(messages),
            created: DateTimeAsMicroseconds::now(),
            size,
        }
    }

    pub fn create_with_all_missing(sub_page_id: SubPageId) -> Self {
        Self {
            sub_page_id,
            messages: SubPageMessages::AllAreMissing,
            created: DateTimeAsMicroseconds::now(),
            size: 0,
        }
    }

    pub fn add_message(&mut self, message: MySbMessageContent) {
        match &mut self.messages {
            SubPageMessages::AllAreMissing => {
                panic!("You can not insert message into sub page with all messages missing");
            }
            SubPageMessages::Messages(messages) => {
                self.size += message.content.len();
                if let Some(old_message) = messages.insert(message.id, MySbMessage::Loaded(message))
                {
                    if let MySbMessage::Loaded(old_message) = old_message {
                        self.size -= old_message.content.len();
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
                    self.size += message.content.len();
                    if let Some(old_message) =
                        messages.insert(message.id, MySbMessage::Loaded(message))
                    {
                        if let MySbMessage::Loaded(old_message) = old_message {
                            self.size -= old_message.content.len();
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

    pub fn get_size(&self) -> usize {
        self.size
    }
}

fn calculate_size(msgs: &BTreeMap<i64, MySbMessage>) -> usize {
    let mut size = 0;

    for msg in msgs.values() {
        if let MySbMessage::Loaded(msg) = msg {
            size += msg.content.len();
        }
    }

    size
}
