// Wrappers for kdb/q temporal data structures
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use regex::Regex;
use std::cmp::Ordering;
use std::ops::{Add, Sub};
use std::sync::LazyLock;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    months: i32, // Epoch: 2000.01 = 0
}

impl Month {
    const MAX_MONTHS: i32 = 95999; // 9999.12
    const MIN_MONTHS: i32 = -23988; // 0001.01
    pub const MAX: Month = Month {
        months: Month::MAX_MONTHS,
    }; // 9999.12
    pub const MIN: Month = Month {
        months: Month::MIN_MONTHS,
    }; // 0001.01

    /// Creates a Month from a literal string in format "YYYY.MMm"
    pub fn from_literal(literal: &str) -> Result<Self, String> {
        // Expected format: "YYYY.MMm" (exactly 8 characters)
        if literal.len() != 8 || !literal.ends_with('m') || literal.as_bytes()[4] != b'.' {
            return Err(format!("'{literal}"));
        }

        let year: u32 = literal[0..4].parse().map_err(|_| format!("'{literal}"))?;
        let month: i32 = literal[5..7].parse().map_err(|_| format!("'{literal}"))?;

        if !(1..=12).contains(&month) {
            return Err(format!("'{literal}"));
        }

        let months = (year as i32 - 2000) * 12 + (month - 1);

        assert!((Month::MIN_MONTHS..=Month::MAX_MONTHS).contains(&months));
        Ok(Month { months })
    }

    /// Converts the Month to a literal string in format "YYYY.MMm"
    pub fn to_literal(self) -> String {
        let total_months = self.months + (2000 * 12); // months since year 0
        let year = total_months / 12;
        let month = (total_months % 12) + 1;
        format!("{:04}.{:02}m", year, month)
    }

    pub fn year(&self) -> i32 {
        let total_months = self.months + (2000 * 12);
        total_months / 12
    }

    pub fn mm(&self) -> i32 {
        let total_months = self.months + (2000 * 12);
        (total_months % 12) + 1
    }

    pub fn from_i32(months: i32) -> Self {
        assert!((Month::MIN_MONTHS..=Month::MAX_MONTHS).contains(&months));
        Month { months }
    }

    pub fn to_i32(self) -> i32 {
        self.months
    }
}

impl From<i32> for Month {
    fn from(months: i32) -> Self {
        assert!((Month::MIN_MONTHS..=Month::MAX_MONTHS).contains(&months));
        Month { months }
    }
}

impl From<Month> for i32 {
    fn from(month: Month) -> Self {
        month.months
    }
}

impl PartialEq<i32> for Month {
    fn eq(&self, other: &i32) -> bool {
        self.months == *other
    }
}

impl PartialEq<Month> for i32 {
    fn eq(&self, other: &Month) -> bool {
        *self == other.months
    }
}

impl PartialOrd<i32> for Month {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.months.partial_cmp(other)
    }
}

impl PartialOrd<Month> for i32 {
    fn partial_cmp(&self, other: &Month) -> Option<Ordering> {
        self.partial_cmp(&other.months)
    }
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_literal())
    }
}

impl Add<i32> for Month {
    type Output = Month;

    fn add(self, rhs: i32) -> Month {
        Month {
            months: self.to_i32() + rhs,
        }
    }
}

impl Add<Month> for i32 {
    type Output = Month;

    fn add(self, rhs: Month) -> Month {
        Month {
            months: self + rhs.to_i32(),
        }
    }
}

impl Sub<i32> for Month {
    type Output = Month;

    fn sub(self, rhs: i32) -> Month {
        Month {
            months: self.to_i32() - rhs,
        }
    }
}

impl Sub<Month> for i32 {
    type Output = Month;

    fn sub(self, rhs: Month) -> Month {
        Month {
            months: self - rhs.to_i32(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timespan {
    nanoseconds: i64,
}

static TIMESPAN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(-?\d+)D(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,9}))?$").unwrap());

impl Timespan {
    const MIN_NANO: i64 = -i64::MAX + 1;
    const MAX_NANO: i64 = i64::MAX - 1;
    pub const MIN: Timespan = Timespan {
        nanoseconds: Timespan::MIN_NANO,
    };
    pub const MAX: Timespan = Timespan {
        nanoseconds: Timespan::MAX_NANO,
    };

    pub fn from_literal(literal: &str) -> Result<Self, String> {
        let caps = TIMESPAN_RE
            .captures(literal)
            .ok_or_else(|| format!("'{literal}"))?;

        let days: i64 = caps[1].parse().map_err(|_| format!("'{literal}"))?;
        let hours: i64 = caps[2].parse().map_err(|_| format!("'{literal}"))?;
        let minutes: i64 = caps[3].parse().map_err(|_| format!("'{literal}"))?;
        let seconds: i64 = caps[4].parse().map_err(|_| format!("'{literal}"))?;

        let nanos: i64 = caps
            .get(5)
            .map(|m| {
                let nanos_str = format!("{:0<9}", m.as_str());
                nanos_str[0..9].parse().unwrap_or(0)
            })
            .unwrap_or(0);

        let nanoseconds = days * 86400 * 1_000_000_000
            + hours * 3600 * 1_000_000_000
            + minutes * 60 * 1_000_000_000
            + seconds * 1_000_000_000
            + nanos;

        assert!((Timespan::MIN_NANO..=Timespan::MAX_NANO).contains(&nanoseconds));
        Ok(Timespan { nanoseconds })
    }

    /// Converts the Timespan to a literal string in format "DDxDHH:MM:SS.nnnnnnnnn"
    pub fn to_literal(self) -> String {
        let is_negative = self.nanoseconds < 0;
        let abs_nanos = self.nanoseconds.abs();

        let days = abs_nanos / (86400 * 1_000_000_000);
        let remainder = abs_nanos % (86400 * 1_000_000_000);

        let hours = remainder / (3600 * 1_000_000_000);
        let remainder = remainder % (3600 * 1_000_000_000);

        let minutes = remainder / (60 * 1_000_000_000);
        let remainder = remainder % (60 * 1_000_000_000);

        let seconds = remainder / 1_000_000_000;
        let nanos = remainder % 1_000_000_000;

        let sign = if is_negative { "-" } else { "" };
        format!(
            "{}{}D{:02}:{:02}:{:02}.{:09}",
            sign, days, hours, minutes, seconds, nanos
        )
    }

    pub fn to_i64(self) -> i64 {
        self.nanoseconds
    }

    pub fn from_i64(nanoseconds: i64) -> Self {
        Timespan { nanoseconds }
    }

    pub fn hh(&self) -> i64 {
        self.nanoseconds / (3600 * 1_000_000_000)
    }

    pub fn mm(&self) -> i64 {
        self.nanoseconds / (60 * 1_000_000_000)
    }

    pub fn uu(&self) -> i64 {
        self.nanoseconds / (60 * 1_000_000_000)
    }

    pub fn ss(&self) -> i64 {
        self.nanoseconds / 1_000_000_000
    }
}

impl From<i64> for Timespan {
    fn from(nanoseconds: i64) -> Self {
        assert!((Timespan::MIN_NANO..=Timespan::MAX_NANO).contains(&nanoseconds));
        Timespan { nanoseconds }
    }
}

impl From<Timespan> for i64 {
    fn from(ts: Timespan) -> Self {
        ts.nanoseconds
    }
}

impl PartialEq<i64> for Timespan {
    fn eq(&self, other: &i64) -> bool {
        self.nanoseconds == *other
    }
}

impl PartialEq<Timespan> for i64 {
    fn eq(&self, other: &Timespan) -> bool {
        *self == other.nanoseconds
    }
}

impl PartialOrd<i64> for Timespan {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        self.nanoseconds.partial_cmp(other)
    }
}

impl PartialOrd<Timespan> for i64 {
    fn partial_cmp(&self, other: &Timespan) -> Option<Ordering> {
        self.partial_cmp(&other.nanoseconds)
    }
}

impl std::fmt::Display for Timespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_literal())
    }
}

impl Add<i64> for Timespan {
    type Output = Timespan;

    fn add(self, rhs: i64) -> Timespan {
        Timespan {
            nanoseconds: self.to_i64() + rhs,
        }
    }
}

impl Add<Timespan> for i64 {
    type Output = Timespan;

    fn add(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self + rhs.to_i64(),
        }
    }
}

impl Sub<i64> for Timespan {
    type Output = Timespan;

    fn sub(self, rhs: i64) -> Timespan {
        Timespan {
            nanoseconds: self.to_i64() - rhs,
        }
    }
}

impl Sub<Timespan> for i64 {
    type Output = Timespan;

    fn sub(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self - rhs.to_i64(),
        }
    }
}

impl Sub<Timespan> for Timespan {
    type Output = Timespan;

    fn sub(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self.nanoseconds - rhs.nanoseconds,
        }
    }
}

impl Add<Timespan> for Timespan {
    type Output = Timespan;

    fn add(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self.nanoseconds + rhs.nanoseconds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Minute {
    minutes: i32, // Minutes since midnight
}

impl Minute {
    const MAX_MINUTES: i32 = i32::MAX - 1;
    const MIN_MINUTES: i32 = -i32::MAX + 1;
    pub const MAX: Minute = Minute {
        minutes: Minute::MAX_MINUTES,
    };
    pub const MIN: Minute = Minute {
        minutes: Minute::MIN_MINUTES,
    };

    pub fn from_literal(literal: &str) -> Result<Self, String> {
        if literal.len() != 5 || literal.as_bytes()[2] != b':' {
            return Err(format!("'{literal}"));
        }

        let hours: i32 = literal[0..2].parse().map_err(|_| format!("'{literal}"))?;
        let mins: i32 = literal[3..5].parse().map_err(|_| format!("'{literal}"))?;

        if !(0..24).contains(&hours) || !(0..60).contains(&mins) {
            return Err(format!("'{literal}"));
        }

        let minutes = hours * 60 + mins;
        assert!((Minute::MIN_MINUTES..=Minute::MAX_MINUTES).contains(&minutes));
        Ok(Minute { minutes })
    }

    pub fn to_literal(self) -> String {
        // let total_mins = self.minutes.rem_eucuid(1440);
        let hours = self.minutes / 60;
        let mins = self.minutes % 60;
        format!("{:02}:{:02}", hours, mins)
    }

    pub fn from_i32(minutes: i32) -> Self {
        assert!((Minute::MIN_MINUTES..=Minute::MAX_MINUTES).contains(&minutes));
        Minute { minutes }
    }

    pub fn to_i32(self) -> i32 {
        self.minutes
    }
}

impl From<i32> for Minute {
    fn from(minutes: i32) -> Self {
        assert!((Minute::MIN_MINUTES..=Minute::MAX_MINUTES).contains(&minutes));
        Minute { minutes }
    }
}

impl From<Minute> for i32 {
    fn from(minute: Minute) -> Self {
        minute.minutes
    }
}

impl PartialEq<i32> for Minute {
    fn eq(&self, other: &i32) -> bool {
        self.minutes == *other
    }
}

impl PartialEq<Minute> for i32 {
    fn eq(&self, other: &Minute) -> bool {
        *self == other.minutes
    }
}

impl PartialOrd<i32> for Minute {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.minutes.partial_cmp(other)
    }
}

impl PartialOrd<Minute> for i32 {
    fn partial_cmp(&self, other: &Minute) -> Option<Ordering> {
        self.partial_cmp(&other.minutes)
    }
}

impl std::fmt::Display for Minute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_literal())
    }
}

impl Add<i32> for Minute {
    type Output = Minute;

    fn add(self, rhs: i32) -> Minute {
        Minute {
            minutes: self.to_i32() + rhs,
        }
    }
}

impl Add<Minute> for i32 {
    type Output = Minute;

    fn add(self, rhs: Minute) -> Minute {
        Minute {
            minutes: self + rhs.to_i32(),
        }
    }
}

impl Sub<i32> for Minute {
    type Output = Minute;

    fn sub(self, rhs: i32) -> Minute {
        Minute {
            minutes: self.to_i32() - rhs,
        }
    }
}

impl Sub<Minute> for i32 {
    type Output = Minute;

    fn sub(self, rhs: Minute) -> Minute {
        Minute {
            minutes: self - rhs.to_i32(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Second {
    seconds: i32, // Seconds since midnight
}

impl Second {
    const MAX_SECONDS: i32 = i32::MAX - 1;
    const MIN_SECONDS: i32 = -i32::MAX + 1;
    pub const MAX: Second = Second {
        seconds: Second::MAX_SECONDS,
    };
    pub const MIN: Second = Second {
        seconds: Second::MIN_SECONDS,
    };

    pub fn from_literal(literal: &str) -> Result<Self, String> {
        if literal.len() != 8 || literal.as_bytes()[2] != b':' || literal.as_bytes()[5] != b':' {
            return Err(format!("'{literal}"));
        }

        let hours: i32 = literal[0..2].parse().map_err(|_| format!("'{literal}"))?;
        let mins: i32 = literal[3..5].parse().map_err(|_| format!("'{literal}"))?;
        let secs: i32 = literal[6..8].parse().map_err(|_| format!("'{literal}"))?;

        if !(0..24).contains(&hours) || !(0..60).contains(&mins) || !(0..60).contains(&secs) {
            return Err(format!("'{literal}"));
        }

        let seconds = hours * 3600 + mins * 60 + secs;
        assert!((Second::MIN_SECONDS..=Second::MAX_SECONDS).contains(&seconds));
        Ok(Second { seconds })
    }

    pub fn to_literal(self) -> String {
        let total_secs = self.seconds.rem_euclid(86400);
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }

    pub fn from_i32(seconds: i32) -> Self {
        assert!((Second::MIN_SECONDS..=Second::MAX_SECONDS).contains(&seconds));
        Second { seconds }
    }

    pub fn to_i32(self) -> i32 {
        self.seconds
    }
}

impl From<i32> for Second {
    fn from(seconds: i32) -> Self {
        assert!((Second::MIN_SECONDS..=Second::MAX_SECONDS).contains(&seconds));
        Second { seconds }
    }
}

impl From<Second> for i32 {
    fn from(second: Second) -> Self {
        second.seconds
    }
}

impl PartialEq<i32> for Second {
    fn eq(&self, other: &i32) -> bool {
        self.seconds == *other
    }
}

impl PartialEq<Second> for i32 {
    fn eq(&self, other: &Second) -> bool {
        *self == other.seconds
    }
}

impl PartialOrd<i32> for Second {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.seconds.partial_cmp(other)
    }
}

impl PartialOrd<Second> for i32 {
    fn partial_cmp(&self, other: &Second) -> Option<Ordering> {
        self.partial_cmp(&other.seconds)
    }
}

impl std::fmt::Display for Second {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_literal())
    }
}

impl Add<i32> for Second {
    type Output = Second;

    fn add(self, rhs: i32) -> Second {
        Second {
            seconds: self.to_i32() + rhs,
        }
    }
}

impl Add<Second> for i32 {
    type Output = Second;

    fn add(self, rhs: Second) -> Second {
        Second {
            seconds: self + rhs.to_i32(),
        }
    }
}

impl Sub<i32> for Second {
    type Output = Second;

    fn sub(self, rhs: i32) -> Second {
        Second {
            seconds: self.to_i32() - rhs,
        }
    }
}

impl Sub<Second> for i32 {
    type Output = Second;

    fn sub(self, rhs: Second) -> Second {
        Second {
            seconds: self - rhs.to_i32(),
        }
    }
}

// Cross-type operations between Minute and Second

impl PartialEq<Second> for Minute {
    fn eq(&self, other: &Second) -> bool {
        self.minutes * 60 == other.seconds
    }
}

impl PartialEq<Minute> for Second {
    fn eq(&self, other: &Minute) -> bool {
        self.seconds == other.minutes * 60
    }
}

impl PartialOrd<Second> for Minute {
    fn partial_cmp(&self, other: &Second) -> Option<Ordering> {
        (self.minutes * 60).partial_cmp(&other.seconds)
    }
}

impl PartialOrd<Minute> for Second {
    fn partial_cmp(&self, other: &Minute) -> Option<Ordering> {
        self.seconds.partial_cmp(&(other.minutes * 60))
    }
}

impl Add<Second> for Minute {
    type Output = Second;

    fn add(self, rhs: Second) -> Second {
        Second {
            seconds: self.minutes * 60 + rhs.seconds,
        }
    }
}

impl Add<Minute> for Second {
    type Output = Second;

    fn add(self, rhs: Minute) -> Second {
        Second {
            seconds: self.seconds + rhs.minutes * 60,
        }
    }
}

impl Sub<Second> for Minute {
    type Output = Second;

    fn sub(self, rhs: Second) -> Second {
        Second {
            seconds: self.minutes * 60 - rhs.seconds,
        }
    }
}

impl Sub<Minute> for Second {
    type Output = Second;

    fn sub(self, rhs: Minute) -> Second {
        Second {
            seconds: self.seconds - rhs.minutes * 60,
        }
    }
}

// Cross-type operations between Timespan and Minute

impl PartialEq<Minute> for Timespan {
    fn eq(&self, other: &Minute) -> bool {
        self.nanoseconds == other.minutes as i64 * 60_000_000_000
    }
}

impl PartialEq<Timespan> for Minute {
    fn eq(&self, other: &Timespan) -> bool {
        self.minutes as i64 * 60_000_000_000 == other.nanoseconds
    }
}

impl PartialOrd<Minute> for Timespan {
    fn partial_cmp(&self, other: &Minute) -> Option<Ordering> {
        self.nanoseconds
            .partial_cmp(&(other.minutes as i64 * 60_000_000_000))
    }
}

impl PartialOrd<Timespan> for Minute {
    fn partial_cmp(&self, other: &Timespan) -> Option<Ordering> {
        (self.minutes as i64 * 60_000_000_000).partial_cmp(&other.nanoseconds)
    }
}

impl Add<Minute> for Timespan {
    type Output = Timespan;

    fn add(self, rhs: Minute) -> Timespan {
        Timespan {
            nanoseconds: self.nanoseconds + rhs.minutes as i64 * 60_000_000_000,
        }
    }
}

impl Add<Timespan> for Minute {
    type Output = Timespan;

    fn add(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self.minutes as i64 * 60_000_000_000 + rhs.nanoseconds,
        }
    }
}

impl Sub<Minute> for Timespan {
    type Output = Timespan;

    fn sub(self, rhs: Minute) -> Timespan {
        Timespan {
            nanoseconds: self.nanoseconds - rhs.minutes as i64 * 60_000_000_000,
        }
    }
}

impl Sub<Timespan> for Minute {
    type Output = Timespan;

    fn sub(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self.minutes as i64 * 60_000_000_000 - rhs.nanoseconds,
        }
    }
}

// Cross-type operations between Timespan and Second

impl PartialEq<Second> for Timespan {
    fn eq(&self, other: &Second) -> bool {
        self.nanoseconds == other.seconds as i64 * 1_000_000_000
    }
}

impl PartialEq<Timespan> for Second {
    fn eq(&self, other: &Timespan) -> bool {
        self.seconds as i64 * 1_000_000_000 == other.nanoseconds
    }
}

impl PartialOrd<Second> for Timespan {
    fn partial_cmp(&self, other: &Second) -> Option<Ordering> {
        self.nanoseconds
            .partial_cmp(&(other.seconds as i64 * 1_000_000_000))
    }
}

impl PartialOrd<Timespan> for Second {
    fn partial_cmp(&self, other: &Timespan) -> Option<Ordering> {
        (self.seconds as i64 * 1_000_000_000).partial_cmp(&other.nanoseconds)
    }
}

impl Add<Second> for Timespan {
    type Output = Timespan;

    fn add(self, rhs: Second) -> Timespan {
        Timespan {
            nanoseconds: self.nanoseconds + rhs.seconds as i64 * 1_000_000_000,
        }
    }
}

impl Add<Timespan> for Second {
    type Output = Timespan;

    fn add(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self.seconds as i64 * 1_000_000_000 + rhs.nanoseconds,
        }
    }
}

impl Sub<Second> for Timespan {
    type Output = Timespan;

    fn sub(self, rhs: Second) -> Timespan {
        Timespan {
            nanoseconds: self.nanoseconds - rhs.seconds as i64 * 1_000_000_000,
        }
    }
}

impl Sub<Timespan> for Second {
    type Output = Timespan;

    fn sub(self, rhs: Timespan) -> Timespan {
        Timespan {
            nanoseconds: self.seconds as i64 * 1_000_000_000 - rhs.nanoseconds,
        }
    }
}
