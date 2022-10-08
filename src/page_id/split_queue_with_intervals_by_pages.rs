use my_service_bus_abstractions::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

use super::{get_last_message_id_of_the_page, get_page_id, PageId};

pub struct SplittedByPageId {
    pub page_id: PageId,
    pub ids: QueueWithIntervals,
}

pub struct SplittedByPageIdIterator {
    intervals: Vec<QueueIndexRange>,
    index: usize,
}

impl SplittedByPageIdIterator {
    pub fn new(src: &QueueWithIntervals) -> Self {
        Self {
            intervals: src.intervals.clone(),
            index: 0,
        }
    }
}

impl Iterator for SplittedByPageIdIterator {
    type Item = SplittedByPageId;

    fn next(&mut self) -> Option<Self::Item> {
        let el = self.intervals.get_mut(self.index)?;

        if el.is_empty() {
            return None;
        }

        let mut ids = QueueWithIntervals::new();
        let mut page_id = -1;

        while let Some(el) = self.intervals.get_mut(self.index) {
            if page_id < 0 {
                page_id = get_page_id(el.from_id);
            }

            let from_page_id = get_page_id(el.from_id);

            if from_page_id > page_id {
                return Some(SplittedByPageId { page_id, ids });
            }

            let to_page_id = get_page_id(el.to_id);

            if to_page_id > page_id {
                let to_id = get_last_message_id_of_the_page(page_id);

                ids.intervals.push(QueueIndexRange {
                    from_id: el.from_id,
                    to_id,
                });

                el.from_id = to_id + 1;

                return Some(SplittedByPageId { page_id, ids });
            }

            ids.intervals.push(QueueIndexRange {
                from_id: el.from_id,
                to_id: el.to_id,
            });

            self.index += 1;
        }

        return Some(SplittedByPageId { page_id, ids });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_both_on_the_same_page() {
        let src = QueueWithIntervals::from_single_interval(100, 200);

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(1, result.len());
        assert_eq!(0, result[0].page_id);

        assert_eq!(100, result[0].ids.intervals[0].from_id);
        assert_eq!(200, result[0].ids.intervals[0].to_id);
    }

    #[test]
    fn test_we_are_jumping_behind_the_page() {
        let src = QueueWithIntervals::from_single_interval(99998, 100002);

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(2, result.len());
        assert_eq!(0, result[0].page_id);
        assert_eq!(1, result[1].page_id);

        assert_eq!(99998, result[0].ids.intervals[0].from_id);
        assert_eq!(99999, result[0].ids.intervals[0].to_id);

        assert_eq!(100000, result[1].ids.intervals[0].from_id);
        assert_eq!(100002, result[1].ids.intervals[0].to_id);
    }

    #[test]
    fn test_we_are_jumping_behind_the_page_2() {
        let mut src = QueueWithIntervals::from_single_interval(99_998, 100_002);

        src.intervals.push(QueueIndexRange {
            from_id: 100_010,
            to_id: 100_020,
        });

        src.intervals.push(QueueIndexRange {
            from_id: 199_990,
            to_id: 200_020,
        });

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(3, result.len());
        assert_eq!(0, result[0].page_id);
        assert_eq!(1, result[1].page_id);
        assert_eq!(2, result[2].page_id);

        assert_eq!(99_998, result[0].ids.intervals[0].from_id);
        assert_eq!(99_999, result[0].ids.intervals[0].to_id);

        assert_eq!(100_000, result[1].ids.intervals[0].from_id);
        assert_eq!(100_002, result[1].ids.intervals[0].to_id);

        assert_eq!(100_010, result[1].ids.intervals[1].from_id);
        assert_eq!(100_020, result[1].ids.intervals[1].to_id);

        assert_eq!(199_990, result[1].ids.intervals[2].from_id);
        assert_eq!(199_999, result[1].ids.intervals[2].to_id);

        assert_eq!(200_000, result[2].ids.intervals[0].from_id);
        assert_eq!(200_020, result[2].ids.intervals[0].to_id);
    }
}
