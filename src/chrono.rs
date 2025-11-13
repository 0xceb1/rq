// Wrappers for kdb/q temporal data structures
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use std::cmp::Ordering;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    days: i32, // Epoch: 2000-01-01 = 0
}

impl Date {
    //TODO: in q, there're actually two special values of date: 0000.00.00 (stands for all values out of range), and 0Wd (infinite)
    // We use `assert!` to handle these cases for now. These special values will be added later.
    const MAX_DAYS: i32 = 2921939;
    const MIN_DAYS: i32 = -730119;
    pub const MAX: Date = Date {
        days: Date::MAX_DAYS,
    }; // 9999.12.31
    pub const MIN: Date = Date {
        days: Date::MIN_DAYS,
    }; // 0001.01.01
    const EPOCH: NaiveDate = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();

    /// Creates a Date from a literal string in format "YYYY.MM.DD"
    pub fn from_literal(literal: &str) -> Result<Self, String> {
        let date =
            NaiveDate::parse_from_str(literal, "%Y.%m.%d").map_err(|_| format!("'{literal}"))?;

        let days = date.signed_duration_since(Date::EPOCH).num_days() as i32;

        assert!((Date::MIN_DAYS..Date::MAX_DAYS).contains(&days));
        Ok(Date { days })
    }

    /// Converts the Date to a literal string in format "YYYY.MM.DD"
    pub fn to_literal(self) -> String {
        let date = self.to_naive_date();
        format!("{:04}.{:02}.{:02}", date.year(), date.month(), date.day())
    }

    pub fn year(&self) -> i32 {
        self.to_naive_date().year()
    }

    pub fn mm(&self) -> i32 {
        self.to_naive_date().month() as i32
    }

    pub fn dd(&self) -> i32 {
        self.to_naive_date().day() as i32
    }

    pub fn week(&self) -> Date {
        let date = self.to_naive_date();
        let mon = date - Duration::days(date.weekday().num_days_from_monday() as i64);
        Date::from_naive_date(mon)
    }

    pub fn from_i32(days: i32) -> Self {
        assert!((Date::MIN_DAYS..Date::MAX_DAYS).contains(&days));
        Date { days }
    }

    pub fn to_i32(self) -> i32 {
        self.days
    }

    // Helper methods
    fn to_naive_date(self) -> NaiveDate {
        Date::EPOCH + Duration::days(self.days as i64)
    }

    fn from_naive_date(date: NaiveDate) -> Self {
        let days = (date - Date::EPOCH).num_days() as i32;
        Date::from_i32(days)
    }
}

impl From<i32> for Date {
    fn from(days: i32) -> Self {
        assert!((Date::MIN_DAYS..Date::MAX_DAYS).contains(&days));
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

impl Add<i32> for Date {
    type Output = Date;

    fn add(self, rhs: i32) -> Date {
        Date {
            days: self.to_i32() + rhs,
        }
    }
}

impl Add<Date> for i32 {
    type Output = Date;

    fn add(self, rhs: Date) -> Date {
        Date {
            days: self + rhs.to_i32(),
        }
    }
}

impl Sub<i32> for Date {
    type Output = Date;

    fn sub(self, rhs: i32) -> Date {
        Date {
            days: self.to_i32() - rhs,
        }
    }
}

impl Sub<Date> for i32 {
    type Output = Date;

    fn sub(self, rhs: Date) -> Date {
        Date {
            days: self - rhs.to_i32(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    nanoseconds: i64, // Epoch: 2000.01.01D00:00:00.000000000
}

impl Timestamp {
    // WARN: The range of q timestamp type is from 1707.09.22D00:12:43.145224194 to 2292.04.10D23:47:16.854775806
    // This is because q define inf = i64::MAX, and -inf = -i64::MAX
    const MIN_NANO: i64 = -i64::MAX + 1;
    const MAX_NANO: i64 = i64::MAX - 1;
    pub const MIN: Timestamp = Timestamp {
        nanoseconds: Timestamp::MIN_NANO,
    };
    pub const MAX: Timestamp = Timestamp {
        nanoseconds: Timestamp::MAX_NANO,
    };
    const EPOCH: NaiveDateTime = NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    const MIN_NAIVE_DATE_TIME: NaiveDateTime = NaiveDate::from_ymd_opt(1707, 9, 22)
        .unwrap()
        .and_hms_nano_opt(0, 12, 43, 145224194)
        .unwrap();
    const MAX_NAIVE_DATE_TIME: NaiveDateTime = NaiveDate::from_ymd_opt(2292, 4, 10)
        .unwrap()
        .and_hms_nano_opt(23, 47, 16, 854775806)
        .unwrap();

    fn from_literal(literal: &str) -> Result<Self, String> {
        let dt = NaiveDateTime::parse_from_str(literal, "%Y.%m.%dD%H:%M:%S%.9f")
            .map_err(|_| format!("'{literal}"))?;

        let nanoseconds = dt
            .signed_duration_since(Timestamp::EPOCH)
            .num_nanoseconds()
            .unwrap();

        assert!((Timestamp::MIN_NANO..Timestamp::MAX_NANO).contains(&nanoseconds));
        Ok(Timestamp { nanoseconds })
    }

    pub fn to_literal(self) -> String {
        let dt = self.to_naive_date_time();
        format!(
            "{:04}.{:02}.{:02}D{:02}:{:02}:{:02}.{:09}",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.nanosecond()
        )
    }

    pub fn to_i64(self) -> i64 {
        self.nanoseconds
    }

    pub fn from_i64(nanoseconds: i64) -> Self {
        Timestamp { nanoseconds }
    }

    pub fn year(&self) -> i32 {
        self.to_naive_date_time().year()
    }

    pub fn mm(&self) -> i32 {
        self.to_naive_date_time().month() as i32
    }

    pub fn dd(&self) -> i32 {
        self.to_naive_date_time().day() as i32
    }

    pub fn week(&self) -> Date {
        let dt = self.to_naive_date_time();
        let mon = dt.date() - Duration::days(dt.weekday().num_days_from_monday() as i64);
        Date::from_naive_date(mon)
    }

    pub fn hh(&self) -> i32 {
        self.to_naive_date_time().hour() as i32
    }

    pub fn uu(&self) -> i32 {
        self.to_naive_date_time().minute() as i32
    }

    pub fn ss(&self) -> i32 {
        self.to_naive_date_time().second() as i32
    }

    // Helper methods
    fn to_naive_date_time(self) -> NaiveDateTime {
        Timestamp::EPOCH + Duration::nanoseconds(self.nanoseconds)
    }
}

impl From<i64> for Timestamp {
    fn from(nanoseconds: i64) -> Self {
        assert!((Timestamp::MIN_NANO..Timestamp::MAX_NANO).contains(&nanoseconds));
        Timestamp { nanoseconds }
    }
}

impl From<Timestamp> for i64 {
    fn from(ts: Timestamp) -> Self {
        ts.nanoseconds
    }
}

impl PartialEq<i64> for Timestamp {
    fn eq(&self, other: &i64) -> bool {
        self.nanoseconds == *other
    }
}

impl PartialEq<Timestamp> for i64 {
    fn eq(&self, other: &Timestamp) -> bool {
        *self == other.nanoseconds
    }
}

impl PartialOrd<i64> for Timestamp {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        self.nanoseconds.partial_cmp(other)
    }
}

impl PartialOrd<Timestamp> for i64 {
    fn partial_cmp(&self, other: &Timestamp) -> Option<Ordering> {
        self.partial_cmp(&other.nanoseconds)
    }
}

impl Add<i64> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: i64) -> Timestamp {
        Timestamp {
            nanoseconds: self.to_i64() + rhs,
        }
    }
}

impl Add<Timestamp> for i64 {
    type Output = Timestamp;

    fn add(self, rhs: Timestamp) -> Timestamp {
        Timestamp {
            nanoseconds: self + rhs.to_i64(),
        }
    }
}

impl Sub<i64> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: i64) -> Timestamp {
        Timestamp {
            nanoseconds: self.to_i64() - rhs,
        }
    }
}

impl Sub<Timestamp> for i64 {
    type Output = Timestamp;

    fn sub(self, rhs: Timestamp) -> Timestamp {
        Timestamp {
            nanoseconds: self - rhs.to_i64(),
        }
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_literal())
    }
}
pub struct Month {}

pub struct Day {}
