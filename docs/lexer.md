

# C-Minus 编译器设计文档（Rust 实现）

## 目录

1. [C-Minus 语言规范](#1-c-minus-语言规范)
2. [编译器总体架构](#2-编译器总体架构)
3. [词法分析器详细设计](#3-词法分析器详细设计)
4. [完整 Rust 实现代码](#4-完整-rust-实现代码)
5. [测试](#5-测试)

---

## 1. C-Minus 语言规范

C-Minus 是 C 语言的一个子集（参考 Kenneth Louden《编译原理及实践》附录 A）。

### 1.1 词法元素

| 类别 | 内容 |
|------|------|
| **关键字** | `if`, `else`, `int`, `void`, `while`, `return` |
| **标识符** | `[a-zA-Z][a-zA-Z]*`（纯字母） |
| **数字** | `[0-9]+`（纯数字，无浮点） |
| **运算符** | `+` `-` `*` `/` `<` `<=` `>` `>=` `==` `!=` `=` |
| **分隔符** | `;` `,` `(` `)` `{` `}` `[` `]` |
| **注释** | `/* ... */`（C 风格块注释，可跨行，**不嵌套**） |
| **空白** | 空格、制表符、换行符（跳过） |

### 1.2 语法（BNF，供后续阶段参考）

```
program          → declaration-list
declaration-list → declaration { declaration }
declaration      → var-declaration | fun-declaration
var-declaration  → type-specifier ID ';'
                 | type-specifier ID '[' NUM ']' ';'
fun-declaration  → type-specifier ID '(' params ')' compound-stmt
type-specifier   → int | void
params           → param-list | void
param-list       → param { ',' param }
param            → type-specifier ID | type-specifier ID '[' ']'
compound-stmt    → '{' local-declarations statement-list '}'
statement        → expression-stmt | compound-stmt | selection-stmt
                 | iteration-stmt | return-stmt
selection-stmt   → if '(' expression ')' statement
                 | if '(' expression ')' statement else statement
iteration-stmt   → while '(' expression ')' statement
return-stmt      → return ';' | return expression ';'
expression       → var '=' expression | simple-expression
var              → ID | ID '[' expression ']'
simple-expression→ additive-expr relop additive-expr | additive-expr
relop            → '<=' | '<' | '>' | '>=' | '==' | '!='
additive-expr    → term { addop term }
addop            → '+' | '-'
term             → factor { mulop factor }
mulop            → '*' | '/'
factor           → '(' expression ')' | var | call | NUM
call             → ID '(' args ')'
args             → arg-list | ε
arg-list         → expression { ',' expression }
```

---

## 2. 编译器总体架构

```
┌──────────────────────────────────────────────────────────┐
│                    C-Minus 源代码                          │
└──────────────┬───────────────────────────────────────────┘
               │
               ▼
┌──────────────────────────┐
│   Phase 1: 词法分析器     │  源代码 → Token 流
│   (Lexer / Scanner)      │
└──────────────┬───────────┘
               │
               ▼
┌──────────────────────────┐
│   Phase 2: 语法分析器     │  Token 流 → AST
│   (Parser)               │  递归下降法
└──────────────┬───────────┘
               │
               ▼
┌──────────────────────────┐
│   Phase 3: 语义分析       │  AST → 标注 AST
│   (Semantic Analyzer)    │  符号表 + 类型检查
└──────────────┬───────────┘
               │
               ▼
┌──────────────────────────┐
│   Phase 4: 代码生成       │  AST → 目标代码
│   (Code Generator)       │  TM 虚拟机 / LLVM IR
└──────────────────────────┘
```

### 2.1 项目目录结构

```
cminus-compiler/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口
│   ├── lexer/
│   │   ├── mod.rs           # 词法分析器模块
│   │   └── token.rs         # Token 定义
│   ├── parser/              # (Phase 2)
│   │   ├── mod.rs
│   │   └── ast.rs
│   ├── semantic/            # (Phase 3)
│   │   └── mod.rs
│   └── codegen/             # (Phase 4)
│       └── mod.rs
└── tests/
    └── lexer_test.rs        # 词法分析器测试
```

---

## 3. 词法分析器详细设计

### 3.1 Token 类型定义

```
TokenKind:
  // 关键字
  If, Else, Int, Void, While, Return

  // 标识符和字面量
  Identifier(String)
  Number(i64)

  // 运算符
  Plus, Minus, Star, Slash
  Lt, Le, Gt, Ge, Eq, Ne
  Assign

  // 分隔符
  Semicolon, Comma
  LParen, RParen
  LBrace, RBrace
  LBracket, RBracket

  // 特殊
  Eof
```

### 3.2 状态转移图（DFA 核心）

```
                    ┌───────────┐
         ┌─letter──▶  IN_ID    │──not letter──▶ 产出 ID/Keyword
         │          └───┬──────┘
         │              │ letter
         │              └──┘
         │
         │          ┌───────────┐
         ├─digit───▶  IN_NUM   │──not digit───▶ 产出 NUM
         │          └───┬──────┘
         │              │ digit
START ───┤              └──┘
         │
         ├─ '/' ───▶ 检查下一个字符
         │            ├─ '*' → 进入 IN_COMMENT → 遇到 '*/' 退出
         │            └─ 其他 → 产出 Slash, 回退
         │
         ├─ '<' ───▶ 检查 '=' → Le 或 Lt
         ├─ '>' ───▶ 检查 '=' → Ge 或 Gt
         ├─ '=' ───▶ 检查 '=' → Eq 或 Assign
         ├─ '!' ───▶ 检查 '=' → Ne 或 Error
         │
         └─ 单字符 ─▶ 直接产出对应 Token
```

### 3.3 错误处理策略

- **未闭合注释**: 到达 EOF 时报错 `"Unterminated comment"`
- **非法字符**: 如 `!` 后不跟 `=`，报错 `"Unexpected character"`
- **错误恢复**: 跳过非法字符，继续扫描

---

## 4. 完整 Rust 实现代码

### `Cargo.toml`

```toml
[package]
name = "cminus-compiler"
version = "0.1.0"
edition = "2021"
```

### `src/lexer/token.rs`

```rust
use std::fmt;

/// Token 在源代码中的位置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Token 种类
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // ── 关键字 ──
    If,
    Else,
    Int,
    Void,
    While,
    Return,

    // ── 标识符 & 字面量 ──
    Identifier(String),
    Number(i64),

    // ── 运算符 ──
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=
    Eq,       // ==
    Ne,       // !=
    Assign,   // =

    // ── 分隔符 ──
    Semicolon,  // ;
    Comma,      // ,
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]

    // ── 特殊 ──
    Eof,
}

impl TokenKind {
    /// 尝试将标识符字符串匹配为关键字
    pub fn lookup_keyword(ident: &str) -> TokenKind {
        match ident {
            "if"     => TokenKind::If,
            "else"   => TokenKind::Else,
            "int"    => TokenKind::Int,
            "void"   => TokenKind::Void,
            "while"  => TokenKind::While,
            "return" => TokenKind::Return,
            _        => TokenKind::Identifier(ident.to_string()),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // 关键字
            TokenKind::If       => write!(f, "IF"),
            TokenKind::Else     => write!(f, "ELSE"),
            TokenKind::Int      => write!(f, "INT"),
            TokenKind::Void     => write!(f, "VOID"),
            TokenKind::While    => write!(f, "WHILE"),
            TokenKind::Return   => write!(f, "RETURN"),
            // 标识符 & 数字
            TokenKind::Identifier(s) => write!(f, "ID({s})"),
            TokenKind::Number(n)     => write!(f, "NUM({n})"),
            // 运算符
            TokenKind::Plus     => write!(f, "PLUS"),
            TokenKind::Minus    => write!(f, "MINUS"),
            TokenKind::Star     => write!(f, "STAR"),
            TokenKind::Slash    => write!(f, "SLASH"),
            TokenKind::Lt       => write!(f, "LT"),
            TokenKind::Le       => write!(f, "LE"),
            TokenKind::Gt       => write!(f, "GT"),
            TokenKind::Ge       => write!(f, "GE"),
            TokenKind::Eq       => write!(f, "EQ"),
            TokenKind::Ne       => write!(f, "NE"),
            TokenKind::Assign   => write!(f, "ASSIGN"),
            // 分隔符
            TokenKind::Semicolon => write!(f, "SEMI"),
            TokenKind::Comma     => write!(f, "COMMA"),
            TokenKind::LParen    => write!(f, "LPAREN"),
            TokenKind::RParen    => write!(f, "RPAREN"),
            TokenKind::LBrace    => write!(f, "LBRACE"),
            TokenKind::RBrace    => write!(f, "RBRACE"),
            TokenKind::LBracket  => write!(f, "LBRACKET"),
            TokenKind::RBracket  => write!(f, "RBRACKET"),
            // 特殊
            TokenKind::Eof       => write!(f, "EOF"),
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
        write!(f, "[{:>3}:{:<3}] {}", self.span.line, self.span.column, self.kind)
    }
}
```

### `src/lexer/mod.rs`

```rust
pub mod token;

use token::{Token, TokenKind, Span};

/// 词法分析错误
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error at {}] {}", self.span, self.message)
    }
}

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
    /// 构造新的词法分析器
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    // ═══════════════════════════════════════════
    //  底层字符操作
    // ═══════════════════════════════════════════

    /// 查看当前字符（不消耗）
    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    /// 查看下一个字符（不消耗）
    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    /// 消耗当前字符并前进
    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    /// 记录当前位置（用于 Token 起始位置）
    fn current_span(&self) -> Span {
        Span::new(self.line, self.column)
    }

    // ═══════════════════════════════════════════
    //  跳过空白和注释
    // ═══════════════════════════════════════════

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 跳过块注释 /* ... */
    /// 调用时已经消耗了 '/' 和 '*'
    fn skip_block_comment(&mut self, start_span: Span) {
        loop {
            match self.advance() {
                Some('*') => {
                    if self.peek() == Some('/') {
                        self.advance(); // 消耗 '/'
                        return;
                    }
                }
                None => {
                    // EOF 但注释未闭合
                    self.errors.push(LexError {
                        message: "Unterminated block comment".to_string(),
                        span: start_span,
                    });
                    return;
                }
                _ => {} // 继续
            }
        }
    }

    // ═══════════════════════════════════════════
    //  扫描各类 Token
    // ═══════════════════════════════════════════

    /// 扫描标识符或关键字
    fn scan_identifier(&mut self) -> Token {
        let span = self.current_span();
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = TokenKind::lookup_keyword(&ident);
        Token::new(kind, span)
    }

    /// 扫描数字字面量
    fn scan_number(&mut self) -> Token {
        let span = self.current_span();
        let mut num_str = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let value: i64 = num_str.parse().unwrap_or(0);
        Token::new(TokenKind::Number(value), span)
    }

    // ═══════════════════════════════════════════
    //  主扫描循环
    // ═══════════════════════════════════════════

    /// 获取下一个 Token
    pub fn next_token(&mut self) -> Token {
        // 跳过空白
        self.skip_whitespace();

        // 记录起始位置
        let span = self.current_span();

        // 检查是否到达末尾
        let ch = match self.peek() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, span),
        };

        // ─── 标识符 / 关键字 ───
        if ch.is_ascii_alphabetic() {
            return self.scan_identifier();
        }

        // ─── 数字 ───
        if ch.is_ascii_digit() {
            return self.scan_number();
        }

        // ─── 运算符 & 分隔符 ───
        self.advance(); // 消耗第一个字符

        match ch {
            '+' => Token::new(TokenKind::Plus, span),
            '-' => Token::new(TokenKind::Minus, span),
            '*' => Token::new(TokenKind::Star, span),
            '/' => {
                // 可能是注释
                if self.peek() == Some('*') {
                    self.advance(); // 消耗 '*'
                    self.skip_block_comment(span);
                    // 注释跳过后，递归获取下一个 token
                    self.next_token()
                } else {
                    Token::new(TokenKind::Slash, span)
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Le, span)
                } else {
                    Token::new(TokenKind::Lt, span)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Ge, span)
                } else {
                    Token::new(TokenKind::Gt, span)
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Eq, span)
                } else {
                    Token::new(TokenKind::Assign, span)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Ne, span)
                } else {
                    self.errors.push(LexError {
                        message: format!("Unexpected character: '!'  (expected '!=' )"),
                        span: span.clone(),
                    });
                    // 错误恢复：跳过，继续扫描
                    self.next_token()
                }
            }
            ';' => Token::new(TokenKind::Semicolon, span),
            ',' => Token::new(TokenKind::Comma, span),
            '(' => Token::new(TokenKind::LParen, span),
            ')' => Token::new(TokenKind::RParen, span),
            '{' => Token::new(TokenKind::LBrace, span),
            '}' => Token::new(TokenKind::RBrace, span),
            '[' => Token::new(TokenKind::LBracket, span),
            ']' => Token::new(TokenKind::RBracket, span),
            _ => {
                self.errors.push(LexError {
                    message: format!("Unexpected character: '{ch}'"),
                    span: span.clone(),
                });
                // 错误恢复：跳过非法字符，继续
                self.next_token()
            }
        }
    }

    /// 一次性扫描所有 Token（包括最终的 EOF）
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}
```

### `src/main.rs`

```rust
mod lexer;

use lexer::Lexer;

fn main() {
    let source = r#"
int gcd(int u, int v){
    if(v==0) return u;
    else return gcd(v, u-u/v*v);
}
"#;

    println!("╔══════════════════════════════════════════╗");
    println!("║     C-Minus Lexer — Token Output         ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Source:                                  ║");
    println!("╠══════════════════════════════════════════╣");

    // 打印源代码（带行号）
    for (i, line) in source.lines().enumerate() {
        if !line.is_empty() {
            println!("║  {:<3} │ {:<36}║", i + 1, line);
        }
    }

    println!("╠══════════════════════════════════════════╣");
    println!("║  Tokens:                                  ║");
    println!("╠══════════════════════════════════════════╣");

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // 打印每个 Token
    println!("║ {:<8} {:<33}║", "Loc", "Token");
    println!("║ ──────── ─────────────────────────────── ║");
    for token in &tokens {
        println!("║ {:>3}:{:<3}   {:<33}║", 
            token.span.line, token.span.column, 
            token.kind.to_string());
    }

    println!("╠══════════════════════════════════════════╣");

    // 打印错误
    if lexer.errors.is_empty() {
        println!("║  ✓ No lexical errors                     ║");
    } else {
        for err in &lexer.errors {
            println!("║  ✗ {:<38}║", err);
        }
    }

    println!("╚══════════════════════════════════════════╝");

    // 统计
    let total = tokens.len();
    println!("\nTotal tokens: {} (including EOF)", total);
}
```

### `tests/lexer_test.rs`

```rust
use cminus_compiler::lexer::Lexer;
use cminus_compiler::lexer::token::TokenKind;

/// 辅助函数：将源代码扫描为 TokenKind 列表（不含 EOF）
fn scan_kinds(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    assert!(lexer.errors.is_empty(), "Unexpected lexer errors: {:?}", lexer.errors);
    tokens.into_iter()
        .map(|t| t.kind)
        .filter(|k| *k != TokenKind::Eof)
        .collect()
}

#[test]
fn test_gcd_function() {
    let source = r#"int gcd(int u, int v){
    if(v==0) return u;
    else return gcd(v, u-u/v*v);
}"#;

    let expected = vec![
        // int gcd(int u, int v){
        TokenKind::Int,
        TokenKind::Identifier("gcd".into()),
        TokenKind::LParen,
        TokenKind::Int,
        TokenKind::Identifier("u".into()),
        TokenKind::Comma,
        TokenKind::Int,
        TokenKind::Identifier("v".into()),
        TokenKind::RParen,
        TokenKind::LBrace,
        // if(v==0) return u;
        TokenKind::If,
        TokenKind::LParen,
        TokenKind::Identifier("v".into()),
        TokenKind::Eq,
        TokenKind::Number(0),
        TokenKind::RParen,
        TokenKind::Return,
        TokenKind::Identifier("u".into()),
        TokenKind::Semicolon,
        // else return gcd(v, u-u/v*v);
        TokenKind::Else,
        TokenKind::Return,
        TokenKind::Identifier("gcd".into()),
        TokenKind::LParen,
        TokenKind::Identifier("v".into()),
        TokenKind::Comma,
        TokenKind::Identifier("u".into()),
        TokenKind::Minus,
        TokenKind::Identifier("u".into()),
        TokenKind::Slash,
        TokenKind::Identifier("v".into()),
        TokenKind::Star,
        TokenKind::Identifier("v".into()),
        TokenKind::RParen,
        TokenKind::Semicolon,
        // }
        TokenKind::RBrace,
    ];

    let actual = scan_kinds(source);
    assert_eq!(actual.len(), expected.len(),
        "Token count mismatch: got {}, expected {}", actual.len(), expected.len());

    for (i, (act, exp)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(act, exp, "Token mismatch at index {}: got {:?}, expected {:?}", i, act, exp);
    }
}

#[test]
fn test_keywords() {
    let kinds = scan_kinds("if else int void while return");
    assert_eq!(kinds, vec![
        TokenKind::If,
        TokenKind::Else,
        TokenKind::Int,
        TokenKind::Void,
        TokenKind::While,
        TokenKind::Return,
    ]);
}

#[test]
fn test_operators() {
    let kinds = scan_kinds("+ - * / < <= > >= == != =");
    assert_eq!(kinds, vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Lt,
        TokenKind::Le,
        TokenKind::Gt,
        TokenKind::Ge,
        TokenKind::Eq,
        TokenKind::Ne,
        TokenKind::Assign,
    ]);
}

#[test]
fn test_delimiters() {
    let kinds = scan_kinds("; , ( ) { } [ ]");
    assert_eq!(kinds, vec![
        TokenKind::Semicolon,
        TokenKind::Comma,
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::LBracket,
        TokenKind::RBracket,
    ]);
}

#[test]
fn test_numbers() {
    let kinds = scan_kinds("0 42 12345");
    assert_eq!(kinds, vec![
        TokenKind::Number(0),
        TokenKind::Number(42),
        TokenKind::Number(12345),
    ]);
}

#[test]
fn test_block_comment_skipped() {
    let kinds = scan_kinds("int /* this is a comment */ x;");
    assert_eq!(kinds, vec![
        TokenKind::Int,
        TokenKind::Identifier("x".into()),
        TokenKind::Semicolon,
    ]);
}

#[test]
fn test_multiline_comment() {
    let source = r#"int x;
/* this comment
   spans multiple
   lines */
int y;"#;
    let kinds = scan_kinds(source);
    assert_eq!(kinds, vec![
        TokenKind::Int,
        TokenKind::Identifier("x".into()),
        TokenKind::Semicolon,
        TokenKind::Int,
        TokenKind::Identifier("y".into()),
        TokenKind::Semicolon,
    ]);
}

#[test]
fn test_unterminated_comment() {
    let source = "int x; /* unterminated comment";
    let mut lexer = Lexer::new(source);
    let _tokens = lexer.tokenize();
    assert!(!lexer.errors.is_empty(), "Should report unterminated comment");
    assert!(lexer.errors[0].message.contains("Unterminated"));
}

#[test]
fn test_unexpected_character() {
    let source = "int x = 5 @ ;";
    let mut lexer = Lexer::new(source