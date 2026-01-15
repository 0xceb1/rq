use miette::{Diagnostic, Error, SourceSpan};
use std::fmt;
use thiserror::Error;

const NUMERICAL_SUFFIXES: &[char] = &['b', 'h', 'i', 'j', 'e', 'f', 'p', 'n', 'm', 'd', 'u', 'v'];

#[derive(Diagnostic, Debug, Error)]
#[error("Unexpected token '{token}'")]
pub struct SingleTokenError {
    #[source_code]
    src: String,

    pub token: char,

    #[label = "this input character"]
    err_span: SourceSpan,

    #[help]
    help: Option<String>,
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

// #[derive(Debug, Clone, Copy, PartialEq, Display)]
// pub enum Literal<'de> {
//     Bool(bool),
//     Char(AsciiChar),
//     Byte(u8),
//     Symbol(Symbol),
//     #[display("{}", Literal::unescape(_0))]
//     QString(&'de str),
//     Short(i16),
//     Int(i32),
//     Long(i64),
//     Real(f32),
//     Float(f64),
//     Date(Date),
//     Month(Month),
//     Minute(Minute),
//     Second(Second),
//     Timespan(Timespan),
//     Timestamp(Timestamp),
//     #[display("nil")]
//     Nil,
// }

// impl Literal<'_> {
//     pub fn unescape<'de>(s: &'de str) -> Cow<'de, str> {
//         // TODO: impl escaping
//         s.strip_prefix('"')
//             .and_then(|s| s.strip_suffix('"'))
//             .map(Cow::Borrowed)
//             .unwrap_or(Cow::Borrowed(s))
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'de> {
    pub origin: &'de str,
    pub offset: usize,
    pub kind: TokenKind,
    // pub literal: Literal<'de>,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kind={:?}, origin={}", self.kind, self.origin)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum AssignThrough {
    Dot,        // .:
    At,         // @:
    Dollar,     // $:
    Bang,       // !:
    Query,      // ?:
    Plus,       // +:
    Minus,      // -:
    Star,       // *:
    Percent,    // %:
    Equal,      // =:
    Tilde,      // ~:
    Less,       // <:
    Greater,    // >:
    Pipe,       // |:
    Ampersand,  // &:
    Hash,       // #:
    Underscore, // _:
    Caret,      // ^:
    Comma,      // ,:
}

impl AssignThrough {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '.' => Some(Self::Dot),
            '@' => Some(Self::At),
            '$' => Some(Self::Dollar),
            '!' => Some(Self::Bang),
            '?' => Some(Self::Query),
            '+' => Some(Self::Plus),
            '-' => Some(Self::Minus),
            '*' => Some(Self::Star),
            '%' => Some(Self::Percent),
            '=' => Some(Self::Equal),
            '~' => Some(Self::Tilde),
            '<' => Some(Self::Less),
            '>' => Some(Self::Greater),
            '|' => Some(Self::Pipe),
            '&' => Some(Self::Ampersand),
            '#' => Some(Self::Hash),
            '_' => Some(Self::Underscore),
            '^' => Some(Self::Caret),
            ',' => Some(Self::Comma),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Numerical {
    Boolean,
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
    Percent,    // %
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
    Colon,          // :
    ColonColon,     // ::
    Quote,          // '
    QuoteColon,     // ':
    SlashColon,     // /: each right
    BackslashColon, // \: each left
    AssignThrough(AssignThrough),

    // Literals.
    Identifier,
    // Guid(Uuid),
    Byte,
    Char,
    Symbol,
    Typed,
    Untyped,

    // Non-atomic types
    QString,
    ByteVec,

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
        let mut is_previous_whitespace = false;
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
                Symbol,
                String,
                Number(u32),
                Identifier,
                // IfEqualElse(TokenKind, TokenKind), // >=, <=
                // IfColonElse(TokenKind, TokenKind),
            }

            let mut just = |kind: TokenKind| {
                is_previous_whitespace = false;
                Some(Ok(Token {
                    kind,
                    offset: c_at,
                    origin: c_str,
                }))
            };

            let started = match c {
                '(' => return just(TokenKind::LeftParen),
                ')' => return just(TokenKind::RightParen),
                '{' => return just(TokenKind::LeftBrace),
                '}' => return just(TokenKind::RightBrace),
                '[' => return just(TokenKind::LeftBracket),
                ']' => return just(TokenKind::RightBracket),
                ';' => return just(TokenKind::Semicolon),
                c @ ('.' | '@' | '$' | '!' | '?' | '+' | '-' | '*' | '%' | '=' | '~' | '<'
                | '>' | '|' | '&' | '#' | '_' | '^' | ',') => {
                    if self.rest.starts_with(':') {
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        let assign_through = AssignThrough::from_char(c).unwrap();
                        return Some(Ok(Token {
                            origin: &c_onwards[..2],
                            offset: c_at,
                            kind: TokenKind::AssignThrough(assign_through),
                        }));
                    }
                    let kind = match c {
                        '.' => TokenKind::Dot,
                        '@' => TokenKind::At,
                        '$' => TokenKind::Dollar,
                        '!' => TokenKind::Bang,
                        '?' => TokenKind::Query,
                        '+' => TokenKind::Plus,
                        '-' => TokenKind::Minus,
                        '*' => TokenKind::Star,
                        '%' => TokenKind::Percent,
                        '=' => TokenKind::Equal,
                        '~' => TokenKind::Tilde,
                        '<' => TokenKind::Less,
                        '>' => TokenKind::Greater,
                        '|' => TokenKind::Pipe,
                        '&' => TokenKind::Ampersand,
                        '#' => TokenKind::Hash,
                        '_' => TokenKind::Underscore,
                        '^' => TokenKind::Caret,
                        ',' => TokenKind::Comma,
                        _ => unreachable!(),
                    };
                    return just(kind);
                }
                '`' => Started::Symbol,
                '"' => Started::String,
                '/' => Started::Slash,
                'a'..='z' | 'A'..='Z' => Started::Identifier,
                n @ '0'..='9' => Started::Number(n.to_digit(10).unwrap()),
                c if c.is_whitespace() => {
                    is_previous_whitespace = true;
                    continue;
                }
                c => {
                    return Some(Err(SingleTokenError {
                        src: self.whole.to_string(),
                        token: c,
                        err_span: SourceSpan::from(self.byte - c.len_utf8()..self.byte),
                        help: None,
                    }
                    .into()));
                }
            };
            break match started {
                Started::Symbol => {
                    // WARN: when backtick is followed by some built-in operators, the behavior is bizarre!
                    // This is not supported in our toy interpreter for now, and is unlikely to be supported in the future.
                    // Examples:
                    // q)x:`*
                    // q)x
                    // *[`]
                    // q)type x
                    // 104h
                    // q)x:`*
                    // q)x
                    // *[`]
                    // q)type x
                    // 104h
                    // q)x:`\
                    // q)x
                    // `\
                    // q)type x
                    // 108h
                    // ```
                    let end = self
                        .rest
                        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != ':')
                        .unwrap_or(self.rest.len());

                    if self.rest.starts_with('_') {
                        // a symbol literal can contains _ but can't start with
                        let c = self.rest.chars().next().unwrap();
                        let err = SingleTokenError {
                            src: self.whole.to_string(),
                            token: c,
                            err_span: SourceSpan::from(self.byte..self.byte + c.len_utf8()),
                            help: Some("a symbol starting with _ is ambiguous in q".to_string()),
                        };
                        self.byte += self.rest.len();
                        self.rest = &self.rest[self.rest.len()..];
                        return Some(Err(err.into()));
                    }

                    //TODO: if a symbol starts with :, it will be parsed as a path, therefore / is allowed

                    let literal = &c_onwards[..end + 1];
                    self.byte += end;
                    self.rest = &self.rest[end..];
                    Some(Ok(Token {
                        origin: literal,
                        offset: c_at,
                        kind: TokenKind::Symbol,
                    }))
                }
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
                        let token_kind = if end == 1 {
                            TokenKind::Char
                        } else {
                            TokenKind::QString
                        };
                        Some(Ok(Token {
                            origin: literal,
                            offset: c_at,
                            kind: token_kind,
                        }))
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

                Started::Slash => {
                    // TODO:
                    // 1. a slash is also valid for a comment when it's at the beginning of one file
                    // 2. support multi-line comments
                    // 3. parse each right
                    if is_previous_whitespace {
                        let line_end = self.rest.find('\n').unwrap_or(self.rest.len());
                        let comment_closed = self.rest.find('\\');

                        let offset = if let Some(comment_closed) = comment_closed {
                            comment_closed + 1
                        } else {
                            line_end
                        };
                        self.byte += offset;
                        self.rest = &self.rest[offset..];
                        continue;
                    } else {
                        Some(Ok(Token {
                            origin: c_str,
                            offset: c_at,
                            kind: TokenKind::Slash,
                        }))
                    }
                }
                Started::Identifier => {
                    let first_non_ident = c_onwards
                        .find(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
                        .unwrap_or(c_onwards.len());

                    let literal = &c_onwards[..first_non_ident];
                    let extra_bytes = literal.len() - c.len_utf8();
                    self.byte += extra_bytes;
                    self.rest = &self.rest[extra_bytes..];

                    Some(Ok(Token {
                        origin: literal,
                        offset: c_at,
                        kind: TokenKind::Identifier,
                    }))
                }
                Started::Number(n) => {
                    if n == 0 && self.rest.starts_with('x') {
                        let after_0x = &c_onwards[2..]; // skip "0x"
                        let hex_len = after_0x
                            .find(|c: char| !c.is_ascii_hexdigit())
                            .unwrap_or(after_0x.len());
                        let first_non_digit = 2 + hex_len;
                        let literal = &c_onwards[..first_non_digit];

                        let extra_bytes = literal.len() - c.len_utf8();
                        self.byte += extra_bytes;
                        self.rest = &self.rest[extra_bytes..];
                        let token_kind = if literal.len() <= 4 {
                            TokenKind::Byte
                        } else {
                            TokenKind::ByteVec
                        };
                        return Some(Ok(Token {
                            origin: literal,
                            offset: c_at,
                            kind: token_kind,
                        }));
                    // TODO: a leading D is a valid timespan literal!
                    } else {
                        // Numerical literal patterns:
                        // short     : 42h
                        // int       : 42i
                        // long      : 42 42j
                        // real      : 4.2e
                        // float     : 4.2 4.2f
                        // timestamp : 2013.02.06D12:34:56.123456789 2013.02.06D12:34:56.123456789p
                        //             12:34:56.123456789p -> 2000.01.01D12:34:56.123456789
                        //             2013.02p            -> 2000.01.01D20:13:00.020000000(coerced to long)
                        //             2013.02.06p         -> 2013.02.06D00:00:00.000000000
                        //             12:34p              -> 2000.01.01D12:34:00.000000000
                        //             12:34:56p           -> 2000.01.01D12:34:56.000000000
                        // timespan  : 12:34:56.123456789 12:34:56.123456789n
                        //             2013.02.06D12:34:56.123456789n -> 4785D12:34:56.123456789
                        //             2013.02n                       -> 0D20:13:00.02000000 (coerced to long)
                        //             2013.02.06n                    -> 4785D00:00:00.000000000
                        //             12:34n                         -> 0D12:34:00.000000000
                        //             12:34:56n                      -> 0D12:34:56.00000000
                        // month     : 2013.02m
                        //             2013.02.06m -> 2013.06m (coerced to long)
                        // date      : 2013.02.06 2013.02.06d
                        // minute    : 12:34 12:34u
                        // second    : 12:34:56 12:34:56v 12:34v 12v
                        let mut first_non_digit = c_onwards
                            .find(|c| {
                                !matches!(c, '.' | ':' | 'D' | 'N' | 'W' | 'n' | 'w' | '0'..='9')
                            })
                            .unwrap_or(c_onwards.len());
                        let suffix = c_onwards[first_non_digit..].chars().next().unwrap_or('\0');
                        let mut token_kind = TokenKind::Untyped;
                        if NUMERICAL_SUFFIXES.contains(&suffix) {
                            first_non_digit += 1;
                            token_kind = TokenKind::Typed;
                        }
                        let literal = &c_onwards[..first_non_digit];
                        let extra_bytes = literal.len() - c.len_utf8();
                        self.byte += extra_bytes;
                        self.rest = &self.rest[extra_bytes..];

                        Some(Ok(Token {
                            origin: literal,
                            offset: c_at,
                            kind: token_kind,
                        }))
                    }
                }
                _ => todo!(),
            };
        } // loop
    }
}
