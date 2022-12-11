use my_service_bus_abstractions::MessageId;

use crate::page_id::PageId;

pub const SUB_PAGE_MESSAGES_AMOUNT: i64 = 1000;
pub const SUB_PAGES_PER_PAGE: i64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct SubPageId(i64);

impl SubPageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }
    pub fn from_message_id(message_id: MessageId) -> Self {
        Self(message_id / SUB_PAGE_MESSAGES_AMOUNT)
    }

    pub fn from_page_id(page_id: PageId) -> Self {
        Self(page_id.get_value() * SUB_PAGES_PER_PAGE)
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn get_first_message_id(&self) -> MessageId {
        self.get_value() * SUB_PAGE_MESSAGES_AMOUNT
    }

    pub fn get_first_message_id_of_next_sub_page(&self) -> MessageId {
        self.get_first_message_id() + SUB_PAGE_MESSAGES_AMOUNT as MessageId
    }

    pub fn iterate_message_ids(&self) -> std::ops::Range<i64> {
        let first_message_id = self.get_first_message_id();
        first_message_id..first_message_id + SUB_PAGE_MESSAGES_AMOUNT
    }
}

impl AsRef<i64> for SubPageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::{page_id::PageId, sub_page::*};

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
        assert_eq!(0, SubPageId::from_page_id(PageId::new(0)).get_value());

        assert_eq!(100, SubPageId::from_page_id(PageId::new(1)).get_value());
        assert_eq!(200, SubPageId::from_page_id(PageId::new(2)).get_value());

        //Made cross check from MessageID and From PageID
        let message_id = 100_000;
        let page_id = PageId::from_message_id(message_id);

        assert_eq!(
            SubPageId::from_page_id(page_id).get_value(),
            SubPageId::from_message_id(message_id).get_value()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::SubPageId;

    #[test]
    fn test_b_tree_map() {
        let mut map = std::collections::BTreeMap::new();

        map.insert(SubPageId::new(1), 1);
        map.insert(SubPageId::new(2), 2);
        map.insert(SubPageId::new(4), 4);
        map.insert(SubPageId::new(3), 3);

        assert_eq!(1, map[&SubPageId::new(1)]);
        assert_eq!(2, map[&SubPageId::new(2)]);
        assert_eq!(3, map[&SubPageId::new(3)]);
        assert_eq!(4, map[&SubPageId::new(4)]);
    }
}
