use crate::{messages::MessageId, queue_with_intervals::queue_index_range::QueueIndexRange};

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

    fn get_index_to_insert(&self, message_id: MessageId) -> usize {
        for i in 0..self.intervals.len() {
            if self.intervals[i].is_before(message_id) {
                return i;
            }
        }

        return self.intervals.len();
    }

    fn get_my_interval_index(&self, message_id: MessageId) -> Option<usize> {
        for i in 0..self.intervals.len() {
            let interval = &self.intervals[i];
            if interval.is_my_interval(message_id) {
                return Some(i);
            }
        }
        return None;
    }

    fn get_interval_index(&mut self, message_id: MessageId) -> usize {
        match self.get_my_interval_index(message_id) {
            Some(index) => return index,
            None => {
                let index = self.get_index_to_insert(message_id);

                let new_index_range = QueueIndexRange::new(0);
                if index >= self.intervals.len() {
                    self.intervals.push(new_index_range)
                } else {
                    self.intervals.insert(index, new_index_range);
                }
                return index;
            }
        }
    }

    pub fn remove(&mut self, message_id: MessageId) -> bool {
        let found_interval = self.get_my_interval_index(message_id);

        if let Some(index) = found_interval {
            let new_item = self.intervals[index].remove(message_id);

            if let Some(new_item) = new_item {
                self.intervals.insert(index + 1, new_item);
            }

            if self.len() > 1 {
                if self.intervals[index].is_empty() {
                    self.intervals.remove(index);
                }
            }

            return true;
        }

        return false;
    }

    pub fn enqueue(&mut self, message_id: MessageId) {
        let index = self.get_interval_index(message_id);

        self.intervals[index].enqueue(message_id);

        if index > 0 {
            let element = self.intervals[index].clone();
            if self.intervals[index - 1].try_merge_next(&element) {
                self.intervals.remove(index);
            }
        } else {
            if self.intervals.len() > 1 {
                let next_element = self.intervals[index + 1].clone();
                if self.intervals[index].try_merge_next(&next_element) {
                    self.intervals.remove(index + 1);
                }
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
}
