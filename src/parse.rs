use crate::error::QError;
use crate::lex::*;
use miette::{Error, LabeledSpan, SourceSpan};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // %
    And,      // &
    Or,       // |

    Equal,        // =
    NotEqual,     // <>
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Subtract => write!(f, "-"),
            Op::Multiply => write!(f, "*"),
            Op::Divide => write!(f, "%"),
            Op::And => write!(f, "&"),
            Op::Or => write!(f, "|"),
            Op::Equal => write!(f, "="),
            Op::NotEqual => write!(f, "<>"),
            Op::Less => write!(f, "<"),
            Op::LessEqual => write!(f, "<="),
            Op::Greater => write!(f, ">"),
            Op::GreaterEqual => write!(f, ">="),
        }
    }
}

impl Op {
    fn from_kind(kind: TokenKind) -> Option<Op> {
        use TokenKind as T;
        Some(match kind {
            T::Plus => Op::Add,
            T::Minus => Op::Subtract,
            T::Star => Op::Multiply,
            T::Percent => Op::Divide,
            T::Ampersand => Op::And,
            T::Pipe => Op::Or,
            T::Equal => Op::Equal,
            T::NotEqual => Op::NotEqual,
            T::Less => Op::Less,
            T::LessEqual => Op::LessEqual,
            T::Greater => Op::Greater,
            T::GreaterEqual => Op::GreaterEqual,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub enum TokenTree<'de> {
    Noun(Token<'de>),
    Cons(Op, SourceSpan, Vec<TokenTree<'de>>),
}

impl fmt::Display for TokenTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Noun(token) => write!(f, "{}", token.origin),
            TokenTree::Cons(op, _, exprs) => {
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
        let tree = self.parse_expr()?;
        match self.lexer.next().transpose()? {
            None => Ok(tree),
            Some(t) => Err(self.unexpected(t, "here")),
        }
    }

    fn parse_expr(&mut self) -> Result<TokenTree<'de>, Error> {
        let mut lhs = self.parse_operand()?;
        loop {
            let t = match self.lexer.peek() {
                None
                | Some(Ok(Token {
                    kind: TokenKind::RightParen,
                    ..
                })) => break,
                Some(Ok(t)) => *t,
                Some(Err(_)) => return Err(self.next_token().unwrap_err()),
            };
            let op = Op::from_kind(t.kind).ok_or_else(|| {
                QError::nyi(format!("'{}' is not supported yet", t.origin))
                    .at(self.source, span_of(t))
            })?;

            self.lexer.next();
            let rhs = self.parse_expr()?;
            lhs = TokenTree::Cons(op, span_of(t), vec![lhs, rhs]);
        }
        Ok(lhs)
    }

    /// Parse a single operand: a literal noun, a parenthesis group,
    /// or a unary `-` applied to either.
    fn parse_operand(&mut self) -> Result<TokenTree<'de>, Error> {
        let token = self.next_token()?;
        if token.kind != TokenKind::Minus {
            return self.noun_or_group(token);
        }
        let inner = self.next_token()?;
        match inner.kind {
            TokenKind::Single(_) | TokenKind::Vector(_) | TokenKind::LeftParen => {
                Ok(TokenTree::Cons(
                    Op::Subtract,
                    span_of(token),
                    vec![self.noun_or_group(inner)?],
                ))
            }
            _ => Err(self.unexpected(inner, "expected a literal or '(' after '-'")),
        }
    }

    /// A literal noun or a `(...)` group — the two forms an operand can take.
    fn noun_or_group(&mut self, token: Token<'de>) -> Result<TokenTree<'de>, Error> {
        match token.kind {
            TokenKind::Single(_) | TokenKind::Vector(_) => Ok(TokenTree::Noun(token)),
            TokenKind::LeftParen => self.parse_paren_body(token),
            _ => Err(self.unexpected(token, "expected a literal or '('")),
        }
    }

    /// Parse the inside of a parenthesis group; `open` is the `(` token.
    fn parse_paren_body(&mut self, open: Token<'de>) -> Result<TokenTree<'de>, Error> {
        let inner = self.parse_expr()?;
        match self.lexer.next().transpose()? {
            Some(Token {
                kind: TokenKind::RightParen,
                ..
            }) => Ok(inner),
            // `parse_expr` only stops at `)` or end of input
            _ => Err(self.syntax_error(span_of(open), "unterminated '('", "never closed")),
        }
    }

    fn next_token(&mut self) -> Result<Token<'de>, Error> {
        let end = self.source.trim_end().len();
        self.lexer.next().transpose()?.ok_or_else(|| {
            self.syntax_error(
                (end, 0).into(),
                "unexpected end of input",
                "expected an operand",
            )
        })
    }

    fn unexpected(&self, t: Token<'_>, label: &str) -> Error {
        self.syntax_error(span_of(t), &format!("unexpected '{}'", t.origin), label)
    }

    fn syntax_error(&self, span: SourceSpan, msg: &str, label: &str) -> Error {
        miette::miette!(labels = vec![LabeledSpan::at(span, label)], "{msg}")
            .with_source_code(self.source.to_string())
    }
}

fn span_of(t: Token<'_>) -> SourceSpan {
    (t.offset, t.origin.len()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unary_minus_does_not_stack() {
        // matches q, which rejects `--2` ('- ) rather than treating it as double negation
        assert!(Parser::new("--5").parse().is_err());
    }

    #[test]
    fn stray_close_paren_is_an_error() {
        assert!(Parser::new("1 2 3)").parse().is_err());
        assert!(Parser::new("(1+2))").parse().is_err());
    }

    #[test]
    fn unterminated_paren_is_an_error() {
        assert!(Parser::new("(2+3").parse().is_err());
    }

    #[test]
    fn missing_operand_is_an_error() {
        assert!(Parser::new("1+").parse().is_err());
        assert!(Parser::new("-").parse().is_err());
    }
}
