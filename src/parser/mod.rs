pub mod ast;
pub mod error;
#[cfg(test)]
mod tests;

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
impl Parser {
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

    /// 检查当前token是否是某个特定的类型
    pub fn matches(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(expected)
    }

    /// 期望当前token是某个特定的类型，如果是则消耗并返回，否则返回错误
    pub fn expect(&mut self, expected: TokenKind) -> ParseResult<Token> {
        if self.matches(&expected) {
            Ok(self.advance().clone())
        } else {
            Err(ParseError::new(
                format!("Expected token {:?}, found {:?}", expected, self.peek()),
                self.current(),
            ))
        }
    }

    /// 是否为标识符或字面量
    pub fn is_identifier(&self) -> bool {
        matches!(self.peek(), TokenKind::Identifier(_))
    }

    pub fn is_literal(&self) -> bool {
        matches!(self.peek(), TokenKind::Number(_) | TokenKind::String(_))
    }

    /// 消耗一个标识符，返回名字
    pub fn expect_identifier(&mut self) -> ParseResult<String> {
        if let TokenKind::Identifier(name) = self.peek() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::new(
                format!("Expected identifier, found {:?}", self.peek()),
                self.current(),
            ))
        }
    }

    /// 消耗一个字面量，返回值
    pub fn expect_number(&mut self) -> ParseResult<i64> {
        if let TokenKind::Number(value) = self.peek() {
            let value = *value;
            self.advance();
            Ok(value)
        } else {
            Err(ParseError::new(
                format!("Expected number, found {:?}", self.peek()),
                self.current(),
            ))
        }
    }

    pub fn expect_string(&mut self) -> ParseResult<String> {
        if let TokenKind::String(value) = self.peek() {
            let value = value.clone();
            self.advance();
            Ok(value)
        } else {
            Err(ParseError::new(
                format!("Expected string, found {:?}", self.peek()),
                self.current(),
            ))
        }
    }
}

/// 语法解析
impl Parser {
    /// 解析整个程序
    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut declarations = Vec::new();

        while !self.matches(&TokenKind::EOF) {
            declarations.push(self.parse_declaration()?);
        }
        Ok(Program { declarations })
    }

    /// 解析一个声明
    //  declaration → type-specifier ID declaration-tail
    //  declaration-tail → ';' | '[' NUM ']' ';' | '(' params ')' compound-stmt
    pub fn parse_declaration(&mut self) -> ParseResult<Declaration> {
        let type_spec = self.parse_type_spec()?;
        let name = self.expect_identifier()?;

        match self.peek() {
            TokenKind::Semicolon => {
                // var declaration: type-specifier ID ';'
                self.advance();
                Ok(Declaration::Var(VarDecl {
                    type_spec,
                    name,
                    array_size: None,
                }))
            }
            TokenKind::LBracket => {
                // array declaration: type-specifier ID '[' NUM ']' ';'
                self.advance();
                let array_size = self.expect_number()?;
                self.expect(TokenKind::RBracket)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Declaration::Var(VarDecl {
                    type_spec,
                    name,
                    array_size: Some(array_size.try_into().unwrap()),
                }))
            }
            TokenKind::LParen => {
                // function declaration: type-specifier ID '(' params ')' compound-stmt
                self.advance();
                let params = self.parse_params()?;
                self.expect(TokenKind::RParen)?;
                let body = self.parse_compound_stmt()?;
                Ok(Declaration::Func(FuncDecl {
                    return_type: type_spec,
                    name,
                    params,
                    body,
                }))
            }
            _ => Err(ParseError::new(
                format!("Expected ';', '[', or '(', found {:?}", self.peek()),
                self.current(),
            )),
        }
    }

    /// 解析一个类型
    pub fn parse_type_spec(&mut self) -> ParseResult<TypeSpec> {
        match self.peek() {
            TokenKind::Int => {
                self.advance();
                Ok(TypeSpec::Int)
            }
            TokenKind::Void => {
                self.advance();
                Ok(TypeSpec::Void)
            }
            _ => Err(ParseError::new(
                format!("Expected type specifier, found {:?}", self.peek()),
                self.current(),
            )),
        }
    }

    /// 解析参数列表
    pub fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.matches(&TokenKind::RParen) {
            return Ok(params);
        }

        // void 作为唯一参数表示无参数
        if self.matches(&TokenKind::Void) {
            self.advance();
            return Ok(params);
        }

        // Parse parameter declarations
        while !self.matches(&TokenKind::RParen) {
            let type_spec = self.parse_type_spec()?;
            let name = self.expect_identifier()?;
            if self.matches(&TokenKind::LBracket) {
                self.advance();
                let array_size = if matches!(self.peek(), TokenKind::Number(_)) {
                    Some(self.expect_number()?.try_into().unwrap())
                } else {
                    None
                };
                self.expect(TokenKind::RBracket)?;
                params.push(Param {
                    type_spec,
                    name,
                    array_size,
                });
            } else {
                params.push(Param {
                    type_spec,
                    name,
                    array_size: None,
                });
            }

            if !self.matches(&TokenKind::RParen) {
                self.expect(TokenKind::Comma)?;
            }
        }

        Ok(params)
    }

    /// 解析一个语句
    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            TokenKind::If => self.parse_selection_stmt(),
            TokenKind::While => self.parse_iteration_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::LBrace => {
                let compound = self.parse_compound_stmt()?;
                Ok(Stmt::Compound(compound))
            }
            TokenKind::Semicolon => {
                self.advance();
                Ok(Stmt::Empty)
            }
            _ => self.parse_expression_stmt(),
        }
    }

    /// 解析一个选择语句 if '(' expression ')' stmt [else stmt]
    pub fn parse_selection_stmt(&mut self) -> ParseResult<Stmt> {
        self.advance(); // 消耗 'if'
        self.expect(TokenKind::LParen)?; // 消耗 '('
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RParen)?; // 消耗 ')'

        let then_branch = Box::new(self.parse_stmt()?); // 解析 then 分支
        let else_branch = if self.matches(&TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };
        Ok(Stmt::Selection(SelectionStmt {
            condition,
            then_brach: then_branch,
            else_brach: else_branch,
        }))
    }

    /// 解析一个表达式, expression → simple-expression | var '=' expression
    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_simple_expression()?;
        if let Expression::LVar(lvar) = &expr {
            if self.matches(&TokenKind::Assign) {
                self.advance();
                let rhs = self.parse_expression()?;
                return Ok(Expression::Assign {
                    lvar: lvar.clone(),
                    expr: Box::new(rhs),
                });
            }
        }
        Ok(expr)
    }

    /// 解析一个简单表达式, simple-expression → additive-expression relop additive-expression
    /// relop → '<' | '>' | '<=' | '>=' | '==' | '!='
    pub fn parse_simple_expression(&mut self) -> ParseResult<Expression> {
        let left = self.parse_additive_expression()?;
        let op = match self.peek() {
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Ge => BinaryOp::Ge,
            TokenKind::Eq => BinaryOp::Eq,
            TokenKind::Ne => BinaryOp::Ne,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_additive_expression()?;
        Ok(Expression::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// 解析一个加法表达式, additive-expression → term (addop term)*
    pub fn parse_additive_expression(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_term()?;

        while matches!(self.peek(), TokenKind::Plus | TokenKind::Minus) {
            let op = match self.peek() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expression::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// 解析一个项, term → factor (mulop factor)*
    pub fn parse_term(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_factor()?;

        while matches!(self.peek(), TokenKind::Star | TokenKind::Slash) {
            let op = match self.peek() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expression::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// 解析一个因子, factor → '(' expression ')'| ID factor-tail | NUM | STRING
    pub fn parse_factor(&mut self) -> ParseResult<Expression> {
        match self.peek() {
            TokenKind::LParen => {
                self.advance(); // 消耗 '('
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?; // 消耗 ')'
                Ok(expr)
            }
            TokenKind::Identifier(_) => {
                let id = self.expect_identifier()?;
                self.parse_factor_tail(id)
            }
            TokenKind::Number(num) => {
                let num = *num as i32;
                self.advance(); // 消耗 NUM
                Ok(Expression::Number(num))
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.advance(); // 消耗 STRING
                Ok(Expression::String(value))
            }
            _ => Err(ParseError::new(
                format!(
                    "Expected '(', identifier, number, or string, found {:?}",
                    self.peek()
                ),
                self.current(),
            )),
        }
    }

    /// 解析一个因子的尾部(函数调用), factor-tail → '(' args ')' | ε
    pub fn parse_factor_tail(&mut self, name: String) -> ParseResult<Expression> {
        match self.peek() {
            TokenKind::LParen => {
                self.advance();
                let mut args = Vec::new();
                if !self.matches(&TokenKind::RParen) {
                    args.push(self.parse_expression()?);
                    while !self.matches(&TokenKind::RParen) {
                        self.expect(TokenKind::Comma)?;
                        args.push(self.parse_expression()?);
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Expression::Call { name, args })
            }
            TokenKind::LBracket => {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                Ok(Expression::LVar(LVar {
                    name,
                    index: Some(Box::new(index)),
                }))
            }
            _ => Ok(Expression::LVar(LVar { name, index: None })), // ε
        }
    }

    pub fn parse_iteration_stmt(&mut self) -> ParseResult<Stmt> {
        self.advance(); // 消耗 'while'
        self.expect(TokenKind::LParen)?; // 消耗 '('

        let condition = self.parse_expression()?;
        self.expect(TokenKind::RParen)?; // 消耗 ')'

        let body = self.parse_stmt()?; // 解析循环体

        Ok(Stmt::Iteration(IterationStmt {
            condition,
            body: Box::new(body),
        }))
    }

    pub fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
        self.advance(); // 消耗 'return'

        match self.peek() {
            TokenKind::Semicolon => {
                self.advance(); // 消耗 ';'
                Ok(Stmt::Return(None))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Return(Some(expr)))
            }
        }
    }

    pub fn parse_expression_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.parse_expression()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expression(Some(expr)))
    }

    pub fn parse_compound_stmt(&mut self) -> ParseResult<CompoundStmt> {
        self.expect(TokenKind::LBrace)?;
        let mut declarations = Vec::new();
        let mut statements = Vec::new();

        while !self.matches(&TokenKind::RBrace) {
            if let TokenKind::Int = self.peek() {
                declarations.push(self.parse_declaration()?);
            } else {
                statements.push(self.parse_stmt()?);
            }
        }
        self.advance(); // 消耗 '}'

        Ok(CompoundStmt {
            local_decls: declarations,
            stmts: statements,
        })
    }
}

