use std::collections::HashMap;

use crate::{
    page_id::{PageId, MESSAGES_IN_PAGE},
    MessageId, MySbMessage, MySbMessageContent,
};

pub struct MessagesPageRestoreSnapshot {
    pub page_id: PageId,
    pub requested_from_id: MessageId,
    pub requested_to_id: MessageId,
    pub messages: Option<HashMap<MessageId, MySbMessageContent>>,
    pub current_message_id: MessageId,
}

impl MessagesPageRestoreSnapshot {
    pub fn new(page_id: PageId, requested_from_id: MessageId, requested_to_id: MessageId) -> Self {
        Self {
            page_id,
            requested_from_id,
            requested_to_id,
            messages: None,
            current_message_id: page_id * MESSAGES_IN_PAGE,
        }
    }
    pub fn new_with_messages(
        page_id: PageId,
        requested_from_id: MessageId,
        requested_to_id: MessageId,
        messages: HashMap<MessageId, MySbMessageContent>,
    ) -> Self {
        Self {
            page_id,
            requested_from_id,
            requested_to_id,
            messages: Some(messages),
            current_message_id: page_id * MESSAGES_IN_PAGE,
        }
    }
}

impl Iterator for MessagesPageRestoreSnapshot {
    type Item = MySbMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_message_id > self.requested_to_id {
            return None;
        }

        let id = self.current_message_id;

        let result = if self.current_message_id < self.requested_from_id {
            MySbMessage::NotLoaded { id }
        } else {
            match &mut self.messages {
                Some(message) => {
                    if let Some(content) = message.remove(&id) {
                        MySbMessage::Loaded(content)
                    } else {
                        MySbMessage::Missing { id }
                    }
                }
                None => MySbMessage::Missing { id },
            }
        };

        self.current_message_id += 1;

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;

    #[test]
    fn test_general_case() {
        let mut messages = HashMap::new();

        messages.insert(
            2,
            MySbMessageContent {
                id: 2,
                content: vec![2, 2],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            },
        );

        messages.insert(
            4,
            MySbMessageContent {
                id: 4,
                content: vec![4, 4, 4],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            },
        );

        let snapshot = MessagesPageRestoreSnapshot::new_with_messages(0, 2, 5, messages);

        let mut result: Vec<MySbMessage> = snapshot.collect();

        assert_eq!(6, result.len());

        let el = result.remove(0);
        if let MySbMessage::NotLoaded { id } = el {
            assert_eq!(0, id)
        } else {
            panic!("Should not be here")
        }

        let el = result.remove(0);
        if let MySbMessage::NotLoaded { id } = el {
            assert_eq!(1, id)
        } else {
            panic!("Should not be here")
        }

        let el = result.remove(0);
        if let MySbMessage::Loaded(content) = el {
            assert_eq!(2, content.id);
            assert_eq!(vec![2u8, 2u8], content.content);
        } else {
            panic!("Should not be here")
        }

        let el = result.remove(0);
        if let MySbMessage::Missing { id } = el {
            assert_eq!(3, id);
        } else {
            panic!("Should not be here")
        }

        let el = result.remove(0);
        if let MySbMessage::Loaded(content) = el {
            assert_eq!(4, content.id);
            assert_eq!(vec![4u8, 4u8, 4u8], content.content);
        } else {
            panic!("Should not be here")
        }

        let el = result.remove(0);
        if let MySbMessage::Missing { id } = el {
            assert_eq!(5, id);
        } else {
            panic!("Should not be here")
        }
    }
}
