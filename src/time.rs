use chrono::{prelude::*, Duration};

pub trait PrettyPrint {
    fn pretty_string(&self) -> String;
}

impl PrettyPrint for DateTime<Local> {
    fn pretty_string(&self) -> String {
        self.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[derive(Debug)]
pub struct DateOffset {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl DateOffset {
    pub fn new(offset: Duration) -> Self {
        let now = Local::now();
        let start = now - offset;
        Self { start, end: now }
    }

    pub fn normalize_date(date: DateTime<Local>) -> Result<DateTime<Local>, std::io::Error> {
        let midnight = chrono::naive::NaiveTime::from_hms_opt(0, 0, 0);
        Ok(date.with_time(midnight.unwrap()).unwrap())
    }

    /// Normalize the start and end dates to midnight
    pub fn normalize(&self) -> Self {
        DateOffset {
            start: DateOffset::normalize_date(self.start).unwrap(),
            end: self.end,
        }
    }

    pub fn to_strings(&self) -> (String, String) {
        (self.start.to_rfc3339(), self.end.to_rfc3339())
    }

    pub fn pretty_strings(&self) -> (String, String) {
        (self.start.pretty_string(), self.end.pretty_string())
    }
}
