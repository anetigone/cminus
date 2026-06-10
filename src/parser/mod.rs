pub mod ast;
pub mod error;
pub mod print;
#[cfg(test)]
mod tests;

use crate::lexer::token::*;
use ast::*;
use error::ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    /// 收集到的错误
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
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

    /// 期望当前token是某个特定的类型，如果是则消耗并返回，否则记录错误并返回 None
    pub fn expect(&mut self, expected: TokenKind) -> Option<Token> {
        if self.matches(&expected) {
            Some(self.advance().clone())
        } else {
            self.error(format!("Expected token {:?}, found {:?}", expected, self.peek()))
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
    pub fn expect_identifier(&mut self) -> Option<String> {
        if let TokenKind::Identifier(name) = self.peek() {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            self.error(format!("Expected identifier, found {:?}", self.peek()))
        }
    }

    /// 消耗一个数字字面量，返回值
    pub fn expect_number(&mut self) -> Option<i64> {
        if let TokenKind::Number(value) = self.peek() {
            let value = *value;
            self.advance();
            Some(value)
        } else {
            self.error(format!("Expected number, found {:?}", self.peek()))
        }
    }

    pub fn expect_string(&mut self) -> Option<String> {
        if let TokenKind::String(value) = self.peek() {
            let value = value.clone();
            self.advance();
            Some(value)
        } else {
            self.error(format!("Expected string, found {:?}", self.peek()))
        }
    }

    /// 记录一个语法错误并返回 None
    fn error<T>(&mut self, message: String) -> Option<T> {
        self.errors.push(ParseError::new(message, self.current()));
        None
    }

    /// 恐慌模式同步：跳过 token 直到遇到同步点
    /// 同步 token: `;`、`)` (消费), `}`、`EOF` (不消费)
    fn synchronize(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Semicolon => {
                    self.advance();
                    return;
                }
                TokenKind::RParen => {
                    self.advance();
                    return;
                }
                TokenKind::RBrace | TokenKind::EOF => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

/// 语法解析
impl Parser {
    /// 解析整个程序，返回 Program（声明列表可能不完整）
    pub fn parse_program(&mut self) -> Program {
        let mut declarations = Vec::new();

        while !self.matches(&TokenKind::EOF) {
            let pos = self.current;
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            } else {
                // parse_declaration 失败并已同步
                // 如果位置没有前进（卡住），强制前进避免死循环
                if self.current == pos {
                    self.advance();
                }
            }
        }
        Program { declarations }
    }

    /// 解析一个声明
    //  declaration → type-specifier ID declaration-tail
    //  declaration-tail → ';' | '[' NUM ']' ';' | '(' params ')' compound-stmt
    pub fn parse_declaration(&mut self) -> Option<Declaration> {
        let type_spec = match self.parse_type_spec() {
            Some(t) => t,
            None => {
                self.synchronize();
                return None;
            }
        };

        let name = match self.expect_identifier() {
            Some(n) => n,
            None => {
                self.synchronize();
                return None;
            }
        };

        match self.peek() {
            TokenKind::Semicolon => {
                // var declaration: type-specifier ID ';'
                self.advance();
                Some(Declaration::Var(VarDecl {
                    type_spec,
                    name,
                    array_size: None,
                }))
            }
            TokenKind::LBracket => {
                // array declaration: type-specifier ID '[' NUM ']' ';'
                self.advance();
                let array_size = match self.expect_number() {
                    Some(n) => n,
                    None => {
                        self.synchronize();
                        return None;
                    }
                };
                if self.expect(TokenKind::RBracket).is_none() {
                    self.synchronize();
                    return None;
                }
                if self.expect(TokenKind::Semicolon).is_none() {
                    self.synchronize();
                    return None;
                }
                Some(Declaration::Var(VarDecl {
                    type_spec,
                    name,
                    array_size: Some(array_size.try_into().unwrap()),
                }))
            }
            TokenKind::LParen => {
                // function declaration: type-specifier ID '(' params ')' compound-stmt
                self.advance();
                let params = match self.parse_params() {
                    Some(p) => p,
                    None => {
                        self.synchronize();
                        return None;
                    }
                };
                if self.expect(TokenKind::RParen).is_none() {
                    self.synchronize();
                    return None;
                }
                let body = match self.parse_compound_stmt() {
                    Some(b) => b,
                    None => {
                        self.synchronize();
                        return None;
                    }
                };
                Some(Declaration::Func(FuncDecl {
                    return_type: type_spec,
                    name,
                    params,
                    body,
                }))
            }
            _ => {
                self.error::<()>(format!(
                    "Expected ';', '[', or '(', found {:?}",
                    self.peek()
                ));
                self.synchronize();
                None
            }
        }
    }

    /// 解析一个类型
    pub fn parse_type_spec(&mut self) -> Option<TypeSpec> {
        match self.peek() {
            TokenKind::Int => {
                self.advance();
                Some(TypeSpec::Int)
            }
            TokenKind::Void => {
                self.advance();
                Some(TypeSpec::Void)
            }
            _ => self.error(format!("Expected type specifier, found {:?}", self.peek())),
        }
    }

    /// 解析参数列表
    pub fn parse_params(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();

        if self.matches(&TokenKind::RParen) {
            return Some(params);
        }

        // void 作为唯一参数表示无参数
        if self.matches(&TokenKind::Void) {
            self.advance();
            return Some(params);
        }

        // Parse parameter declarations
        while !self.matches(&TokenKind::RParen) && !self.matches(&TokenKind::EOF) {
            let type_spec = self.parse_type_spec()?;
            let name = self.expect_identifier()?;
            let array_size = if self.matches(&TokenKind::LBracket) {
                self.advance();
                let size = if matches!(self.peek(), TokenKind::Number(_)) {
                    Some(self.expect_number()?.try_into().unwrap())
                } else {
                    None
                };
                self.expect(TokenKind::RBracket)?;
                size
            } else {
                None
            };

            params.push(Param {
                type_spec,
                name,
                array_size,
            });

            if !self.matches(&TokenKind::RParen) {
                self.expect(TokenKind::Comma)?;
            }
        }

        Some(params)
    }

    /// 解析一个语句
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            TokenKind::If => self.parse_selection_stmt(),
            TokenKind::While => self.parse_iteration_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::LBrace => {
                let compound = self.parse_compound_stmt()?;
                Some(Stmt::Compound(compound))
            }
            TokenKind::Semicolon => {
                self.advance();
                Some(Stmt::Empty)
            }
            _ => self.parse_expression_stmt(),
        }
    }

    /// 解析一个选择语句 if '(' expression ')' stmt [else stmt]
    pub fn parse_selection_stmt(&mut self) -> Option<Stmt> {
        self.advance(); // 消耗 'if'

        if self.expect(TokenKind::LParen).is_none() {
            self.synchronize();
            return None;
        }

        let condition = match self.parse_expression() {
            Some(e) => e,
            None => {
                self.synchronize();
                return None;
            }
        };

        if self.expect(TokenKind::RParen).is_none() {
            self.synchronize();
            return None;
        }

        let then_branch = match self.parse_stmt() {
            Some(s) => Box::new(s),
            None => {
                self.synchronize();
                return None;
            }
        };

        let else_branch = if self.matches(&TokenKind::Else) {
            self.advance();
            match self.parse_stmt() {
                Some(s) => Some(Box::new(s)),
                None => {
                    self.synchronize();
                    return None;
                }
            }
        } else {
            None
        };

        Some(Stmt::Selection(SelectionStmt {
            condition,
            then_brach: then_branch,
            else_brach: else_branch,
        }))
    }

    /// 解析一个表达式, expression → simple-expression | var '=' expression
    pub fn parse_expression(&mut self) -> Option<Expression> {
        let expr = self.parse_simple_expression()?;
        if let Expression::LVar(lvar) = &expr {
            if self.matches(&TokenKind::Assign) {
                self.advance();
                let rhs = self.parse_expression()?;
                return Some(Expression::Assign {
                    lvar: lvar.clone(),
                    expr: Box::new(rhs),
                });
            }
        }
        Some(expr)
    }

    /// 解析一个简单表达式, simple-expression → additive-expression relop additive-expression
    /// relop → '<' | '>' | '<=' | '>=' | '==' | '!='
    pub fn parse_simple_expression(&mut self) -> Option<Expression> {
        let left = self.parse_additive_expression()?;
        let op = match self.peek() {
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Ge => BinaryOp::Ge,
            TokenKind::Eq => BinaryOp::Eq,
            TokenKind::Ne => BinaryOp::Ne,
            _ => return Some(left),
        };
        self.advance();
        let right = self.parse_additive_expression()?;
        Some(Expression::BinOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// 解析一个加法表达式, additive-expression → term (addop term)*
    pub fn parse_additive_expression(&mut self) -> Option<Expression> {
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
        Some(left)
    }

    /// 解析一个项, term → factor (mulop factor)*
    pub fn parse_term(&mut self) -> Option<Expression> {
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
        Some(left)
    }

    /// 解析一个因子, factor → '(' expression ')'| ID factor-tail | NUM | STRING
    pub fn parse_factor(&mut self) -> Option<Expression> {
        match self.peek() {
            TokenKind::LParen => {
                self.advance(); // 消耗 '('
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Some(expr)
            }
            TokenKind::Identifier(_) => {
                let id = self.expect_identifier()?;
                self.parse_factor_tail(id)
            }
            TokenKind::Number(num) => {
                let num = *num as i32;
                self.advance(); // 消耗 NUM
                Some(Expression::Number(num))
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.advance(); // 消耗 STRING
                Some(Expression::String(value))
            }
            _ => self.error(format!(
                "Expected '(', identifier, number, or string, found {:?}",
                self.peek()
            )),
        }
    }

    /// 解析一个因子的尾部(函数调用), factor-tail → '(' args ')' | ε
    pub fn parse_factor_tail(&mut self, name: String) -> Option<Expression> {
        match self.peek() {
            TokenKind::LParen => {
                self.advance();
                let mut args = Vec::new();
                if !self.matches(&TokenKind::RParen) {
                    args.push(self.parse_expression()?);
                    while !self.matches(&TokenKind::RParen) && !self.matches(&TokenKind::EOF) {
                        self.expect(TokenKind::Comma)?;
                        args.push(self.parse_expression()?);
                    }
                }
                self.expect(TokenKind::RParen)?;
                Some(Expression::Call { name, args })
            }
            TokenKind::LBracket => {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                Some(Expression::LVar(LVar {
                    name,
                    index: Some(Box::new(index)),
                }))
            }
            _ => Some(Expression::LVar(LVar { name, index: None })),
        }
    }

    pub fn parse_iteration_stmt(&mut self) -> Option<Stmt> {
        self.advance(); // 消耗 'while'

        if self.expect(TokenKind::LParen).is_none() {
            self.synchronize();
            return None;
        }

        let condition = match self.parse_expression() {
            Some(e) => e,
            None => {
                self.synchronize();
                return None;
            }
        };

        if self.expect(TokenKind::RParen).is_none() {
            self.synchronize();
            return None;
        }

        let body = match self.parse_stmt() {
            Some(s) => s,
            None => {
                self.synchronize();
                return None;
            }
        };

        Some(Stmt::Iteration(IterationStmt {
            condition,
            body: Box::new(body),
        }))
    }

    pub fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.advance(); // 消耗 'return'

        match self.peek() {
            TokenKind::Semicolon => {
                self.advance(); // 消耗 ';'
                Some(Stmt::Return(None))
            }
            _ => {
                let expr = match self.parse_expression() {
                    Some(e) => e,
                    None => {
                        self.synchronize();
                        return None;
                    }
                };
                if self.expect(TokenKind::Semicolon).is_none() {
                    self.synchronize();
                    return None;
                }
                Some(Stmt::Return(Some(expr)))
            }
        }
    }

    pub fn parse_expression_stmt(&mut self) -> Option<Stmt> {
        let expr = match self.parse_expression() {
            Some(e) => e,
            None => {
                self.synchronize();
                return None;
            }
        };
        if self.expect(TokenKind::Semicolon).is_none() {
            self.synchronize();
            return None;
        }
        Some(Stmt::Expression(Some(expr)))
    }

    pub fn parse_compound_stmt(&mut self) -> Option<CompoundStmt> {
        if self.expect(TokenKind::LBrace).is_none() {
            return None;
        }

        let mut declarations = Vec::new();
        let mut statements = Vec::new();

        while !self.matches(&TokenKind::RBrace) && !self.matches(&TokenKind::EOF) {
            let pos = self.current;
            if let TokenKind::Int = self.peek() {
                if let Some(decl) = self.parse_declaration() {
                    declarations.push(decl);
                }
                // parse_declaration 失败时已同步
            } else {
                if let Some(stmt) = self.parse_stmt() {
                    statements.push(stmt);
                }
                // parse_stmt 失败时已同步
            }
            // 防止位置没前进导致死循环
            if self.current == pos {
                self.advance();
            }
        }

        // 消耗 '}'
        if self.matches(&TokenKind::RBrace) {
            self.advance();
        } else {
            // 遇到 EOF 或其他意外 token，缺少 '}'
            self.error::<()>(format!(
                "Expected token RBrace, found {:?}",
                self.peek()
            ));
        }

        Some(CompoundStmt {
            local_decls: declarations,
            stmts: statements,
        })
    }
}
