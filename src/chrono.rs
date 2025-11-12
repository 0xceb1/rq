// Wrappers for kdb/q temporal data structures
use chrono::{Datelike, Duration, Months, NaiveDate};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    days: i32, // Epoch: 2000-01-01 = 0
}

impl Date {
    //TODO: in q, there're actually two special values of date: 0000.00.00 (stands for all values out of range), and 0Wd (infinite)
    // We use `assert!` to handle these cases for now. These special values will be added later.
    const MAX_DAYS: i32 = 2921939;
    const MIN_DAYS: i32 = -730119;
    pub const MAX: Date = Date::new(self.MAX_DAYS); // 9999.12.31
    pub const MIN: Date = Date::new(self.MIN_DAYS); // 0001.01.01
    const EPOCH: NaiveDate = NaiveDate::from_ymd(2000, 1, 1);

    /// Creates a new Date from days since 2000-01-01
    pub fn new(days: i32) -> Self {
        assert!(days >= self.MIN_DAYS && days <= self.MAX_DAYS);
        Date { days }
    }

    /// Creates a Date from a literal string in format "YYYY.MM.DD"
    pub fn from_literal(literal: &str) -> Result<Self, String> {
        let date =
            NaiveDate::parse_from_str(literal, "%Y.%m.%d").map_err(|_| format!("'{literal}"))?;

        let days = date.signed_duration_since(self.EPOCH).num_days() as i32;

        assert!(days >= self.MIN_DAYS && days <= self.MAX_DAYS);
        Ok(Date { days })
    }

    /// Converts the Date to a literal string in format "YYYY.MM.DD"
    pub fn to_literal(&self) -> String {
        let date = self.to_naive_date();
        format!("{:04}.{:02}.{:02}", date.year(), date.month(), date.day())
    }

    /// Adds months to the date
    pub fn mm(&self) -> i32 {
        self.to_naive_date().month() as i32
    }

    /// Adds days to the date
    pub fn dd(&self) -> i32 {
        self.to_naive_date().day() as i32
    }

    /// Converts from i32
    pub fn from_i32(days: i32) -> Self {
        assert!(days >= self.MIN_DAYS && days <= self.MAX_DAYS);
        Date { days }
    }

    /// Converts to i32
    pub fn to_i32(&self) -> i32 {
        self.days
    }

    // Helper methods
    fn to_naive_date(&self) -> NaiveDate {
        self.EPOCH + Duration::days(self.days as i64)
    }
}

impl From<i32> for Date {
    fn from(days: i32) -> Self {
        assert!(days >= self.MIN_DAYS && days <= self.MAX_DAYS);
        Date { days }
    }
}

impl From<Date> for i32 {
    fn from(date: Date) -> Self {
        date.days
    }
}

impl PartialEq<i32> for Date {
    fn eq(&self, other: &i32) -> bool {
        self.days == *other
    }
}

impl PartialEq<Date> for i32 {
    fn eq(&self, other: &Date) -> bool {
        *self == other.days
    }
}

impl PartialOrd<i32> for Date {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.days.partial_cmp(other)
    }
}

impl PartialOrd<Date> for i32 {
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        self.partial_cmp(&other.days)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_literal())
    }
}

pub struct Timestamp {}

pub struct Month {}

pub struct Day {}
