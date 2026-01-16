pub mod chrono;
pub mod symbol;

use crate::qtype::chrono::{Date, Minute, Month, Second, Timespan, Timestamp};
use crate::qtype::symbol::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum Q {
    // atom
    Boolean(bool),
    Guid(uuid::Uuid),
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Real(f32),
    Float(f64),
    Char(u8),
    Symbol(Symbol),
    Timestamp(Timestamp),
    Month(Month),
    Date(Date),
    Timespan(Timespan),
    Minute(Minute),
    Second(Second),

    // vector
    Booleans(Vec<bool>),
    Guids(Vec<uuid::Uuid>),
    Bytes(Vec<u8>),
    Shorts(Vec<i16>),
    Ints(Vec<i32>),
    Longs(Vec<i64>),
    Reals(Vec<f32>),
    Floats(Vec<f64>),
    String(Vec<u8>),
    Symbols(Vec<Symbol>),
    Timestamps(Vec<Timestamp>),
    Months(Vec<Month>),
    Dates(Vec<Date>),
    Timespans(Vec<Timespan>),
    Minutes(Vec<Minute>),
    Seconds(Vec<Second>),
}
