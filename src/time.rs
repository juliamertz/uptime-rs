use chrono::{prelude::*, Duration};

pub struct DateOffset {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateOffset {
    pub fn new(offset: Duration) -> Self {
        let now = Utc::now();
        let start = now - offset;
        Self { start, end: now }
    }

    pub fn to_strings(&self) -> (String, String) {
        (self.start.to_rfc3339(), self.end.to_rfc3339())
    }
}
