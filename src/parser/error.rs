use std::error::Error;

use crate::lexer::token::Span;

/// A parse error.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Parse Error at {}] {}", self.span, self.message)
    }
}

impl Error for ParseError {}

impl ParseError {
    pub fn new(message: String, span: Span) -> Self {
        Self { message, span }
    }
}