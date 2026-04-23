use std::fmt;
use displaydoc::Display;

/// Token的位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub row: usize,
    pub col: usize,
}

impl Span {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

/// Token类型
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum TokenKind {
    // Define different token kinds here

    // 关键字
    /// IF
    If,
    /// ELSE
    Else,
    /// WHILE
    While,
    /// RETURN
    Return,
    /// VOID
    Void,
    /// INT
    Int,

    // 标识符
    /// ID({0})
    Identifier(String),
    
    // 字面量
    /// NUM({0})
    Number(i64),
    /// STR({0})
    String(String),

    // 运算符
    /// PLUS
    Plus,
    /// MINUS
    Minus,
    /// STAR
    Star,
    /// SLASH
    Slash,
    /// EQ
    Eq,
    /// NE
    Ne,
    /// ASSIGN
    Assign,
    /// LT
    Lt,
    /// GT
    Gt,
    /// LTE
    Lte,
    /// GTE
    Gte,
    
    // 分隔符
    /// SEMICOLON
    Semicolon,
    /// COMMA
    Comma,
    /// LPAREN
    LParen,
    /// RPAREN
    RParen,
    /// LBRACE
    LBrace,
    /// RBRACE
    RBrace,
    /// LBRACKET
    LBracket,
    /// RBRACKET
    RBracket,

    // 其他
    /// EOF
    EOF, // End of file

}

impl TokenKind {
    pub fn lookup_keyword(s: &str) -> TokenKind {
        match s {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "void" => TokenKind::Void,
            "int" => TokenKind::Int,
            _ => TokenKind::Identifier(s.to_string()),
        }
    }
}

/// 一个完整的 Token = 种类 + 位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:>3}:{:<3}] {}", self.span.row, self.span.col, self.kind)
    }
}

impl Token {
    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::EOF
    }
}