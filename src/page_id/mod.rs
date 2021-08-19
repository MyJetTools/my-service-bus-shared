mod page_id;
mod split_queue_with_intervals_by_pages;

pub use page_id::get_last_message_id_of_the_page;
pub use page_id::get_page_id;
pub use page_id::PageId;
pub use page_id::MESSAGES_IN_PAGE;

pub use split_queue_with_intervals_by_pages::SplittedByPageId;
pub use split_queue_with_intervals_by_pages::SplittedByPageIdIterator;
