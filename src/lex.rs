use miette::{Diagnostic, Error, SourceSpan};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Numerical {
    Boolean,
    Short,
    Int,
    Long,
    Real,
    Float,
    Char,
    Date,
    Month,
    Minute,
    Second,
    Timespan,
    Timestamp,
}

impl Numerical {
    // Atomic literal patterns:
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
    pub fn from_suffix(c: char) -> Option<Self> {
        match c {
            'b' => Some(Self::Boolean),
            'h' => Some(Self::Short),
            'i' => Some(Self::Int),
            'j' => Some(Self::Long),
            'e' => Some(Self::Real),
            'f' => Some(Self::Float),
            'd' => Some(Self::Date),
            'm' => Some(Self::Month),
            'u' => Some(Self::Minute),
            'v' => Some(Self::Second),
            'n' => Some(Self::Timespan),
            'p' => Some(Self::Timestamp),
            _ => None,
        }
    }

    /// Parse untyped literals. Valid forms:
    /// - long      : `42` (digits only)
    /// - float     : `4.2` (digits.digits)
    /// - date      : `2013.02.06` (YYYY.MM.DD)
    /// - timestamp : `2013.02.06D12:34:56.123456789`
    /// - timespan  : `12:34:56.123456789` or `0D12:34:56.123456789`
    /// - minute    : `12:34` (HH:MM)
    /// - second    : `12:34:56` (HH:MM:SS) `12:34.123` (parsed to 12:34:00.123)
    ///
    /// These patterns are rejected while valid in q:
    /// - HH:MM.xxx
    pub fn parse_untyped(
        origin: &str,
        offset: usize,
        src: &str,
    ) -> Result<Self, InvalidLiteralError> {
        let has_d = origin.contains('D');
        let colon_count = origin.matches(':').count();
        let dot_count = origin.matches('.').count();

        let result = if has_d {
            let before_d = origin.split('D').next().unwrap_or("");
            if before_d.contains('.') {
                Some(Self::Timestamp)
            } else {
                Some(Self::Timespan)
            }
        } else if colon_count > 0 {
            let after_last_colon = origin.rsplit(':').next().unwrap_or("");
            let has_fractional = after_last_colon.contains('.');

            match (colon_count, has_fractional) {
                (1, false) => Some(Self::Minute),
                (1, true) => {
                    return Err(InvalidLiteralError {
                        src: src.to_string(),
                        literal: origin.to_string(),
                        err_span: SourceSpan::from(offset..offset + origin.len()),
                        help: Some("HH:MM.xxx is ambiguous; use explicit patterns like HH:MM:SS.xxx or HH:MM:SS".into()),
                    });
                }
                (2, false) => Some(Self::Second),
                (2, true) => Some(Self::Timespan),
                _ => None,
            }
        } else if dot_count > 0 {
            if dot_count == 2 {
                Some(Self::Date)
            } else if dot_count == 1 {
                Some(Self::Float)
            } else {
                None
            }
        } else {
            Some(Self::Long)
        };

        result.ok_or_else(|| InvalidLiteralError {
            src: src.to_string(),
            literal: origin.to_string(),
            err_span: SourceSpan::from(offset..offset + origin.len()),
            help: None,
        })
    }
}

#[derive(Diagnostic, Debug, Error)]
#[error("Invalid literal '{literal}'")]
pub struct InvalidLiteralError {
    #[source_code]
    src: String,

    pub literal: String,

    #[label = "cannot determine type"]
    err_span: SourceSpan,

    #[help]
    help: Option<String>,
}

impl InvalidLiteralError {
    pub fn line(&self) -> usize {
        let until_unrecongized = &self.src[..=self.err_span.offset()];
        until_unrecongized.lines().count()
    }
}

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
    Colon,              // :
    ColonColon,         // ::
    Quote,              // '
    QuoteColon,         // ':
    SlashColon,         // /: each right
    BackslashColon,     // \: each left
    BackslashBackslash, // \\ abort
    AssignThrough(AssignThrough),

    // Literals.
    Identifier,
    Byte,
    Char,
    Symbol,
    Typed(Numerical),
    Untyped(Numerical),

    // Non-atomic types
    QString,
    ByteVec,

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

            let prev_whitespace =
                c_at == 0 || self.whole.as_bytes()[c_at - 1].is_ascii_whitespace();

            enum Started {
                Slash,
                Symbol,
                String,
                Number(u32),
                Identifier,
            }

            let just = |kind: TokenKind| {
                Some(Ok(Token {
                    kind,
                    offset: c_at,
                    origin: c_str,
                }))
            };

            let started = match c {
                // Match the next char
                '(' => return just(TokenKind::LeftParen),
                ')' => return just(TokenKind::RightParen),
                '{' => return just(TokenKind::LeftBrace),
                '}' => return just(TokenKind::RightBrace),
                '[' => return just(TokenKind::LeftBracket),
                ']' => return just(TokenKind::RightBracket),
                ';' => return just(TokenKind::Semicolon),
                c @ ('.' | '@' | '$' | '!' | '?' | '+' | '-' | '*' | '%' | '=' | '~' | '<'
                | '>' | '|' | '&' | '#' | '_' | '^' | ',') => {
                    // These chars can be assign through operator tokens
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
                    // Handle two-chars tokens: <>, >=, <=
                    let kind = match c {
                        '<' if self.rest.starts_with('>') => {
                            self.rest = &self.rest[1..];
                            self.byte += 1;
                            return Some(Ok(Token {
                                origin: &c_onwards[..2],
                                offset: c_at,
                                kind: TokenKind::NotEqual,
                            }));
                        }
                        '<' if self.rest.starts_with('=') => {
                            self.rest = &self.rest[1..];
                            self.byte += 1;
                            return Some(Ok(Token {
                                origin: &c_onwards[..2],
                                offset: c_at,
                                kind: TokenKind::LessEqual,
                            }));
                        }
                        '>' if self.rest.starts_with('=') => {
                            self.rest = &self.rest[1..];
                            self.byte += 1;
                            return Some(Ok(Token {
                                origin: &c_onwards[..2],
                                offset: c_at,
                                kind: TokenKind::GreaterEqual,
                            }));
                        }
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
                ':' => {
                    if self.rest.starts_with(':') {
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        return Some(Ok(Token {
                            origin: &c_onwards[..2],
                            offset: c_at,
                            kind: TokenKind::ColonColon,
                        }));
                    }
                    return just(TokenKind::Colon);
                }
                '\'' => {
                    if self.rest.starts_with(':') {
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        return Some(Ok(Token {
                            origin: &c_onwards[..2],
                            offset: c_at,
                            kind: TokenKind::QuoteColon,
                        }));
                    }
                    return just(TokenKind::Quote);
                }
                '\\' => {
                    if self.rest.starts_with(':') {
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        return Some(Ok(Token {
                            origin: &c_onwards[..2],
                            offset: c_at,
                            kind: TokenKind::BackslashColon,
                        }));
                    } else if self.rest.starts_with('\\') {
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        return Some(Ok(Token {
                            origin: &c_onwards[..2],
                            offset: c_at,
                            kind: TokenKind::BackslashBackslash,
                        }));
                    }
                    return just(TokenKind::BackSlash);
                }
                '`' => Started::Symbol,
                '"' => Started::String,
                '/' => Started::Slash,
                'a'..='z' | 'A'..='Z' => Started::Identifier,
                n @ '0'..='9' => Started::Number(n.to_digit(10).unwrap()),
                c if c.is_whitespace() => continue,
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
                    // TODO: empty space not allowed in symbol vector
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
                    if prev_whitespace {
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
                    } else if self.rest.starts_with(':') {
                        self.rest = &self.rest[1..];
                        self.byte += 1;
                        Some(Ok(Token {
                            origin: &c_onwards[..2],
                            offset: c_at,
                            kind: TokenKind::SlashColon,
                        }))
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
                        let first_non_digit = c_onwards
                            .find(|c| {
                                !matches!(c, '.' | ':' | 'D' | 'N' | 'W' | 'n' | 'w' | '0'..='9')
                            })
                            .unwrap_or(c_onwards.len());
                        let suffix = c_onwards[first_non_digit..].chars().next().unwrap_or('\0');

                        let (literal, token_kind) =
                            if let Some(atomic_type) = Numerical::from_suffix(suffix) {
                                let literal = &c_onwards[..first_non_digit + 1];
                                (literal, TokenKind::Typed(atomic_type))
                            } else {
                                let literal = &c_onwards[..first_non_digit];
                                let atomic_type =
                                    match Numerical::parse_untyped(literal, c_at, self.whole) {
                                        Ok(t) => t,
                                        Err(e) => return Some(Err(e.into())),
                                    };
                                (literal, TokenKind::Untyped(atomic_type))
                            };

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
            };
        } // loop
    }
}
