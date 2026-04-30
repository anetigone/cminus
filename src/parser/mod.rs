pub mod ast;
pub mod error;

use crate::lexer::token::*;
use ast::*;
use error::ParseError;

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
}

/// 基础操作
impl Parser{

    /// 获取当前token，不消耗
    pub fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.current)
            .map(|token| &token.kind)
            .unwrap_or(&TokenKind::EOF)
    }

    /// 获取当前token的位置
    pub fn current(&self) -> Span {
        self.tokens
            .get(self.current)
            .map(|token| token.span.clone())
            .unwrap_or(Span::new(0, 0))
    }

    /// 消耗当前token
    pub fn advance(&mut self) -> &Token {
        let token = self.tokens.get(self.current).unwrap();
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
        token
    }
}