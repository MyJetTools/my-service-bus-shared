use my_service_bus_abstractions::MessageId;

pub const MESSAGES_IN_PAGE: i64 = 100_000;

pub type PageId = i64;

pub fn get_page_id(message_id: MessageId) -> PageId {
    message_id / MESSAGES_IN_PAGE
}

pub fn get_last_message_id_of_the_page(page_id: PageId) -> MessageId {
    (page_id + 1) * MESSAGES_IN_PAGE - 1
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_message_id_of_the_page() {
        assert_eq!(099_999, get_last_message_id_of_the_page(0));

        assert_eq!(199_999, get_last_message_id_of_the_page(1));

        assert_eq!(299_999, get_last_message_id_of_the_page(2));
    }
}
