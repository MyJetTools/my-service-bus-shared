#[derive(Clone, Debug)]
pub struct LockItem {
    pub id: i64,
    pub data: String,
    pub date: i64,
}

pub struct Locks {
    id: i64,
    data: Vec<LockItem>,
}

impl Locks {
    pub fn new() -> Self {
        Self {
            id: 0,
            data: Vec::new(),
        }
    }

    fn get_new_id(&mut self) -> i64 {
        self.id += 1;
        self.id
    }

    pub fn new_lock(&mut self, data: String) -> i64 {
        let id = self.get_new_id();

        self.data.push(LockItem {
            id,
            data,
            date: crate::date_time::DateTimeAsMicroseconds::now().unix_microseconds,
        });

        id
    }

    pub fn remove(&mut self, id: i64) {
        let index = self.data.iter().position(|itm| itm.id == id);
        if let Some(index) = index {
            self.data.remove(index);
        }
    }

    pub fn get_all(&self) -> Vec<LockItem> {
        let mut result = Vec::new();

        for item in &self.data {
            result.push(item.clone());
        }

        result
    }
}
