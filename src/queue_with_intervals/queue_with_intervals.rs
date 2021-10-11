use crate::{
    messages::MessageId,
    queue_with_intervals::queue_index_range::{QueueIndexRange, RemoveResult},
};

use super::iterator::QueueWithIntervalsIterator;
use crate::page_id::SplittedByPageIdIterator;

#[derive(Debug, Clone)]
pub struct QueueWithIntervals {
    pub intervals: Vec<QueueIndexRange>,
}

impl QueueWithIntervals {
    pub fn new() -> QueueWithIntervals {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn restore(intervals: Vec<QueueIndexRange>) -> Self {
        return Self { intervals };
    }

    pub fn from_single_interval(from_id: MessageId, to_id: MessageId) -> Self {
        let mut result = Self {
            intervals: Vec::new(),
        };

        result.intervals.push(QueueIndexRange { from_id, to_id });

        result
    }

    pub fn reset(&mut self, intervals: Vec<QueueIndexRange>) {
        self.intervals.clear();
        self.intervals.extend(intervals)
    }

    pub fn remove(&mut self, id: MessageId) -> bool {
        if self.intervals.len() == 0 {
            println!("We are trying to remove {} but queue is empty #1", id);
            return false;
        }

        for index in 0..self.intervals.len() {
            let item = self.intervals.get_mut(index).unwrap();

            if item.is_my_interval_to_remove(id) {
                let remove_result = item.remove(id);

                match remove_result {
                    RemoveResult::NoUpdate => {}
                    RemoveResult::InsertNew(new_item) => {
                        self.intervals.insert(index + 1, new_item);
                    }
                    RemoveResult::RemoveItem => {
                        self.intervals.remove(index);
                    }
                }

                return true;
            }
        }

        return false;
    }

    pub fn enqueue(&mut self, message_id: MessageId) {
        if self.intervals.len() == 0 {
            let item = QueueIndexRange::new_with_single_value(message_id);
            self.intervals.push(item);
            return;
        }

        let mut found_index = None;

        for index in 0..self.intervals.len() {
            let el = self.intervals.get_mut(index).unwrap();

            if el.try_join(message_id) {
                found_index = Some(index);
                break;
            }

            if message_id < el.from_id - 1 {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.intervals.insert(index, item);
                found_index = Some(index);
                break;
            }
        }

        match found_index {
            Some(index_we_handeled) => {
                if index_we_handeled > 0 {
                    let current_el = self.intervals.get(index_we_handeled).unwrap().clone();
                    let before_el = self.intervals.get_mut(index_we_handeled - 1).unwrap();
                    if before_el.try_join_with_the_next_one(current_el) {
                        self.intervals.remove(index_we_handeled);
                    }
                }

                if index_we_handeled < self.intervals.len() - 1 {
                    let after_el = self.intervals.get(index_we_handeled + 1).unwrap().clone();

                    let current_el = self.intervals.get_mut(index_we_handeled).unwrap();
                    if current_el.try_join_with_the_next_one(after_el) {
                        self.intervals.remove(index_we_handeled + 1);
                    }
                }
            }
            None => {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.intervals.push(item);
                return;
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<MessageId> {
        let first_interval = self.intervals.get_mut(0)?;

        let result = first_interval.dequeue();

        if first_interval.is_empty() && self.intervals.len() > 1 {
            self.intervals.remove(0);
        }

        result
    }

    pub fn peek(&self) -> Option<MessageId> {
        let first_interval = self.intervals.get(0)?;

        first_interval.peek()
    }

    pub fn len(&self) -> i64 {
        let mut result = 0i64;

        for i in 0..self.intervals.len() {
            result = result + self.intervals[i].len();
        }

        result
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        self.intervals.clone()
    }

    pub fn get_min_id(&self) -> Option<MessageId> {
        if self.len() == 0 {
            return None;
        }

        let result = self.intervals.get(0)?;

        Some(result.from_id)
    }

    pub fn get_max_id(&self) -> Option<MessageId> {
        if self.len() == 0 {
            return None;
        }

        let result = self.intervals.get(self.intervals.len() - 1)?;

        Some(result.to_id)
    }

    pub fn split_by_page_id(&self) -> SplittedByPageIdIterator {
        SplittedByPageIdIterator::new(self)
    }
}

impl IntoIterator for QueueWithIntervals {
    type Item = i64;

    type IntoIter = QueueWithIntervalsIterator;

    fn into_iter(self) -> QueueWithIntervalsIterator {
        QueueWithIntervalsIterator::new(self.clone())
    }
}

impl<'s> IntoIterator for &'s QueueWithIntervals {
    type Item = i64;

    type IntoIter = QueueWithIntervalsIterator;

    fn into_iter(self) -> QueueWithIntervalsIterator {
        QueueWithIntervalsIterator::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let queue = QueueWithIntervals::new();

        assert_eq!(true, queue.get_min_id().is_none());
        assert_eq!(0, queue.len());
    }

    #[test]
    fn test_enqueue_and_dequeue() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(5);
        queue.enqueue(6);

        assert_eq!(2, queue.len());

        assert_eq!(5, queue.dequeue().unwrap());
        assert_eq!(6, queue.dequeue().unwrap());
        assert_eq!(true, queue.dequeue().is_none());
    }

    #[test]
    fn test_merge_intervals_at_the_end() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);

        assert_eq!(1, queue.intervals.len());

        queue.enqueue(203);

        assert_eq!(2, queue.intervals.len());

        queue.enqueue(202);
        assert_eq!(1, queue.intervals.len());
    }

    #[test]
    fn test_remove_first_element() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);

        queue.remove(200);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(201, queue.intervals[0].from_id);
        assert_eq!(204, queue.intervals[0].to_id);
    }

    #[test]
    fn test_remove_last_element() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);

        queue.remove(204);

        println!("Len: {}", queue.intervals.len());

        assert_eq!(1, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(203, queue.intervals[0].to_id);
    }

    #[test]
    fn test_remove_middle_element_and_separate() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);

        queue.remove(202);

        println!("Len: {}", queue.intervals.len());

        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);
    }

    #[test]
    fn test_remove_middle_element_and_empty_it() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);
        queue.enqueue(205);
        queue.enqueue(206);

        queue.remove(202);
        assert_eq!(2, queue.intervals.len());

        queue.remove(205);
        assert_eq!(3, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);

        assert_eq!(206, queue.intervals[2].from_id);
        assert_eq!(206, queue.intervals[2].to_id);

        queue.remove(203);
        queue.remove(204);
        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(206, queue.intervals[1].from_id);
        assert_eq!(206, queue.intervals[1].to_id);
    }

    #[test]
    fn test_remove_element_and_empty_last_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);
        queue.enqueue(205);
        queue.enqueue(206);

        queue.remove(202);
        assert_eq!(2, queue.intervals.len());

        queue.remove(205);
        assert_eq!(3, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);

        assert_eq!(206, queue.intervals[2].from_id);
        assert_eq!(206, queue.intervals[2].to_id);

        queue.remove(206);
        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);
    }

    #[test]
    fn one_insert_one_remove_len_shoud_be_0() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(20466);

        let result = queue.dequeue();

        assert_eq!(20466, result.unwrap());
        assert_eq!(0, queue.len());

        let result = queue.dequeue();

        assert_eq!(true, result.is_none());

        assert_eq!(0, queue.len());
    }

    #[test]
    fn test_if_we_push_intervals_randomly_but_as_one_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(502);
        queue.enqueue(503);
        queue.enqueue(504);

        queue.enqueue(508);
        assert_eq!(queue.intervals.len(), 2);

        queue.enqueue(506);
        assert_eq!(queue.intervals.len(), 3);
        queue.enqueue(507);
        assert_eq!(queue.intervals.len(), 2);
    }
}
