pub mod token;
pub mod error;
#[cfg(test)]
mod tests;

use token::{Token, TokenKind, Span};
use error::LexError;

/// 词法分析器
pub struct Lexer {
    /// 源代码的字符数组
    source: Vec<char>,
    /// 当前读取位置
    pos: usize,
    /// 当前行号（从 1 开始）
    line: usize,
    /// 当前列号（从 1 开始）
    column: usize,
    /// 收集到的错误
    pub errors: Vec<LexError>,
}

impl Lexer {
    /// 创建一个新的词法分析器
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }
}

impl Lexer {
    // 底层字符操作

    /// 获取当前字符（不消耗）
    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).cloned()
    }

    /// 获取下一个字符（不消耗）
    #[allow(dead_code)]
    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).cloned()
    }

    /// 消耗当前字符并前进
    fn advance(&mut self) -> Option<char> {
        let current_char = self.peek()?;
        self.pos += 1;
        if current_char == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(current_char)
    }
    
    /// 记录当前位置
    fn current_span(&self) -> Span {
        Span::new(self.line, self.column)
    }

    /// 跳过空白
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 跳过单行注释
    /// 以 `//` 开头的注释
    /// `//`已被消耗
    pub fn skip_line_comment(&mut self) {
        while let Some(ch) = self.advance() {
            if ch == '\n' {
                return;
            }
        }
    }

    /// 跳过多行注释
    /// 以 `/*` 开头，以 `*/` 结尾的注释
    /// `/*`已被消耗
    pub fn skip_block_comment(&mut self, start_span: Span) {
        while let Some(ch) = self.advance() {
            if ch == '*' && self.peek() == Some('/') {
                self.advance(); // 跳过 '/'
                return;
            }
        }

        self.errors.push(LexError::new("Unterminated block comment".to_string(), start_span));
    }

    /// 扫描标识符
    pub fn scan_identifier(&mut self) -> Option<Token> {
        let start_span = self.current_span();
        let mut identifier = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            }
            else {
                break;
            }
        }

        let kind = TokenKind::lookup_keyword(&identifier);

        Some(Token::new(kind, start_span))
    }

    /// 扫描数字
    pub fn scan_number(&mut self) -> Option<Token> {
        let start_span = self.current_span();
        let mut number = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                number.push(ch);
                self.advance();
            }
            else {
                break;
            }
        }

        let number: i64 = number.parse().unwrap_or(0);

        Some(Token::new(TokenKind::Number(number), start_span))
    }

    /// 扫描字符串
    pub fn scan_string(&mut self) -> Option<Token> {
        let start_span = self.current_span();
        let mut string = String::new();

        self.advance(); // 跳过开始引号

        while let Some(ch) = self.advance() {
            match ch {
                '"' => return Some(Token::new(TokenKind::String(string), start_span)),
                '\\' => {
                    if let Some(escaped) = self.scan_escape_char(&start_span) {
                        string.push(escaped);
                    }
                }
                _ => string.push(ch),
            }
        }

        self.errors.push(LexError::new("Unterminated string".to_string(), start_span));
        None
    }

    /// 扫描转义字符
    fn scan_escape_char(&mut self, start_span: &Span) -> Option<char> {
        match self.advance() {
            Some('n') => Some('\n'),
            Some('t') => Some('\t'),
            Some('\\') => Some('\\'),
            Some('"') => Some('"'),
            Some(ch) => {
                self.errors.push(LexError::new(format!("Unknown escape character: \\{}", ch), start_span.clone()));
                Some(ch) // 仍然返回原字符，继续解析
            }
            None => {
                self.errors.push(LexError::new("Unterminated string".to_string(), start_span.clone()));
                None
            }
        }
    }

    /// 获取下一个token
    pub fn next_token(&mut self) -> Option<Token> {
        //跳过空白
        self.skip_whitespace();
        //起始位置
        let span = self.current_span();
        //检查是否结束
        let ch = match self.peek() {
            Some(ch) => ch,
            None => return Some(Token::new(TokenKind::EOF, span)),
        };

        //根据当前字符返回token
        // ---- 标识符&关键字 ----
        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.scan_identifier();
        }

        // ---- 数字 ----
        if ch.is_digit(10) {
            return self.scan_number();
        }

        // ---- 字符串 ----
        if ch == '"' {
            match self.scan_string() {
                Some(token) => return Some(token),
                None => {
                    // 字符串解析失败（如未终止），继续解析下一个token
                    return self.next_token();
                }
            }
        }

        // --- 运算符&分隔符 ---
        self.advance();//消耗当前字符

        match ch {
            '+' => Some(Token::new(TokenKind::Plus, span)),
            '-' => Some(Token::new(TokenKind::Minus, span)),
            '*' => Some(Token::new(TokenKind::Star, span)),
            '/' => {
                //检查是否是注释
                //如果是注释，则跳过注释，并返回下一个token
                if self.peek() == Some('/') {
                    self.advance(); //消耗 '/'
                    self.skip_line_comment();
                    self.next_token()
                }
                else if self.peek() == Some('*') {
                    self.advance(); //消耗 '*'
                    self.skip_block_comment(span);
                    self.next_token()
                }
                else {
                    Some(Token::new(TokenKind::Slash, span))
                }
            },
            '=' => {
                if self.peek() == Some('=') {
                    self.advance(); //消耗 '='
                    Some(Token::new(TokenKind::Eq, span))
                }
                else {
                    Some(Token::new(TokenKind::Assign, span))
                }
            },
            '<' => {
                if self.peek() == Some('=') {
                    self.advance(); //消耗 '='
                    Some(Token::new(TokenKind::Le, span))
                }
                else {
                    Some(Token::new(TokenKind::Lt, span))
                }
            },
            '>' => {
                if self.peek() == Some('=') {
                    self.advance(); //消耗 '='
                    Some(Token::new(TokenKind::Ge, span))
                }
                else {
                    Some(Token::new(TokenKind::Gt, span))
                }
            },
            '!' => {
                if self.peek() == Some('=') {
                    self.advance(); //消耗 '='
                    Some(Token::new(TokenKind::Ne, span))
                }
                else {
                    self.errors.push(LexError::new(
                        format!("Unexpected character: '{}',
                        expected '!='", ch),
                        span.clone()
                    ));
                    self.next_token() //返回下一个token
                }
            }
            ';' => Some(Token::new(TokenKind::Semicolon, span)),
            ',' => Some(Token::new(TokenKind::Comma, span)),
            '(' => Some(Token::new(TokenKind::LParen, span)),
            ')' => Some(Token::new(TokenKind::RParen, span)),
            '{' => Some(Token::new(TokenKind::LBrace, span)),
            '}' => Some(Token::new(TokenKind::RBrace, span)),
            '[' => Some(Token::new(TokenKind::LBracket, span)),
            ']' => Some(Token::new(TokenKind::RBracket, span)),
            _ => {
                self.errors.push(LexError::new(
                    format!("Unexpected character: '{}'", ch),
                    span
                ));
                self.next_token() //返回下一个token
            }
        }
    }


    /// 获取所有token（包含最后的EOF）
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            let is_eof = token.is_eof();
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}