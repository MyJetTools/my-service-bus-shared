use my_service_bus_abstractions::MessageId;

use crate::sub_page::SubPageId;

use super::SubPagesIterator;

pub const MESSAGES_IN_PAGE: i64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct PageId(i64);

impl PageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }
    pub fn from_message_id(message_id: MessageId) -> Self {
        Self(message_id / MESSAGES_IN_PAGE)
    }

    pub fn get_first_message_id(&self) -> MessageId {
        self.0 * MESSAGES_IN_PAGE
    }

    pub fn get_last_message_id(&self) -> MessageId {
        (self.0 + 1) * MESSAGES_IN_PAGE - 1
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn iterate_messages(&self) -> std::ops::Range<i64> {
        let first_message_id = self.get_first_message_id();
        first_message_id..first_message_id + MESSAGES_IN_PAGE
    }

    pub fn iterate_sub_page_ids(&self) -> SubPagesIterator {
        let message_id = self.get_first_message_id();
        let first_sub_page_id = SubPageId::from_message_id(message_id);

        SubPagesIterator {
            now_id: first_sub_page_id.get_value(),
            to_id: first_sub_page_id.get_value() + crate::sub_page::SUB_PAGES_PER_PAGE,
        }
    }
}

impl AsRef<i64> for PageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl std::ops::Sub<i64> for PageId {
    type Output = Self;

    fn sub(self, rhs: i64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl std::ops::Add<i64> for PageId {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_message_id_of_the_page() {
        assert_eq!(099_999, PageId(0).get_last_message_id());

        assert_eq!(199_999, PageId(1).get_last_message_id());

        assert_eq!(299_999, PageId(2).get_last_message_id());
    }
}
