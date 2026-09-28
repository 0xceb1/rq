//! Tree-walking evaluator: turns a parsed [`TokenTree`] into a [`Noun`].
//!
//! TODO: Only numeric arithmetic (atoms and vectors) is modelled. Non-numeric
//! operands and null/infinity (`0N`/`0W`) are not handled yet.

use crate::error::QError;
use crate::parse::{Op, TokenTree};
use crate::qtype::Noun;
use miette::Error;

/// Evaluate a parsed [`TokenTree`] into a [`Noun`].
pub fn eval(tree: &TokenTree<'_>, src: &str) -> Result<Noun, Error> {
    match tree {
        TokenTree::Noun(token) => Noun::try_from_token(*token, src),
        // `parse_operand` only ever builds a 1-child Cons for unary `-`,
        // and `parse`'s loop only ever builds a 2-child Cons for binary ops.
        TokenTree::Cons(op, span, args) => {
            let result = match args.as_slice() {
                [operand] => negate(eval(operand, src)?),
                [lhs, rhs] => apply(*op, eval(lhs, src)?, eval(rhs, src)?),
                _ => unreachable!("Cons is always unary (`-x`) or binary"),
            };
            // nested errors already returned via `?` above, located at their own op
            result.map_err(|e| e.at(src, *span).into())
        }
    }
}

/// Unary minus
fn negate(v: Noun) -> Result<Noun, QError> {
    let zero = match v.rank().ok_or_else(|| QError::type_("not numeric"))? {
        NumRank::Boolean => Noun::Boolean(false),
        NumRank::Byte => Noun::Byte(0),
        NumRank::Short => Noun::Short(0),
        NumRank::Int => Noun::Int(0),
        NumRank::Long => Noun::Long(0),
        NumRank::Real => Noun::Real(0.0),
        NumRank::Float => Noun::Float(0.0),
    };
    apply(Op::Subtract, zero, v)
}

fn apply(op: Op, lhs: Noun, rhs: Noun) -> Result<Noun, QError> {
    let lr = lhs
        .rank()
        .ok_or_else(|| QError::type_("lhs is not numeric"))?;
    let rr = rhs
        .rank()
        .ok_or_else(|| QError::type_("rhs is not numeric"))?;

    use NumRank as R;
    if op.is_comparison() {
        let s = match lr.max(rr) {
            R::Real | R::Float => {
                combine(lhs.into_f64(), rhs.into_f64(), |a, b| compare(op, a, b))?
            }
            R::Boolean | R::Byte | R::Short | R::Int | R::Long => {
                combine(lhs.into_i64(), rhs.into_i64(), |a, b| compare(op, a, b))?
            }
        };
        return Ok(s.into_noun(Noun::Boolean, Noun::VecBoolean));
    }

    let widened = op.result_rank_override().unwrap_or(lr.max(rr));
    let rank = if widened == NumRank::Boolean && !matches!(op, Op::And | Op::Or) {
        NumRank::Int
    } else {
        widened
    };

    Ok(match rank {
        R::Boolean | R::Byte | R::Short | R::Int | R::Long => {
            let s = combine(lhs.into_i64(), rhs.into_i64(), |a, b| int_op(op, a, b))?;
            match rank {
                R::Boolean => s.map(|x| x != 0).into_noun(Noun::Boolean, Noun::VecBoolean),
                R::Byte => s.map(|x| x as u8).into_noun(Noun::Byte, Noun::VecByte),
                R::Short => s.map(|x| x as i16).into_noun(Noun::Short, Noun::VecShort),
                R::Int => s.map(|x| x as i32).into_noun(Noun::Int, Noun::VecInt),
                _ => s.into_noun(Noun::Long, Noun::VecLong),
            }
        }
        R::Real | R::Float => {
            let s = combine(lhs.into_f64(), rhs.into_f64(), |a, b| float_op(op, a, b))?;
            match rank {
                R::Real => s.map(|x| x as f32).into_noun(Noun::Real, Noun::VecReal),
                _ => s.into_noun(Noun::Float, Noun::VecFloat),
            }
        }
    })
}

impl Op {
    /// Some ops force a result type regardless of operand rank, e.g. `%` (division)
    /// always yields a float in q, even for two integer operands.
    fn result_rank_override(&self) -> Option<NumRank> {
        match self {
            Op::Divide => Some(NumRank::Float),
            _ => None,
        }
    }

    fn is_comparison(&self) -> bool {
        use Op as O;
        matches!(
            self,
            O::Equal | O::NotEqual | O::Less | O::LessEqual | O::Greater | O::GreaterEqual
        )
    }
}

/// Promotion rank: an operation on two numeric atoms produces the wider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum NumRank {
    Boolean,
    Byte,
    Short,
    Int,
    Long,
    Real,
    Float,
}

/// An operand widened for computation, tagged with its shape.
/// An atom broadcasts; a 1-element vector does not.
#[derive(Debug, Clone, PartialEq)]
enum Shape<T> {
    Atom(T),
    Vector(Vec<T>),
}

/// Match atom/vector variant pairs and widen them into `Shape<$to>` via `From`,
/// with a fallback arm for everything else.
macro_rules! widen {
    ($noun:expr => $to:ty; $($atom:ident / $vec:ident),+; $other:ident => $fallback:expr) => {
        match $noun {
            $(
                Noun::$atom(x) => Shape::Atom(<$to>::from(x)),
                Noun::$vec(v) => Shape::Vector(v.into_iter().map(<$to>::from).collect()),
            )+
            $other => $fallback,
        }
    };
}

impl Noun {
    fn rank(&self) -> Option<NumRank> {
        match self {
            Noun::Boolean(_) => Some(NumRank::Boolean),
            Noun::Byte(_) => Some(NumRank::Byte),
            Noun::Short(_) => Some(NumRank::Short),
            Noun::Int(_) => Some(NumRank::Int),
            Noun::Long(_) => Some(NumRank::Long),
            Noun::Real(_) => Some(NumRank::Real),
            Noun::Float(_) => Some(NumRank::Float),
            Noun::VecBoolean(_) => Some(NumRank::Boolean),
            Noun::VecByte(_) => Some(NumRank::Byte),
            Noun::VecShort(_) => Some(NumRank::Short),
            Noun::VecInt(_) => Some(NumRank::Int),
            Noun::VecLong(_) => Some(NumRank::Long),
            Noun::VecReal(_) => Some(NumRank::Real),
            Noun::VecFloat(_) => Some(NumRank::Float),
            _ => None,
        }
    }

    /// Widen an integer-family noun to `i64`.
    fn into_i64(self) -> Shape<i64> {
        widen!(self => i64;
            Boolean / VecBoolean, Byte / VecByte, Short / VecShort, Int / VecInt, Long / VecLong;
            other => unreachable!("into_i64 on non-integer '{other}'"))
    }

    /// Widen a numeric noun to `f64`.
    fn into_f64(self) -> Shape<f64> {
        widen!(self => f64;
            Real / VecReal, Float / VecFloat;
            int => int.into_i64().map(|x| x as f64))
    }
}

impl<T> Shape<T> {
    fn map<U>(self, f: impl Fn(T) -> U) -> Shape<U> {
        match self {
            Shape::Atom(x) => Shape::Atom(f(x)),
            Shape::Vector(v) => Shape::Vector(v.into_iter().map(f).collect()),
        }
    }

    /// Wrap back into a [`Noun`], e.g. `s.into_noun(Noun::Short, Noun::VecShort)`.
    fn into_noun(self, atom: fn(T) -> Noun, vector: fn(Vec<T>) -> Noun) -> Noun {
        match self {
            Shape::Atom(x) => atom(x),
            Shape::Vector(v) => vector(v),
        }
    }
}

/// Integer ops on operands already widened to i64. Wrapping reproduces q's silent
/// overflow for Long; for Short/Int the i64 math never overflows, and the later
/// `as` truncation wraps instead.
fn int_op(op: Op, a: i64, b: i64) -> i64 {
    match op {
        Op::Add => a.wrapping_add(b),
        Op::Subtract => a.wrapping_sub(b),
        Op::Multiply => a.wrapping_mul(b),
        Op::Divide => unreachable!("`%` always promotes to float"),
        Op::And => a.min(b),
        Op::Or => a.max(b),
        _ => unreachable!("comparisons are handled by `compare`"),
    }
}

fn float_op(op: Op, a: f64, b: f64) -> f64 {
    match op {
        Op::Add => a + b,
        Op::Subtract => a - b,
        Op::Multiply => a * b,
        Op::Divide => a / b,
        Op::And => a.min(b),
        Op::Or => a.max(b),
        _ => unreachable!("comparisons are handled by `compare`"),
    }
}

/// Comparisons yield booleans. Operands are compared by value across types, so `1=1.0`.
// TODO: q compares floats with a relative tolerance (`\P`); this is exact.
fn compare<T: PartialOrd>(op: Op, a: T, b: T) -> bool {
    match op {
        Op::Equal => a == b,
        Op::NotEqual => a != b,
        Op::Less => a < b,
        Op::LessEqual => a <= b,
        Op::Greater => a > b,
        Op::GreaterEqual => a >= b,
        _ => unreachable!("not a comparison: {op}"),
    }
}

/// Broadcasting
fn combine<T: Copy, U>(
    lhs: Shape<T>,
    rhs: Shape<T>,
    f: impl Fn(T, T) -> U,
) -> Result<Shape<U>, QError> {
    use Shape::{Atom, Vector};
    match (lhs, rhs) {
        (Atom(x), Atom(y)) => Ok(Atom(f(x, y))),
        (Atom(x), Vector(ys)) => Ok(Vector(ys.into_iter().map(|y| f(x, y)).collect())),
        (Vector(xs), Atom(y)) => Ok(Vector(xs.into_iter().map(|x| f(x, y)).collect())),
        (Vector(xs), Vector(ys)) if xs.len() == ys.len() => Ok(Vector(
            xs.into_iter().zip(ys).map(|(x, y)| f(x, y)).collect(),
        )),
        (Vector(xs), Vector(ys)) => Err(QError::length(xs.len(), ys.len())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Parser;

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

    #[test]
    fn unary_minus() {
        assert_eq!(run("-5"), "-5");
        assert_eq!(run("-2h"), "-2h"); // stays Short, doesn't promote to Long
    }

    #[test]
    fn unary_minus_binds_tighter_than_binary_ops() {
        assert_eq!(run("2+-3"), "-1"); // (+ 2 (- 3)), not -(2+3)
        assert_eq!(run("2--3"), "5"); // (- 2 (- 3)) == 2 - (-3)
    }

    #[test]
    fn parens_override_evaluation_order() {
        assert_eq!(run("(2+3)*4"), "20");
        assert_eq!(run("2*(3+4)"), "14");
    }

    #[test]
    fn unary_minus_applies_to_parenthesized_expr() {
        assert_eq!(run("-(2+3)"), "-5");
    }

    #[test]
    fn boolean_and_byte_literals() {
        assert_eq!(run("1b"), "1b");
        assert_eq!(run("0b"), "0b");
        assert_eq!(run("101b"), "101b");
        assert_eq!(run("0x2a"), "0x2a");
        assert_eq!(run("0x01020a"), "0x01020a");
    }

    #[test]
    fn boolean_arithmetic_promotes_to_int() {
        // a 1-bit type can't hold 2, so q always promotes pure-boolean results to int
        assert_eq!(run("1b+1b"), "2i");
        // mixing with a wider type just widens normally
        assert_eq!(run("1b+2i"), "3i");
    }

    #[test]
    fn byte_arithmetic_wraps() {
        assert_eq!(run("0x01+0x01"), "0x02");
        assert_eq!(run("0xff+0x01"), "0x00"); // wraps mod 256, like Short/Long overflow
    }

    #[test]
    fn and_or_are_min_max() {
        assert_eq!(run("3&5"), "3");
        assert_eq!(run("3|5"), "5");
    }

    #[test]
    fn and_or_keep_boolean_type() {
        // unlike +/-/*, min/max of two booleans can never leave {0,1}
        assert_eq!(run("1b&0b"), "0b");
        assert_eq!(run("1b|0b"), "1b");
        // mixing with a wider type still widens normally
        assert_eq!(run("1b&2i"), "1i");
    }

    #[test]
    fn negate_wraps_at_short_min() {
        // -32768h can't be typed directly (32768 overflows i16 before negation),
        // so exercise the wraparound via negate() directly.
        assert_eq!(
            negate(Noun::Short(i16::MIN)).unwrap(),
            Noun::Short(i16::MIN)
        );
    }

    #[test]
    fn combine_broadcasts_atoms_only() {
        use Shape::{Atom, Vector};
        let add = |x: i64, y: i64| x + y;
        assert_eq!(combine(Atom(1), Atom(2), add).unwrap(), Atom(3));
        assert_eq!(
            combine(Vector(vec![1, 2]), Atom(1), add).unwrap(),
            Vector(vec![2, 3])
        );
        assert_eq!(
            combine(Atom(1), Vector(vec![1, 2]), add).unwrap(),
            Vector(vec![2, 3])
        );
        assert_eq!(
            combine(Vector(vec![1, 2]), Vector(vec![3, 4]), add).unwrap(),
            Vector(vec![4, 6])
        );
        // a 1-element vector is not an atom: no broadcasting
        assert!(combine(Vector(vec![1]), Vector(vec![1, 2]), add).is_err());
    }

    fn eval_src(src: &str) -> Noun {
        eval(&Parser::new(src).parse().unwrap(), src).unwrap()
    }

    #[test]
    fn strings_are_bytes() {
        assert_eq!(eval_src(r#""你好""#), Noun::VecChar("你好".into()));
        assert_eq!(run(r#""你好""#), r#""\344\275\240\345\245\275""#);
    }

    #[test]
    fn escaped_single_char_is_an_atom() {
        assert_eq!(eval_src(r#""\t""#), Noun::Char(b'\t'));
        assert_eq!(eval_src(r#""\001""#), Noun::Char(1));
        assert_eq!(eval_src(r#""a""#), Noun::Char(b'a'));
    }

    #[test]
    fn string_display_escapes() {
        assert_eq!(run(r#""a\"b\\c\n""#), r#""a\"b\\c\n""#);
        assert_eq!(run(r#""\001""#), r#""\001""#);
    }

    #[test]
    fn invalid_escape_is_an_error() {
        let src = r#""\q""#;
        assert!(eval(&Parser::new(src).parse().unwrap(), src).is_err());
    }

    #[test]
    fn vector_atom_broadcast() {
        assert_eq!(run("1 2 3+1"), "2 3 4");
        assert_eq!(run("1+1 2 3"), "2 3 4");
    }

    #[test]
    fn vector_vector_elementwise() {
        assert_eq!(run("1 2 3*4 5 6"), "4 10 18");
    }

    #[test]
    fn vector_length_mismatch_is_an_error() {
        let src = "1 2+1 2 3";
        let err = eval(&Parser::new(src).parse().unwrap(), src).unwrap_err();
        let err = err.downcast::<QError>().unwrap();
        assert_eq!(err.name, "length");
        assert_eq!(err.label, "lhs has 2 items, rhs has 3");
        assert_eq!(err.span, Some((3, 1).into())); // points at `+`
    }

    #[test]
    fn nested_length_error_points_at_inner_op() {
        let src = "(1 2+1 2 3)*2";
        let err = eval(&Parser::new(src).parse().unwrap(), src).unwrap_err();
        assert_eq!(err.downcast::<QError>().unwrap().span, Some((4, 1).into()));
    }

    #[test]
    fn vector_type_promotion() {
        assert_eq!(run("1 2h+1"), "2 3"); // Short vec + Long -> Long vec
        assert_eq!(run("1 2 3%2"), "0.5 1 1.5"); // always float
        assert_eq!(run("101b+1b"), "2 1 2i"); // bool arithmetic -> int
        assert_eq!(run("101b&110b"), "100b"); // & | keep bool
    }

    #[test]
    fn vector_negate() {
        assert_eq!(run("-1 2 3"), "-1 -2 -3");
    }

    #[test]
    fn vector_in_nested_expr() {
        assert_eq!(run("(1 2 3+1)*2"), "4 6 8");
    }

    #[test]
    fn comparison_atoms() {
        assert_eq!(run("1=1"), "1b");
        assert_eq!(run("1<>2"), "1b");
        assert_eq!(run("1<2"), "1b");
        assert_eq!(run("2<1"), "0b");
        assert_eq!(run("1<=1"), "1b");
        assert_eq!(run("1>2"), "0b");
        assert_eq!(run("2>=3"), "0b");
    }

    #[test]
    fn comparison_across_types() {
        assert_eq!(run("1=1.0"), "1b");
        assert_eq!(run("1h<2.5"), "1b");
        assert_eq!(run("1b=1"), "1b");
    }

    #[test]
    fn comparison_vectors() {
        assert_eq!(run("1 2 3=2"), "010b");
        assert_eq!(run("1 2 3<2 2 2"), "100b");
        let src = "1 2=1 2 3";
        assert!(eval(&Parser::new(src).parse().unwrap(), src).is_err());
    }

    #[test]
    fn comparison_composes_right_to_left() {
        assert_eq!(run("1 2 3=1+0 1 2"), "111b"); // = (1+0 1 2)
        assert_eq!(run("(1 2 3>1)+1"), "1 2 2"); // booleans promote in arithmetic
    }

    fn eval_err(src: &str) -> QError {
        let err = Parser::new(src)
            .parse()
            .and_then(|t| eval(&t, src))
            .unwrap_err();
        err.downcast::<QError>().unwrap()
    }

    #[test]
    fn type_error_points_at_op() {
        let err = eval_err(r#""ab"+1"#);
        assert_eq!((err.name, err.span), ("type", Some((4, 1).into())));
    }

    #[test]
    fn unsupported_input_is_nyi_not_a_panic() {
        assert_eq!(eval_err("1 x").name, "nyi"); // juxtaposition
        assert_eq!(eval_err("1+2;3").name, "nyi");
        assert_eq!(eval_err("`a").name, "nyi"); // symbol literal
    }
}
