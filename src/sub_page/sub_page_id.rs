use crate::{page_id::PageId, MessageId};

use super::SubPageMessagesIterator;

pub const SUB_PAGE_MESSAGES_AMOUNT: usize = 1000;
pub const SUB_PAGES_PER_PAGE: usize = 100;

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

    pub fn from_page_id(page_id: PageId) -> Self {
        Self {
            value: page_id as usize * SUB_PAGES_PER_PAGE,
        }
    }

    pub fn get_first_message_id(&self) -> MessageId {
        let result = self.value * SUB_PAGE_MESSAGES_AMOUNT;
        result as MessageId
    }

    pub fn get_first_message_id_of_next_sub_page(&self) -> MessageId {
        self.get_first_message_id() + SUB_PAGE_MESSAGES_AMOUNT as MessageId
    }

    pub fn iterate_through_messages(&self) -> SubPageMessagesIterator {
        SubPageMessagesIterator::new(self)
    }
}

#[cfg(test)]
mod test {
    use crate::{page_id::get_page_id, sub_page::*};

    #[test]
    fn test_first_message_id() {
        assert_eq!(0, SubPageId::new(0).get_first_message_id());
        assert_eq!(1000, SubPageId::new(1).get_first_message_id());
        assert_eq!(2000, SubPageId::new(2).get_first_message_id());
    }

    #[test]
    fn test_first_message_id_of_next_page() {
        assert_eq!(
            1000,
            SubPageId::new(0).get_first_message_id_of_next_sub_page()
        );
        assert_eq!(
            2000,
            SubPageId::new(1).get_first_message_id_of_next_sub_page()
        );
        assert_eq!(
            3000,
            SubPageId::new(2).get_first_message_id_of_next_sub_page()
        );
    }

    #[test]
    fn test_creating_from_page_id() {
        assert_eq!(0, SubPageId::from_page_id(0).value);

        assert_eq!(100, SubPageId::from_page_id(1).value);
        assert_eq!(200, SubPageId::from_page_id(2).value);

        //Made cross check from MessageID and From PageID
        let message_id = 100_000;
        let page_id = get_page_id(message_id);

        assert_eq!(
            SubPageId::from_page_id(page_id).value,
            SubPageId::from_message_id(message_id).value
        );
    }
}
