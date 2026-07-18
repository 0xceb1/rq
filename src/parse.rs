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

// ---- evaluation ----------------------------------------------------------
//
// Only numeric atom arithmetic is modelled. Vectors, non-numeric operands,
// null/infinity (`0N`/`0W`), and unary operators are not handled yet.

/// Promotion rank: an operation on two numeric atoms produces the wider type.
const RANK_SHORT: u8 = 0;
const RANK_INT: u8 = 1;
const RANK_LONG: u8 = 2;
const RANK_REAL: u8 = 3;
const RANK_FLOAT: u8 = 4;

fn num_rank(n: &Noun) -> Option<u8> {
    match n {
        Noun::Short(_) => Some(RANK_SHORT),
        Noun::Int(_) => Some(RANK_INT),
        Noun::Long(_) => Some(RANK_LONG),
        Noun::Real(_) => Some(RANK_REAL),
        Noun::Float(_) => Some(RANK_FLOAT),
        _ => None,
    }
}

/// Widen an integer atom to i64. Lossless: only called on operands whose rank is
/// <= the (integer) result rank, so the source is always Short/Int/Long.
fn to_i64(n: &Noun) -> i64 {
    match n {
        Noun::Short(x) => *x as i64,
        Noun::Int(x) => *x as i64,
        Noun::Long(x) => *x,
        _ => unreachable!("to_i64 on non-integer"),
    }
}

fn to_f64(n: &Noun) -> f64 {
    match n {
        Noun::Short(x) => *x as f64,
        Noun::Int(x) => *x as f64,
        Noun::Long(x) => *x as f64,
        Noun::Real(x) => *x as f64,
        Noun::Float(x) => *x,
        _ => unreachable!("to_f64 on non-numeric"),
    }
}

/// Add/Subtract/Multiply on operands already widened to i64.
/// For a Short/Int result the i64 math cannot overflow (operands are i16/i32-ranged),
/// and the caller truncates with `as`, which reproduces q's silent wraparound.
fn narrow_int_op(op: Op, a: i64, b: i64) -> i64 {
    match op {
        Op::Add => a + b,
        Op::Subtract => a - b,
        Op::Multiply => a * b,
        Op::Divide => unreachable!("`%` always promotes to float"),
    }
}

fn long_op(op: Op, a: i64, b: i64) -> i64 {
    match op {
        Op::Add => a.wrapping_add(b),
        Op::Subtract => a.wrapping_sub(b),
        Op::Multiply => a.wrapping_mul(b),
        Op::Divide => unreachable!("`%` always promotes to float"),
    }
}

fn float_op(op: Op, a: f64, b: f64) -> f64 {
    match op {
        Op::Add => a + b,
        Op::Subtract => a - b,
        Op::Multiply => a * b,
        Op::Divide => a / b,
    }
}

/// Apply a binary arithmetic op to two evaluated atoms.
fn apply(op: Op, lhs: Noun, rhs: Noun) -> Result<Noun, Error> {
    let lr = num_rank(&lhs)
        .ok_or_else(|| miette::miette!("type error: '{lhs}' is not a numeric atom"))?;
    let rr = num_rank(&rhs)
        .ok_or_else(|| miette::miette!("type error: '{rhs}' is not a numeric atom"))?;

    // Division always yields a float
    let rank = if matches!(op, Op::Divide) {
        RANK_FLOAT
    } else {
        lr.max(rr)
    };

    let result = match rank {
        RANK_SHORT => Noun::Short(narrow_int_op(op, to_i64(&lhs), to_i64(&rhs)) as i16),
        RANK_INT => Noun::Int(narrow_int_op(op, to_i64(&lhs), to_i64(&rhs)) as i32),
        RANK_LONG => Noun::Long(long_op(op, to_i64(&lhs), to_i64(&rhs))),
        RANK_REAL => Noun::Real(float_op(op, to_f64(&lhs), to_f64(&rhs)) as f32),
        _ => Noun::Float(float_op(op, to_f64(&lhs), to_f64(&rhs))),
    };
    Ok(result)
}

/// Evaluate a parsed [`TokenTree`] into a [`Noun`].
pub fn eval(tree: &TokenTree<'_>, src: &str) -> Result<Noun, Error> {
    match tree {
        TokenTree::Noun(token) => Noun::try_from_token(*token, src),
        // ponytail: binary-only; the parser only ever builds 2-child Cons today.
        // Revisit when unary operators arrive (match on args.len()).
        TokenTree::Cons(op, args) => {
            let lhs = eval(&args[0], src)?;
            let rhs = eval(&args[1], src)?;
            apply(*op, lhs, rhs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> String {
        let tree = Parser::new(src).parse().unwrap();
        format!("{}", eval(&tree, src).unwrap())
    }

    #[test]
    fn basic_arithmetic() {
        assert_eq!(run("2+3"), "5");
        assert_eq!(run("10-4"), "6");
        assert_eq!(run("6*7"), "42");
    }

    #[test]
    fn divide_is_always_float() {
        assert_eq!(run("7%2"), "3.5");
        assert_eq!(run("10%2"), "5"); // 5.0 float, printed without a suffix by Noun's Display
    }

    #[test]
    fn promotes_to_wider_type() {
        // Long + Int -> Long: a "3i" result would print "3i", so plain "3" proves promotion.
        assert_eq!(run("1i+2"), "3");
        // Short + Long -> Long.
        assert_eq!(run("2h+3"), "5");
        // Long + Float -> Float.
        assert_eq!(run("2+3.0"), "5");
    }

    #[test]
    fn integer_overflow_wraps() {
        assert_eq!(run("32767h+1h"), "-32768h"); // Short wraps via truncation
        assert_eq!(run("9223372036854775807+1"), "-9223372036854775808"); // Long wraps
    }
}
