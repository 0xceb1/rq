use crate::qtype::chrono::*;
use crate::qtype::symbol::Symbol;
use derive_more::Display;
use miette::{Diagnostic, Error, LabeledSpan, SourceSpan};
use std::ascii::Char as AsciiChar;
use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

#[derive(Diagnostic, Debug, Error)]
#[error("Unexpected token '{token}'")]
pub struct SingleTokenError {
    #[source_code]
    src: String,

    pub token: char,

    #[label = "this input character"]
    err_span: SourceSpan,
}

impl SingleTokenError {
    pub fn line(&self) -> usize {
        let until_unrecongized = &self.src[..=self.err_span.offset()];
        until_unrecongized.lines().count()
    }
}

#[derive(Diagnostic, Debug, Error)]
#[error("Unterminated string")]
pub struct StringTerminationError {
    #[source_code]
    src: String,

    #[label = "this string literal"]
    err_span: SourceSpan,
}

impl StringTerminationError {
    pub fn line(&self) -> usize {
        let until_unrecongized = &self.src[..=self.err_span.offset()];
        until_unrecongized.lines().count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Display)]
pub enum Literal<'de> {
    Bool(bool),
    Char(AsciiChar),
    Byte(u8),
    Symbol(Symbol),
    #[display("{}", Literal::unescape(_0))]
    QString(&'de str),
    Short(i16),
    Int(i32),
    Long(i64),
    Real(f32),
    Float(f64),
    Date(Date),
    Month(Month),
    Minute(Minute),
    Second(Second),
    Timespan(Timespan),
    Timestamp(Timestamp),
    #[display("nil")]
    Nil,
}

impl Literal<'_> {
    pub fn unescape<'de>(s: &'de str) -> Cow<'de, str> {
        // TODO: impl escaping
        s.strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .map(Cow::Borrowed)
            .unwrap_or(Cow::Borrowed(s))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'de> {
    pub origin: &'de str,
    pub offset: usize,
    pub kind: TokenKind,
    pub literal: Literal<'de>,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "kind={:?}, origin={}, literal={}",
            self.kind, self.origin, self.literal
        )
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    BackSlash,
    Star,
    BackTick,   // `
    Hash,       // #
    At,         // @
    Tilde,      // ~
    Pipe,       // |
    Ampersand,  // &
    Caret,      // ^
    Query,      // ?
    Dollar,     // $
    Underscore, // _
    Bang,

    // One or two character tokens.
    NotEqual, // <>
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Colon,      // :
    ColonColon, // ::
    Quote,      // '
    QuoteColon, // ':

    // Literals.
    Identifier,
    // Guid(Uuid),
    Byte,
    Char,
    Symbol,
    Short,
    Int,
    Long,
    Real,
    Float,
    Date, // 2000.01.01 = 0
    Month,
    Minute,
    Second,
    // Time,      // hh:mm:ss.uuu
    Timespan,  // hh:mm:ss.nnnnnnnnn
    Timestamp, // YYYY.MM.DDDhh:mm:ss.nnnnnnnn

    // Non-atomic types
    QString,

    // // Keywords.
    // And,
    // Class,
    // Else,
    // False,
    // Fun,
    // For,
    // If,
    // Nil,
    // Or,
    // Print,
    // Return,
    // Super,
    // This,
    // True,
    // Var,
    // While,
    // Lambda,
    Eof,
}

pub struct Lexer<'de> {
    whole: &'de str,
    rest: &'de str,
    byte: usize,
    peeked: Option<Result<Token<'de>, miette::Error>>,
}

impl<'de> Lexer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            rest: input,
            byte: 0,
            peeked: None,
        }
    }
}

impl<'de> Lexer<'de> {
    pub fn peek(&mut self) -> Option<&Result<Token<'de>, miette::Error>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }

        self.peeked = self.next();
        self.peeked.as_ref()
    }
}

impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, Error>;

    /// Once the iterator returns `Err`, it will only return `None`.
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.peeked.take() {
            return Some(next);
        }

        loop {
            let mut chars = self.rest.chars(); // iterator to unparsed chars
            let c = chars.next()?; // current char
            let c_at = self.byte; // byte offset where current char starts
            let c_str = &self.rest[..c.len_utf8()]; // string slice containing single char c
            let c_onwards = self.rest; // remaining chars starting from c
            self.rest = chars.as_str();
            self.byte += c.len_utf8();

            enum Started {
                Slash,
                String,
                Number,
                Identifier,
                // IfEqualElse(TokenKind, TokenKind), // >=, <=
                // IfColonElse(TokenKind, TokenKind),
            }

            let just = move |kind: TokenKind| {
                Some(Ok(Token {
                    kind,
                    offset: c_at,
                    origin: c_str,
                    literal: Literal::Nil,
                }))
            };

            let started = match c {
                '(' => return just(TokenKind::LeftParen),
                ')' => return just(TokenKind::RightParen),
                '{' => return just(TokenKind::LeftBrace),
                '}' => return just(TokenKind::RightBrace),
                '[' => return just(TokenKind::LeftBracket),
                ']' => return just(TokenKind::RightBracket),
                ',' => return just(TokenKind::Comma),
                '.' => return just(TokenKind::Dot),
                '-' => return just(TokenKind::Minus),
                '+' => return just(TokenKind::Plus),
                ';' => return just(TokenKind::Semicolon),
                '*' => return just(TokenKind::Star),
                '`' => return just(TokenKind::BackTick),
                '#' => return just(TokenKind::Hash),
                '@' => return just(TokenKind::At),
                '~' => return just(TokenKind::Tilde),
                '|' => return just(TokenKind::Pipe),
                '&' => return just(TokenKind::Ampersand),
                '^' => return just(TokenKind::Caret),
                '?' => return just(TokenKind::Query),
                '$' => return just(TokenKind::Dollar),
                '!' => return just(TokenKind::Bang),
                '"' => Started::String,
                c => {
                    return Some(Err(SingleTokenError {
                        src: self.whole.to_string(),
                        token: c,
                        err_span: SourceSpan::from(self.byte - c.len_utf8()..self.byte),
                    }
                    .into()));
                }
            };
            break match started {
                Started::String => {
                    let mut escaped = false;
                    let end = self.rest.bytes().position(|b| {
                        if escaped {
                            escaped = false;
                            false
                        } else if b == b'\\' {
                            escaped = true;
                            false
                        } else {
                            b == b'"'
                        }
                    });
                    if let Some(end) = end {
                        let literal = &c_onwards[..end + 1 + 1];
                        self.byte += end + 1;
                        self.rest = &self.rest[end + 1..];
                        if end == 1 {
                            Some(Ok(Token {
                                origin: literal,
                                offset: c_at,
                                kind: TokenKind::Char,
                                literal: Literal::Char(
                                    AsciiChar::from_u8(literal.as_bytes()[1]).unwrap(),
                                ),
                            }))
                        } else {
                            Some(Ok(Token {
                                origin: literal,
                                offset: c_at,
                                kind: TokenKind::QString,
                                literal: Literal::QString(literal),
                            }))
                        }
                    } else {
                        let err = StringTerminationError {
                            src: self.whole.to_string(),
                            err_span: SourceSpan::from(self.byte - c.len_utf8()..self.whole.len()),
                        };

                        // swallow the remainder of input as being a string
                        self.byte += self.rest.len();
                        self.rest = &self.rest[self.rest.len()..];

                        return Some(Err(err.into()));
                    }
                }
                _ => todo!(),
            };
        } // loop
    }
}
