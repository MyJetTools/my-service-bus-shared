use crate::MessageId;

use super::SubPageMessagesIterator;

pub const SUB_PAGE_MESSAGES_AMOUNT: usize = 1000;

#[derive(Debug, Clone, Copy)]
pub struct SubPageId {
    pub value: usize,
}

impl SubPageId {
    pub fn new(value: usize) -> Self {
        Self { value }
    }
    pub fn from_message_id(message_id: MessageId) -> Self {
        Self {
            value: message_id as usize / SUB_PAGE_MESSAGES_AMOUNT,
        }
    }

    pub fn get_first_message_id(&self) -> MessageId {
        let result = self.value * SUB_PAGE_MESSAGES_AMOUNT;
        result as MessageId
    }

    pub fn get_first_message_id_of_next_page(&self) -> MessageId {
        self.get_first_message_id() + SUB_PAGE_MESSAGES_AMOUNT as MessageId
    }

    pub fn iterate_through_messages(&self) -> SubPageMessagesIterator {
        SubPageMessagesIterator::new(self)
    }
}

#[cfg(test)]
mod test {
    use crate::sub_page::*;

    #[test]
    fn test_first_message_id() {
        assert_eq!(0, SubPageId::new(0).get_first_message_id());
        assert_eq!(1000, SubPageId::new(1).get_first_message_id());
        assert_eq!(2000, SubPageId::new(2).get_first_message_id());
    }

    #[test]
    fn test_first_message_id_of_next_page() {
        assert_eq!(1000, SubPageId::new(0).get_first_message_id_of_next_page());
        assert_eq!(2000, SubPageId::new(1).get_first_message_id_of_next_page());
        assert_eq!(3000, SubPageId::new(2).get_first_message_id_of_next_page());
    }
}
