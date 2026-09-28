pub mod chrono;
pub mod symbol;

use crate::error::QError;
use crate::lex::{Atomic, InvalidLiteralError, Token, TokenKind};
use crate::qtype::chrono::{Date, Minute, Month, Second, Timespan, Timestamp};
use crate::qtype::symbol::Symbol;
use itertools::Itertools;
use miette::Error;
use std::fmt;

/// kdb+/q *noun* data type, include:
/// - atomic values
/// - list of atomic values (vector)
/// - nested list
///
/// Reference: <https://code.kx.com/q/basics/syntax/#nouns>
#[derive(Debug, Clone, PartialEq)]
pub enum Noun {
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
    Date(Date),
    Month(Month),
    Minute(Minute),
    Second(Second),
    Timespan(Timespan),
    Timestamp(Timestamp),

    VecBoolean(Vec<bool>),
    VecGuid(Vec<uuid::Uuid>),
    VecByte(Vec<u8>),
    VecShort(Vec<i16>),
    VecInt(Vec<i32>),
    VecLong(Vec<i64>),
    VecReal(Vec<f32>),
    VecFloat(Vec<f64>),
    VecChar(Vec<u8>),
    VecSymbol(Vec<Symbol>),
    VecDate(Vec<Date>),
    VecMonth(Vec<Month>),
    VecMinute(Vec<Minute>),
    VecSecond(Vec<Second>),
    VecTimespan(Vec<Timespan>),
    VecTimestamp(Vec<Timestamp>),
}

impl Noun {
    /// Create a Noun from raw [`Token`]
    pub fn try_from_token(token: Token<'_>, src: &'_ str) -> Result<Noun, Error> {
        let Token {
            kind,
            origin,
            offset,
        } = token;
        macro_rules! parse_err {
            //TODO: carry clearer error message
            ($reason:expr) => {
                |_| {
                    InvalidLiteralError::new(
                        src,
                        origin,
                        $reason,
                        offset..offset + origin.len(),
                        None,
                    )
                }
            };
        }
        match kind {
            TokenKind::Single(Atomic::Boolean) => Ok(Noun::Boolean(
                origin.strip_suffix('b').unwrap_or(origin) == "1",
            )),
            TokenKind::Single(Atomic::Byte) => Ok(Noun::Byte(
                u8::from_str_radix(origin.strip_prefix("0x").unwrap_or(origin), 16)
                    .map_err(parse_err!("cannot parse into Byte"))?,
            )),
            TokenKind::Vector(Atomic::Boolean) => Ok(Noun::VecBoolean(
                origin
                    .strip_suffix('b')
                    .unwrap_or(origin)
                    .chars()
                    .map(|c| c == '1')
                    .collect(),
            )),
            TokenKind::Vector(Atomic::Byte) => {
                let hex = origin.strip_prefix("0x").unwrap_or(origin);
                let vec = hex
                    .as_bytes()
                    .chunks(2)
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(parse_err!("cannot parse into vector Byte"))?;
                Ok(Noun::VecByte(vec))
            }
            TokenKind::Single(Atomic::Short) => Ok(Noun::Short(
                origin
                    .strip_suffix('h')
                    .unwrap_or(origin)
                    .parse::<i16>()
                    .map_err(parse_err!("cannot parse into Short"))?,
            )),
            TokenKind::Single(Atomic::Int) => Ok(Noun::Int(
                origin
                    .strip_suffix('i')
                    .unwrap_or(origin)
                    .parse::<i32>()
                    .map_err(parse_err!("cannot parse into Int"))?,
            )),
            TokenKind::Single(Atomic::Long) => Ok(Noun::Long(
                origin
                    .strip_suffix('j')
                    .unwrap_or(origin)
                    .parse::<i64>()
                    .map_err(parse_err!("cannot parse into Long"))?,
            )),
            TokenKind::Single(Atomic::Real) => Ok(Noun::Real(
                origin
                    .strip_suffix('e')
                    .unwrap_or(origin)
                    .parse::<f32>()
                    .map_err(parse_err!("cannot parse into Real"))?,
            )),
            TokenKind::Single(Atomic::Float) => Ok(Noun::Float(
                origin
                    .strip_suffix('f')
                    .unwrap_or(origin)
                    .parse::<f64>()
                    .map_err(parse_err!("cannot parse into Float"))?,
            )),

            // Vector types (space-separated numerical literals)
            TokenKind::Vector(Atomic::Short) => {
                let content = origin.strip_suffix('h').unwrap_or(origin);
                let vec = content
                    .split_whitespace()
                    .map(|s| s.parse::<i16>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(parse_err!("cannot parse into vector Short"))?;
                Ok(Noun::VecShort(vec))
            }
            TokenKind::Vector(Atomic::Int) => {
                let content = origin.strip_suffix('i').unwrap_or(origin);
                let vec = content
                    .split_whitespace()
                    .map(|s| s.parse::<i32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(parse_err!("cannot parse into vector Int"))?;
                Ok(Noun::VecInt(vec))
            }
            TokenKind::Vector(Atomic::Long) => {
                let content = origin.strip_suffix('j').unwrap_or(origin);
                let vec = content
                    .split_whitespace()
                    .map(|s| s.parse::<i64>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(parse_err!("cannot parse into vector Long"))?;
                Ok(Noun::VecLong(vec))
            }
            TokenKind::Vector(Atomic::Real) => {
                let content = origin.strip_suffix('e').unwrap_or(origin);
                let vec = content
                    .split_whitespace()
                    .map(|s| s.parse::<f32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(parse_err!("cannot parse into vector Real"))?;
                Ok(Noun::VecReal(vec))
            }
            TokenKind::Vector(Atomic::Float) => {
                let content = origin.strip_suffix('f').unwrap_or(origin);
                let vec = content
                    .split_whitespace()
                    .map(|s| s.parse::<f64>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(parse_err!("cannot parse into vector Float"))?;
                Ok(Noun::VecFloat(vec))
            }
            // Atom vs vector is only known after unescaping: "\t" is one char
            TokenKind::Single(Atomic::Char) | TokenKind::Vector(Atomic::Char) => {
                let bytes = unescape(&origin[1..origin.len() - 1])
                    .ok_or(())
                    .map_err(parse_err!("invalid escape in string"))?;
                Ok(match bytes.as_slice() {
                    [b] => Noun::Char(*b),
                    _ => Noun::VecChar(bytes),
                })
            }

            _ => Err(QError::nyi(format!("'{origin}' is not supported yet"))
                .at(src, offset..offset + origin.len()))?,
        }
    }
}

impl fmt::Display for Noun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(x) => write!(f, "{}", if *x { "1b" } else { "0b" }),
            Self::Byte(x) => write!(f, "0x{x:02x}"),
            Self::Short(x) => write!(f, "{x}h"),
            Self::Int(x) => write!(f, "{x}i"),
            Self::Long(x) => write!(f, "{x}"),
            Self::Real(x) => write!(f, "{x}e"),
            Self::Float(x) => write!(f, "{x}"),
            Self::Char(x) => write_q_string(f, &[*x]),
            Self::Symbol(x) => write!(f, "{x}"),
            Self::Date(x) => write!(f, "{x}"),
            Self::Month(x) => write!(f, "{x}"),
            Self::Minute(x) => write!(f, "{x}"),
            Self::Second(x) => write!(f, "{x}"),
            Self::Timespan(x) => write!(f, "{x}"),
            Self::Timestamp(x) => write!(f, "{x}"),

            Self::VecBoolean(v) => {
                write!(
                    f,
                    "{}b",
                    v.iter().map(|b| if *b { '1' } else { '0' }).format("")
                )
            }
            Self::VecByte(v) => write!(
                f,
                "0x{}",
                v.iter().format_with("", |b, f| f(&format_args!("{b:02x}")))
            ),
            Self::VecShort(v) => write!(f, "{}h", v.iter().format(" ")),
            Self::VecInt(v) => write!(f, "{}i", v.iter().format(" ")),
            Self::VecLong(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecReal(v) => write!(f, "{}e", v.iter().format(" ")),
            Self::VecFloat(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecChar(v) => write_q_string(f, v),
            Self::VecDate(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecMonth(v) => write!(
                f,
                "{}m",
                v.iter()
                    .format_with(" ", |m, f| { f(&m.to_literal().trim_end_matches('m')) })
            ),
            Self::VecMinute(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecSecond(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecTimespan(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecTimestamp(v) => write!(f, "{}", v.iter().format(" ")),

            _ => todo!(),
        }
    }
}

/// Decode a q string body into bytes: `\n \t \r \" \\` and 3-digit octal `\ooo`.
fn unescape(s: &str) -> Option<Vec<u8>> {
    let mut bytes = s.bytes();
    let mut out = Vec::with_capacity(s.len());
    while let Some(b) = bytes.next() {
        if b != b'\\' {
            out.push(b);
            continue;
        }
        out.push(match bytes.next()? {
            b'n' => b'\n',
            b't' => b'\t',
            b'r' => b'\r',
            c @ (b'"' | b'\\') => c,
            d @ b'0'..=b'7' => {
                let digits = [d, bytes.next()?, bytes.next()?];
                u8::from_str_radix(std::str::from_utf8(&digits).ok()?, 8).ok()?
            }
            _ => return None,
        });
    }
    Some(out)
}

/// Display bytes the way q does: printable ASCII as-is, everything else escaped.
fn write_q_string(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    write!(f, "\"")?;
    for &b in bytes {
        match b {
            b'"' | b'\\' => write!(f, "\\{}", b as char)?,
            b'\n' => write!(f, "\\n")?,
            b'\t' => write!(f, "\\t")?,
            b'\r' => write!(f, "\\r")?,
            0x20..=0x7e => write!(f, "{}", b as char)?,
            _ => write!(f, "\\{b:03o}")?,
        }
    }
    write!(f, "\"")
}
