pub struct MessageId(i64);

impl MessageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }
}

impl AsRef<i64> for MessageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}
