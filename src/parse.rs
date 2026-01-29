use crate::lex::*;
use crate::qtype::symbol::Symbol;
use itertools::Itertools;
use miette::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // %
}

impl From<Token<'_>> for Op {
    fn from(value: Token<'_>) -> Self {
        use TokenKind as T;
        match value.kind {
            T::Plus => Op::Add,
            T::Minus => Op::Subtract,
            T::Star => Op::Multiply,
            T::Percent => Op::Divide,
            _ => panic!("No a valid Op token"),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Subtract => write!(f, "-"),
            Op::Multiply => write!(f, "*"),
            Op::Divide => write!(f, "%"),
        }
    }
}

/// kdb+/q *noun* data type, include:
/// - atomic values
/// - list of atomic values (vector)
/// - nested list
///
/// Reference: <https://code.kx.com/q/basics/syntax/#nouns>
#[derive(Debug, Clone, PartialEq)]
pub enum Noun {
    Boolean(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    Real(f32),
    Float(f64),
    Char(char),
    Symbol(Symbol),
    Date,
    Month,
    Minute,
    Second,
    Timespan,
    Timestamp,

    VecBoolean(Vec<bool>),
    VecByte(Vec<u8>),
    VecShort(Vec<i16>),
    VecInt(Vec<i32>),
    VecLong(Vec<i64>),
    VecReal(Vec<f32>),
    VecFloat(Vec<f64>),
    VecChar(String),
    VecSymbol(Vec<Symbol>),
    VecDate,
    VecMonth,
    VecMinute,
    VecSecond,
    VecTimespan,
    VecTimestamp,
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
            // TokenKind::Single(Atomic::Boolean) => {
            //     Ok(Noun::Boolean(origin.parse::<bool>().map_err(|_| {
            //         InvalidLiteralError::new(src, origin, offset..offset + origin.len(), None)
            //     })?))
            // }
            // TokenKind::Single(Atomic::Byte) => {
            //     Ok(Noun::Byte(origin.parse::<u8>().map_err(|_| {
            //         InvalidLiteralError::new(src, origin, offset..offset + origin.len(), None)
            //     })?))
            // }
            TokenKind::Single(Atomic::Short) => Ok(Noun::Short(
                origin
                    .parse::<i16>()
                    .map_err(parse_err!("cannot parse into Short"))?,
            )),
            TokenKind::Single(Atomic::Int) => Ok(Noun::Int(
                origin
                    .parse::<i32>()
                    .map_err(parse_err!("cannot parse into Int"))?,
            )),
            TokenKind::Single(Atomic::Long) => Ok(Noun::Long(
                origin
                    .parse::<i64>()
                    .map_err(parse_err!("cannot parse into Long"))?,
            )),
            TokenKind::Single(Atomic::Real) => Ok(Noun::Real(
                origin
                    .parse::<f32>()
                    .map_err(parse_err!("cannot parse into Real"))?,
            )),
            TokenKind::Single(Atomic::Float) => Ok(Noun::Float(
                origin
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

            _ => todo!(),
        }
    }
}

impl fmt::Display for Noun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(x) => write!(f, "{}", if *x { "1b" } else { "0b" }),
            Self::Byte(_) => todo!(),
            Self::Short(x) => write!(f, "{x}h"),
            Self::Int(x) => write!(f, "{x}i"),
            Self::Long(x) => write!(f, "{x}"),
            Self::Real(x) => write!(f, "{x}e"),
            Self::Float(x) => write!(f, "{x}"),
            Self::Char(x) => write!(f, "\"{x}\""),
            Self::Symbol(x) => write!(f, "{x}"),

            Self::VecShort(v) => write!(f, "{}h", v.iter().format(" ")),
            Self::VecInt(v) => write!(f, "{}i", v.iter().format(" ")),
            Self::VecLong(v) => write!(f, "{}", v.iter().format(" ")),
            Self::VecReal(v) => write!(f, "{}e", v.iter().format(" ")),
            Self::VecFloat(v) => write!(f, "{}", v.iter().format(" ")),

            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenTree<'de> {
    Noun(Token<'de>),
    Cons(Op, Vec<TokenTree<'de>>),
}

impl fmt::Display for TokenTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Noun(token) => write!(f, "{}", token.origin),
            TokenTree::Cons(op, exprs) => {
                write!(f, "({}", op)?;
                for s in exprs {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}

pub struct Parser<'de> {
    source: &'de str,
    lexer: Lexer<'de>,
}

impl<'de> Parser<'de> {
    pub fn new(source: &'de str) -> Self {
        let lexer = Lexer::new(source);
        Self { source, lexer }
    }

    pub fn parse(&mut self) -> Result<TokenTree<'de>, Error> {
        let mut lhs = match self
            .lexer
            .next()
            .transpose()?
            .ok_or(miette::miette!("End of tokens"))?
        {
            t @ Token {
                kind: TokenKind::Single(_) | TokenKind::Vector(_),
                ..
            } => TokenTree::Noun(t),
            t => Err(miette::miette!("bad token: {t}"))?,
        };
        loop {
            let op = match self.lexer.peek() {
                Some(&Ok(t)) if is_op_token(t) => Op::from(t),
                Some(_) => panic!("bad token"),
                None => break,
            };

            self.lexer.next();
            let rhs = self.parse()?;
            lhs = TokenTree::Cons(op, vec![lhs, rhs]);
        }
        Ok(lhs)
    }
}

fn is_op_token(t: Token<'_>) -> bool {
    use TokenKind as T;
    matches!(t.kind, T::Plus | T::Minus | T::Star | T::Percent)
}
