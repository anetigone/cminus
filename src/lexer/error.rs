use std::error::Error;
use std::fmt;

use super::token::Span;

/// 词法分析错误
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn new(message: String, span: Span) -> Self {
        Self { message, span }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LexError at {}: {}", self.span, self.message)
    }
}

impl Error for LexError {}
