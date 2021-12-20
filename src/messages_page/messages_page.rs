use std::collections::BTreeMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    messages::{MySbMessage, MySbMessageContent},
    page_id::PageId,
    protobuf_models::MessageProtobufModel,
    queue_with_intervals::QueueWithIntervals,
    MessageId,
};

pub enum MessageSize {
    MessageIsReady(usize),
    NotLoaded,
    Missing,
}

pub struct MessagesPage {
    pub to_be_persisted: QueueWithIntervals,
    full_loaded_messages: QueueWithIntervals,
    pub messages: BTreeMap<MessageId, MySbMessage>,
    pub size: usize,
    pub is_being_persisted: bool,
    pub page_id: PageId,
    pub last_accessed: DateTimeAsMicroseconds,
}

impl MessagesPage {
    pub fn new(page_id: PageId) -> MessagesPage {
        MessagesPage {
            messages: BTreeMap::new(),
            size: 0,
            to_be_persisted: QueueWithIntervals::new(),
            is_being_persisted: false,
            full_loaded_messages: QueueWithIntervals::new(),
            page_id,
            last_accessed: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn new_with_missing_messages(
        page_id: PageId,
        from_id: MessageId,
        to_id: MessageId,
    ) -> MessagesPage {
        let mut result = Self::new(page_id);

        for id in from_id..to_id + 1 {
            result.messages.insert(id, MySbMessage::Missing { id });
        }

        result
    }

    pub fn new_message(&mut self, msg: MySbMessageContent) {
        self.size += msg.content.len();

        self.to_be_persisted.enqueue(msg.id);
        self.full_loaded_messages.enqueue(msg.id);

        let old = self.messages.insert(msg.id, MySbMessage::Loaded(msg));

        if let Some(old) = old {
            self.size -= old.content_size();
        }
    }

    pub fn restore(&mut self, msgs: Vec<MySbMessage>) {
        for msg in msgs {
            self.size += msg.content_size();

            if let MySbMessage::Loaded(full_message) = &msg {
                self.full_loaded_messages.enqueue(full_message.id);
            }

            let old = self.messages.insert(msg.get_id(), msg);

            if let Some(old) = old {
                self.size -= old.content_size();
            }
        }
    }

    pub fn get_message_size(&self, message_id: &MessageId) -> MessageSize {
        let msg = self.messages.get(message_id);

        if msg.is_none() {
            return MessageSize::Missing;
        }

        match msg.unwrap() {
            MySbMessage::Loaded(msg) => MessageSize::MessageIsReady(msg.content.len()),
            MySbMessage::Missing { id: _ } => MessageSize::Missing,
            MySbMessage::NotLoaded { id: _ } => MessageSize::NotLoaded,
        }
    }

    fn gc(&mut self, messages_to_gc: Vec<MySbMessage>) {
        for msg_to_gc in &messages_to_gc {
            self.full_loaded_messages.remove(msg_to_gc.get_id());
        }

        self.restore(messages_to_gc);
    }

    pub fn gc_messages(&mut self, up_to_message_id: MessageId) {
        let mut messages_to_gc_result = None;

        for msg_id in &self.full_loaded_messages {
            if msg_id >= up_to_message_id {
                break;
            }

            if messages_to_gc_result.is_none() {
                messages_to_gc_result = Some(Vec::new())
            }

            if let Some(vec) = &mut messages_to_gc_result {
                vec.push(MySbMessage::NotLoaded { id: msg_id });
            }
        }

        if let Some(messages_to_gc) = messages_to_gc_result {
            self.gc(messages_to_gc);
        }
    }

    pub fn get_messages_to_persist(&self) -> Option<Vec<MessageProtobufModel>> {
        let mut result = None;

        for msg in &self.to_be_persisted {
            if let Some(message) = self.messages.get(&msg) {
                if let MySbMessage::Loaded(content) = message {
                    if result.is_none() {
                        result = Some(Vec::new());
                    }

                    result.as_mut().unwrap().push(MessageProtobufModel {
                        created: Some(content.time.into()),
                        message_id: content.id,
                        data: content.content.clone(),
                        metadata: Vec::new(),
                    });
                }
            }
        }

        result
    }

    pub fn persisted(&mut self, messages: QueueWithIntervals) {
        for msg_id in &messages {
            self.to_be_persisted.remove(msg_id);
        }
    }
}

#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;

    #[test]
    pub fn test_gc_messages() {
        let mut page_data = MessagesPage::new(0);

        let mut msgs_to_restore = Vec::new();

        msgs_to_restore.push(MySbMessage::NotLoaded { id: 1 });
        msgs_to_restore.push(MySbMessage::NotLoaded { id: 2 });
        msgs_to_restore.push(MySbMessage::NotLoaded { id: 3 });
        msgs_to_restore.push(MySbMessage::NotLoaded { id: 4 });

        msgs_to_restore.push(MySbMessage::Loaded(MySbMessageContent {
            id: 5,
            time: DateTimeAsMicroseconds::now(),
            content: vec![5u8, 5u8, 5u8],
        }));

        msgs_to_restore.push(MySbMessage::Loaded(MySbMessageContent {
            id: 6,
            time: DateTimeAsMicroseconds::now(),
            content: vec![6u8, 6u8, 6u8],
        }));

        msgs_to_restore.push(MySbMessage::Loaded(MySbMessageContent {
            id: 7,
            time: DateTimeAsMicroseconds::now(),
            content: vec![7u8, 7u8, 7u8],
        }));

        msgs_to_restore.push(MySbMessage::Loaded(MySbMessageContent {
            id: 8,
            time: DateTimeAsMicroseconds::now(),
            content: vec![7u8, 7u8, 7u8],
        }));

        page_data.restore(msgs_to_restore);

        assert_eq!(4, page_data.full_loaded_messages.len());

        assert_eq!(true, page_data.full_loaded_messages.has_message(5));
        assert_eq!(true, page_data.full_loaded_messages.has_message(6));
        assert_eq!(true, page_data.full_loaded_messages.has_message(7));
        assert_eq!(true, page_data.full_loaded_messages.has_message(8));

        page_data.gc_messages(7);

        assert_eq!(2, page_data.full_loaded_messages.len());

        assert_eq!(false, page_data.full_loaded_messages.has_message(5));
        assert_eq!(false, page_data.full_loaded_messages.has_message(6));
        assert_eq!(true, page_data.full_loaded_messages.has_message(7));
        assert_eq!(true, page_data.full_loaded_messages.has_message(8));
    }

    #[test]
    fn test_new_with_all_missing() {
        let page_data = MessagesPage::new_with_missing_messages(0, 5, 10);

        assert_eq!(6, page_data.messages.len());

        for msg_id in 5..11 {
            let msg = page_data.messages.get(&msg_id).unwrap();

            if let MySbMessage::Missing { id } = msg {
                assert_eq!(*id, msg_id);
            } else {
                panic!("We should not be here");
            }
        }
    }
}
