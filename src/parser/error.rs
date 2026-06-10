use std::error::Error;

use crate::lexer::token::{Span, TokenKind};
use displaydoc::Display;

/// 解析错误的种类
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum ParseErrorKind {
    /// Expected token {expected:?}, found {found:?}
    ExpectedToken {
        expected: TokenKind,
        found: TokenKind,
    },
    /// Expected identifier, found {found:?}
    ExpectedIdentifier { found: TokenKind },
    /// Expected number, found {found:?}
    ExpectedNumber { found: TokenKind },
    /// Expected string, found {found:?}
    ExpectedString { found: TokenKind },
    /// Expected type specifier, found {found:?}
    ExpectedTypeSpec { found: TokenKind },
    /// Expected '(', identifier, number, or string, found {found:?}
    ExpectedFactor { found: TokenKind },
    /// Expected ';', '[', or '(', found {found:?}
    ExpectedDeclTail { found: TokenKind },
    /// Array size {value} too large, max is {max}
    ArraySizeTooLarge { value: i64, max: u32 },
    /// Expected '}}'
    MissingRBrace,
}

/// A parse error.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Parse Error at {}] {}", self.span, self.kind)
    }
}

impl Error for ParseError {}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}
