# C-Minus 语法分析器设计文档与实现指南

## 目录

1. [总体设计思路](#1-总体设计思路)
2. [AST 节点定义](#2-ast-节点定义)
3. [递归下降解析器实现](#3-递归下降解析器实现)
4. [解决文法歧义问题](#4-解决文法歧义问题)
5. [完整 Rust 代码](#5-完整-rust-代码)
6. [测试](#6-测试)

---

## 1. 总体设计思路

### 1.1 选择递归下降法

我们采用**递归下降解析**（Recursive Descent Parsing），原因：
- 手写实现直观，每个非终结符对应一个函数
- 容易嵌入语义动作（构建 AST）
- 错误信息友好，可以精确定位

### 1.2 核心设计原则

```
┌─────────────────┐         ┌──────────────────┐        ┌─────────────┐
│   Token 流       │ ──────▶ │  Parser          │ ─────▶ │   AST       │
│ (Lexer 输出)    │         │  (递归下降)       │        │  (树结构)   │
└─────────────────┘         └──────────────────┘        └─────────────┘
```

Parser 维护一个 **当前 Token** (`current`)，通过 `peek()` 向前看、`eat(expected)` 消耗匹配的 Token。

### 1.3 消除左递归

原始 BNF 中有大量左递归（如 `additive-expression → additive-expression addop term`），递归下降不能处理左递归。我们需要将它们改写为**右递归**或**循环**形式：

```
// 原始（左递归）：
// additive-expression → additive-expression addop term | term

// 改写为循环（EBNF）：
// additive-expression → term { addop term }
```

### 1.4 改写后的 EBNF 文法

```
program              → { declaration }
declaration          → type-specifier ID declaration-tail
declaration-tail     → ';'                                    (变量声明)
                     | '[' NUM ']' ';'                        (数组声明)
                     | '(' params ')' compound-stmt           (函数声明)
type-specifier       → 'int' | 'void'

params               → 'void' | param-list
param-list           → param { ',' param }
param                → type-specifier ID [ '[' ']' ]

compound-stmt        → '{' { var-declaration } { statement } '}'
var-declaration      → type-specifier ID [ '[' NUM ']' ] ';'

statement            → expression-stmt
                     | compound-stmt
                     | selection-stmt
                     | iteration-stmt
                     | return-stmt

expression-stmt      → [ expression ] ';'
selection-stmt       → 'if' '(' expression ')' statement [ 'else' statement ]
iteration-stmt       → 'while' '(' expression ')' statement
return-stmt          → 'return' [ expression ] ';'

expression           → simple-expression                      (可能含赋值)
                     // 赋值的判断：如果解析出 var 且下一个是 '='，则为赋值

simple-expression    → additive-expr [ relop additive-expr ]
relop                → '<=' | '<' | '>' | '>=' | '==' | '!='

additive-expr        → term { addop term }
addop                → '+' | '-'

term                 → factor { mulop factor }
mulop                → '*' | '/'

factor               → '(' expression ')'
                     | ID factor-tail
                     | NUM

factor-tail          → '(' args ')'                          (函数调用)
                     | '[' expression ']'                     (数组下标)
                     | ε                                      (普通变量)

args                 → [ expression { ',' expression } ]
```

### 1.5 关键难点：表达式中的赋值

`expression → var = expression | simple-expression` 有歧义：看到 `ID` 时无法立刻判断是赋值还是普通表达式。

**解决方案**：先按 `simple-expression` 解析，如果结果是一个 `Var` 节点且下一个 token 是 `=`，则回退将其解释为赋值语句的左值。

---

## 2. AST 节点定义

### `src/parser/ast.rs`

```rust
/// 类型说明符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    Int,
    Void,
}

/// 程序 = 声明列表
#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

/// 声明
#[derive(Debug, Clone)]
pub enum Declaration {
    VarDecl(VarDeclaration),
    FunDecl(FunDeclaration),
}

/// 变量声明
#[derive(Debug, Clone)]
pub struct VarDeclaration {
    pub type_spec: TypeSpec,
    pub name: String,
    pub array_size: Option<i64>, // None = 普通变量, Some(n) = 数组[n]
}

/// 函数声明
#[derive(Debug, Clone)]
pub struct FunDeclaration {
    pub return_type: TypeSpec,
    pub name: String,
    pub params: Vec<Param>,
    pub body: CompoundStmt,
}

/// 参数
#[derive(Debug, Clone)]
pub struct Param {
    pub type_spec: TypeSpec,
    pub name: String,
    pub is_array: bool, // true = type id[]
}

/// 复合语句 { local_decls stmts }
#[derive(Debug, Clone)]
pub struct CompoundStmt {
    pub local_declarations: Vec<VarDeclaration>,
    pub statements: Vec<Statement>,
}

/// 语句
#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Option<Expression>), // expression-stmt (None = 空语句 ";")
    Compound(CompoundStmt),
    Selection(SelectionStmt),
    Iteration(IterationStmt),
    Return(Option<Expression>),
}

/// if 语句
#[derive(Debug, Clone)]
pub struct SelectionStmt {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

/// while 语句
#[derive(Debug, Clone)]
pub struct IterationStmt {
    pub condition: Expression,
    pub body: Box<Statement>,
}

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    Assign {
        var: Var,
        expr: Box<Expression>,
    },
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Var(Var),
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Num(i64),
}

/// 变量引用（左值）
#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub index: Option<Box<Expression>>, // None = 普通变量, Some = 数组下标
}

/// 二元运算符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Lt  => write!(f, "<"),
            BinaryOp::Le  => write!(f, "<="),
            BinaryOp::Gt  => write!(f, ">"),
            BinaryOp::Ge  => write!(f, ">="),
            BinaryOp::Eq  => write!(f, "=="),
            BinaryOp::Ne  => write!(f, "!="),
        }
    }
}
```

---

## 3. 递归下降解析器实现

### `src/parser/mod.rs`

```rust
pub mod ast;

use crate::lexer::token::{Token, TokenKind, Span};
use ast::*;

// ═══════════════════════════════════════════════════
//  解析错误
// ═══════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Parse Error at {}] {}", self.span, self.message)
    }
}

type ParseResult<T> = Result<T, ParseError>;

// ═══════════════════════════════════════════════════
//  Parser 结构
// ═══════════════════════════════════════════════════

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ─────────────────────────────────────────────
    //  基础工具方法
    // ─────────────────────────────────────────────

    /// 查看当前 token（不消耗）
    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    /// 获取当前 token 的 span
    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span.clone())
            .unwrap_or(Span::new(0, 0))
    }

    /// 消耗当前 token 并返回
    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    /// 期望当前 token 匹配 expected，匹配则消耗并返回 Ok
    fn expect(&mut self, expected: &TokenKind) -> ParseResult<()> {
        if self.matches(expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.peek()),
                span: self.current_span(),
            })
        }
    }

    /// 判断当前 token 是否匹配（不消耗）
    fn matches(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }

    /// 检查当前 token 是否是某个标识符（任意标识符）
    fn is_identifier(&self) -> bool {
        matches!(self.peek(), TokenKind::Identifier(_))
    }

    /// 消耗一个标识符，返回名字
    fn expect_identifier(&mut self) -> ParseResult<String> {
        if let TokenKind::Identifier(name) = self.peek().clone() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError {
                message: format!("Expected identifier, found {:?}", self.peek()),
                span: self.current_span(),
            })
        }
    }

    /// 消耗一个数字，返回值
    fn expect_number(&mut self) -> ParseResult<i64> {
        if let TokenKind::Number(n) = self.peek().clone() {
            self.advance();
            Ok(n)
        } else {
            Err(ParseError {
                message: format!("Expected number, found {:?}", self.peek()),
                span: self.current_span(),
            })
        }
    }

    // ─────────────────────────────────────────────
    //  program → { declaration }
    // ─────────────────────────────────────────────

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut declarations = Vec::new();
        while *self.peek() != TokenKind::Eof {
            declarations.push(self.parse_declaration()?);
        }
        Ok(Program { declarations })
    }

    // ─────────────────────────────────────────────
    //  declaration → type-specifier ID declaration-tail
    //  declaration-tail → ';' | '[' NUM ']' ';' | '(' params ')' compound-stmt
    // ─────────────────────────────────────────────

    fn parse_declaration(&mut self) -> ParseResult<Declaration> {
        let type_spec = self.parse_type_specifier()?;
        let name = self.expect_identifier()?;

        match self.peek() {
            TokenKind::Semicolon => {
                // var-declaration: type ID ;
                self.advance();
                Ok(Declaration::VarDecl(VarDeclaration {
                    type_spec,
                    name,
                    array_size: None,
                }))
            }
            TokenKind::LBracket => {
                // var-declaration: type ID [ NUM ] ;
                self.advance(); // consume '['
                let size = self.expect_number()?;
                self.expect(&TokenKind::RBracket)?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Declaration::VarDecl(VarDeclaration {
                    type_spec,
                    name,
                    array_size: Some(size),
                }))
            }
            TokenKind::LParen => {
                // fun-declaration: type ID ( params ) compound-stmt
                self.advance(); // consume '('
                let params = self.parse_params()?;
                self.expect(&TokenKind::RParen)?;
                let body = self.parse_compound_stmt()?;
                Ok(Declaration::FunDecl(FunDeclaration {
                    return_type: type_spec,
                    name,
                    params,
                    body,
                }))
            }
            _ => Err(ParseError {
                message: format!("Expected ';', '[', or '(' after identifier in declaration, found {:?}", self.peek()),
                span: self.current_span(),
            }),
        }
    }

    // ─────────────────────────────────────────────
    //  type-specifier → 'int' | 'void'
    // ─────────────────────────────────────────────
}
```