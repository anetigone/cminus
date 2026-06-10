use super::*;
use crate::lexer::Lexer;

fn parse_source(source: &str) -> Program {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();
    assert!(lexer.errors.is_empty(), "{:?}", lexer.errors);

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();
    assert!(parser.errors.is_empty(), "{:?}", parser.errors);
    program
}

fn parse_source_err(source: &str) -> ParseError {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();
    assert!(lexer.errors.is_empty(), "{:?}", lexer.errors);

    let mut parser = Parser::new(tokens);
    let _program = parser.parse_program();
    assert!(!parser.errors.is_empty(), "expected at least one parse error");
    parser.errors.remove(0)
}

fn only_function(program: Program) -> FuncDecl {
    assert_eq!(program.declarations.len(), 1);
    match program.declarations.into_iter().next().unwrap() {
        Declaration::Func(func) => func,
        other => panic!("expected function declaration, got {:?}", other),
    }
}

#[cfg(test)]
mod declaration_tests {
    use super::*;

    #[test]
    fn var_decl_int() {
        let program = parse_source("int x;");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Var(v) => {
                assert_eq!(v.type_spec, TypeSpec::Int);
                assert_eq!(v.name, "x");
                assert_eq!(v.array_size, None);
            }
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn var_decl_void() {
        let program = parse_source("void x;");
        match &program.declarations[0] {
            Declaration::Var(v) => {
                assert_eq!(v.type_spec, TypeSpec::Void);
                assert_eq!(v.name, "x");
            }
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn var_decl_array() {
        let program = parse_source("int arr[10];");
        match &program.declarations[0] {
            Declaration::Var(v) => {
                assert_eq!(v.type_spec, TypeSpec::Int);
                assert_eq!(v.name, "arr");
                assert_eq!(v.array_size, Some(10));
            }
            other => panic!("expected var decl, got {:?}", other),
        }
    }

    #[test]
    fn multiple_var_decls() {
        let program = parse_source("int x; int y; int arr[5];");
        assert_eq!(program.declarations.len(), 3);
    }

    #[test]
    fn func_decl_no_params() {
        let func = only_function(parse_source("void main() {}"));
        assert_eq!(func.return_type, TypeSpec::Void);
        assert_eq!(func.name, "main");
        assert!(func.params.is_empty());
        assert!(func.body.local_decls.is_empty());
        assert!(func.body.stmts.is_empty());
    }

    #[test]
    fn func_decl_with_params() {
        let func = only_function(parse_source("int gcd(int u, int v) {}"));
        assert_eq!(func.return_type, TypeSpec::Int);
        assert_eq!(func.name, "gcd");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].name, "u");
        assert_eq!(func.params[1].name, "v");
    }

    #[test]
    fn func_decl_array_param() {
        let func = only_function(parse_source("void foo(int a[]) {}"));
        assert_eq!(func.params.len(), 1);
        assert_eq!(func.params[0].name, "a");
        assert_eq!(func.params[0].array_size, None);
    }

    #[test]
    fn mixed_decls() {
        let program = parse_source("int x; void main() {} int arr[5];");
        assert_eq!(program.declarations.len(), 3);
        assert!(matches!(&program.declarations[0], Declaration::Var(_)));
        assert!(matches!(&program.declarations[1], Declaration::Func(_)));
        assert!(matches!(&program.declarations[2], Declaration::Var(_)));
    }
}

#[cfg(test)]
mod stmt_tests {
    use super::*;

    #[test]
    fn empty_stmt() {
        let func = only_function(parse_source("void main() { ; }"));
        assert!(matches!(&func.body.stmts[0], Stmt::Empty));
    }

    #[test]
    fn expression_stmt() {
        let func = only_function(parse_source("void main() { x; }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::LVar(v))) => {
                assert_eq!(v.name, "x");
                assert!(v.index.is_none());
            }
            other => panic!("expected expression stmt, got {:?}", other),
        }
    }

    #[test]
    fn return_void() {
        let func = only_function(parse_source("void main() { return; }"));
        match &func.body.stmts[0] {
            Stmt::Return(None) => {}
            other => panic!("expected void return, got {:?}", other),
        }
    }

    #[test]
    fn return_expr() {
        let func = only_function(parse_source("int main() { return 42; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::Number(n))) => assert_eq!(*n, 42),
            other => panic!("expected return 42, got {:?}", other),
        }
    }

    #[test]
    fn if_without_else() {
        let func = only_function(parse_source("void main() { if (x) return; }"));
        match &func.body.stmts[0] {
            Stmt::Selection(sel) => {
                assert!(matches!(&sel.then_brach.as_ref(), Stmt::Return(None)));
                assert!(sel.else_brach.is_none());
            }
            other => panic!("expected if stmt, got {:?}", other),
        }
    }

    #[test]
    fn if_with_else() {
        let func = only_function(parse_source("void main() { if (x) return; else return; }"));
        match &func.body.stmts[0] {
            Stmt::Selection(sel) => {
                assert!(matches!(&sel.then_brach.as_ref(), Stmt::Return(None)));
                assert!(sel.else_brach.is_some());
            }
            other => panic!("expected if-else stmt, got {:?}", other),
        }
    }

    #[test]
    fn while_stmt() {
        let func = only_function(parse_source("void main() { while (i < 10) i = i + 1; }"));
        match &func.body.stmts[0] {
            Stmt::Iteration(iter) => {
                assert!(matches!(
                    &iter.condition,
                    Expression::BinOp { op: BinaryOp::Lt, .. }
                ));
            }
            other => panic!("expected while stmt, got {:?}", other),
        }
    }

    #[test]
    fn compound_stmt() {
        let func = only_function(parse_source("void main() { { x; } }"));
        match &func.body.stmts[0] {
            Stmt::Compound(inner) => {
                assert!(inner.local_decls.is_empty());
                assert_eq!(inner.stmts.len(), 1);
            }
            other => panic!("expected compound stmt, got {:?}", other),
        }
    }

    #[test]
    fn local_var_in_compound() {
        let func = only_function(parse_source("void main() { int x; x = 1; }"));
        assert_eq!(func.body.local_decls.len(), 1);
        assert_eq!(func.body.stmts.len(), 1);
    }
}

#[cfg(test)]
mod expression_tests {
    use super::*;

    #[test]
    fn number_expr() {
        let func = only_function(parse_source("void main() { return 123; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::Number(n))) => assert_eq!(*n, 123),
            other => panic!("expected number, got {:?}", other),
        }
    }

    #[test]
    fn string_expr() {
        let func = only_function(parse_source(r#"void main(){ return "hello"; }"#));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::String(value))) => assert_eq!(value, "hello"),
            other => panic!("expected string, got {:?}", other),
        }
    }

    #[test]
    fn var_expr() {
        let func = only_function(parse_source("void main() { return x; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::LVar(v))) => {
                assert_eq!(v.name, "x");
                assert!(v.index.is_none());
            }
            other => panic!("expected var, got {:?}", other),
        }
    }

    #[test]
    fn array_access_expr() {
        let func = only_function(parse_source("void main() { return arr[0]; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::LVar(v))) => {
                assert_eq!(v.name, "arr");
                assert!(v.index.is_some());
            }
            other => panic!("expected array access, got {:?}", other),
        }
    }

    #[test]
    fn assign_expr() {
        let func = only_function(parse_source("void main() { x = 1; }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Assign { lvar, expr })) => {
                assert_eq!(lvar.name, "x");
                assert!(matches!(expr.as_ref(), Expression::Number(1)));
            }
            other => panic!("expected assign, got {:?}", other),
        }
    }

    #[test]
    fn assign_array_element() {
        let func = only_function(parse_source("void main() { arr[0] = 1; }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Assign { lvar, expr })) => {
                assert_eq!(lvar.name, "arr");
                assert!(lvar.index.is_some());
                assert!(matches!(expr.as_ref(), Expression::Number(1)));
            }
            other => panic!("expected array assign, got {:?}", other),
        }
    }

    #[test]
    fn call_no_args() {
        let func = only_function(parse_source("void main() { input(); }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Call { name, args })) => {
                assert_eq!(name, "input");
                assert!(args.is_empty());
            }
            other => panic!("expected call, got {:?}", other),
        }
    }

    #[test]
    fn call_with_args() {
        let func = only_function(parse_source("void main() { output(x, y); }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Call { name, args })) => {
                assert_eq!(name, "output");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected call with args, got {:?}", other),
        }
    }

    #[test]
    fn call_string_argument() {
        let func = only_function(parse_source(r#"void main(){ output("hello\nworld"); }"#));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Call { name, args })) => {
                assert_eq!(name, "output");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expression::String(value) => assert_eq!(value, "hello\nworld"),
                    expr => panic!("expected string argument, got {:?}", expr),
                }
            }
            other => panic!("expected call, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod binop_tests {
    use super::*;

    #[test]
    fn add_expr() {
        let func = only_function(parse_source("void main() { return a + b; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op, .. })) => {
                assert_eq!(*op, BinaryOp::Add);
            }
            other => panic!("expected add, got {:?}", other),
        }
    }

    #[test]
    fn sub_expr() {
        let func = only_function(parse_source("void main() { return a - b; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op, .. })) => {
                assert_eq!(*op, BinaryOp::Sub);
            }
            other => panic!("expected sub, got {:?}", other),
        }
    }

    #[test]
    fn mul_expr() {
        let func = only_function(parse_source("void main() { return a * b; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op, .. })) => {
                assert_eq!(*op, BinaryOp::Mul);
            }
            other => panic!("expected mul, got {:?}", other),
        }
    }

    #[test]
    fn div_expr() {
        let func = only_function(parse_source("void main() { return a / b; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op, .. })) => {
                assert_eq!(*op, BinaryOp::Div);
            }
            other => panic!("expected div, got {:?}", other),
        }
    }

    #[test]
    fn comparison_operators() {
        let cases = vec![
            ("a < b", BinaryOp::Lt),
            ("a > b", BinaryOp::Gt),
            ("a <= b", BinaryOp::Le),
            ("a >= b", BinaryOp::Ge),
            ("a == b", BinaryOp::Eq),
            ("a != b", BinaryOp::Ne),
        ];
        for (src, expected_op) in cases {
            let program = parse_source(&format!("void main() {{ return {src}; }}"));
            let func = only_function(program);
            match &func.body.stmts[0] {
                Stmt::Return(Some(Expression::BinOp { op, .. })) => {
                    assert_eq!(*op, expected_op, "failed for source: {src}");
                }
                other => panic!("expected binop for {src}, got {:?}", other),
            }
        }
    }

    #[test]
    fn string_relational_expression() {
        let func = only_function(parse_source(r#"void main(){ return "a" == "b"; }"#));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op, left, right })) => {
                assert_eq!(*op, BinaryOp::Eq);
                assert!(matches!(left.as_ref(), Expression::String(v) if v == "a"));
                assert!(matches!(right.as_ref(), Expression::String(v) if v == "b"));
            }
            other => panic!("expected string equality, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod precedence_tests {
    use super::*;

    #[test]
    fn mul_over_add() {
        let func = only_function(parse_source("void main() { return a + b * c; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op: BinaryOp::Add, left, right })) => {
                assert!(matches!(left.as_ref(), Expression::LVar(_)));
                assert!(matches!(
                    right.as_ref(),
                    Expression::BinOp { op: BinaryOp::Mul, .. }
                ));
            }
            other => panic!("expected add with mul child, got {:?}", other),
        }
    }

    #[test]
    fn parentheses_override() {
        let func = only_function(parse_source("void main() { return (a + b) * c; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op: BinaryOp::Mul, left, right })) => {
                assert!(matches!(
                    left.as_ref(),
                    Expression::BinOp { op: BinaryOp::Add, .. }
                ));
                assert!(matches!(right.as_ref(), Expression::LVar(_)));
            }
            other => panic!("expected mul with add child, got {:?}", other),
        }
    }

    #[test]
    fn left_associative_add() {
        let func = only_function(parse_source("void main() { return a + b + c; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op: BinaryOp::Add, left, right })) => {
                assert!(matches!(
                    left.as_ref(),
                    Expression::BinOp { op: BinaryOp::Add, .. }
                ));
                assert!(matches!(right.as_ref(), Expression::LVar(_)));
            }
            other => panic!("expected left-assoc add, got {:?}", other),
        }
    }

    #[test]
    fn left_associative_mul() {
        let func = only_function(parse_source("void main() { return a * b * c; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op: BinaryOp::Mul, left, right })) => {
                assert!(matches!(
                    left.as_ref(),
                    Expression::BinOp { op: BinaryOp::Mul, .. }
                ));
                assert!(matches!(right.as_ref(), Expression::LVar(_)));
            }
            other => panic!("expected left-assoc mul, got {:?}", other),
        }
    }

    #[test]
    fn mixed_ops() {
        let func = only_function(parse_source("void main() { return a + b * c - d / e; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op: BinaryOp::Sub, left, right })) => {
                match left.as_ref() {
                    Expression::BinOp { op: BinaryOp::Add, left: l, right: r } => {
                        assert!(matches!(l.as_ref(), Expression::LVar(_)));
                        assert!(matches!(
                            r.as_ref(),
                            Expression::BinOp { op: BinaryOp::Mul, .. }
                        ));
                    }
                    other => panic!("expected add, got {:?}", other),
                }
                assert!(matches!(
                    right.as_ref(),
                    Expression::BinOp { op: BinaryOp::Div, .. }
                ));
            }
            other => panic!("expected sub, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod nested_tests {
    use super::*;

    #[test]
    fn nested_if_else() {
        let func = only_function(parse_source(
            "void main() { if (a) if (b) return 1; else return 2; }",
        ));
        match &func.body.stmts[0] {
            Stmt::Selection(outer) => {
                assert!(outer.else_brach.is_none());
                match outer.then_brach.as_ref() {
                    Stmt::Selection(inner) => {
                        assert!(inner.else_brach.is_some());
                    }
                    other => panic!("expected nested if, got {:?}", other),
                }
            }
            other => panic!("expected outer if, got {:?}", other),
        }
    }

    #[test]
    fn while_with_compound_body() {
        let func = only_function(parse_source(
            "void main() { while (x > 0) { int y; x = x - 1; } }",
        ));
        match &func.body.stmts[0] {
            Stmt::Iteration(iter) => match iter.body.as_ref() {
                Stmt::Compound(body) => {
                    assert_eq!(body.local_decls.len(), 1);
                    assert_eq!(body.stmts.len(), 1);
                }
                other => panic!("expected compound body, got {:?}", other),
            },
            other => panic!("expected while, got {:?}", other),
        }
    }

    #[test]
    fn nested_call() {
        let func = only_function(parse_source("void main() { return gcd(x, y); }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::Call { name, args })) => {
                assert_eq!(name, "gcd");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected call, got {:?}", other),
        }
    }

    #[test]
    fn call_in_arithmetic() {
        let func = only_function(parse_source("void main() { return input() + 1; }"));
        match &func.body.stmts[0] {
            Stmt::Return(Some(Expression::BinOp { op: BinaryOp::Add, left, right })) => {
                assert!(matches!(left.as_ref(), Expression::Call { .. }));
                assert!(matches!(right.as_ref(), Expression::Number(1)));
            }
            other => panic!("expected add with call, got {:?}", other),
        }
    }

    #[test]
    fn assign_from_call() {
        let func = only_function(parse_source("void main() { x = input(); }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Assign { lvar, expr })) => {
                assert_eq!(lvar.name, "x");
                assert!(matches!(expr.as_ref(), Expression::Call { .. }));
            }
            other => panic!("expected assign from call, got {:?}", other),
        }
    }

    #[test]
    fn complex_assign_rhs() {
        let func = only_function(parse_source("void main() { x = a + b * c; }"));
        match &func.body.stmts[0] {
            Stmt::Expression(Some(Expression::Assign { lvar, expr })) => {
                assert_eq!(lvar.name, "x");
                match expr.as_ref() {
                    Expression::BinOp { op: BinaryOp::Add, left, right } => {
                        assert!(matches!(left.as_ref(), Expression::LVar(_)));
                        assert!(matches!(
                            right.as_ref(),
                            Expression::BinOp { op: BinaryOp::Mul, .. }
                        ));
                    }
                    other => panic!("expected add, got {:?}", other),
                }
            }
            other => panic!("expected assign, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod full_program_tests {
    use super::*;

    #[test]
    fn gcd_program() {
        let source = r#"
int gcd(int u, int v){
    if(v==0) return u;
    else return gcd(v, u-u/v*v);
}

void main(void){
    int x;int y;
    x=input();
    y=input();
    output(gcd(x,y));
}
"#;
        let program = parse_source(source);
        assert_eq!(program.declarations.len(), 2);

        let gcd_func = match &program.declarations[0] {
            Declaration::Func(f) => f,
            other => panic!("expected func, got {:?}", other),
        };
        assert_eq!(gcd_func.name, "gcd");
        assert_eq!(gcd_func.params.len(), 2);

        let main_func = match &program.declarations[1] {
            Declaration::Func(f) => f,
            other => panic!("expected func, got {:?}", other),
        };
        assert_eq!(main_func.name, "main");
        assert!(main_func.params.is_empty());
        assert_eq!(main_func.body.local_decls.len(), 2);
        assert_eq!(main_func.body.stmts.len(), 3);
    }

    #[test]
    fn empty_program() {
        let program = parse_source("");
        assert!(program.declarations.is_empty());
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn missing_semicolon() {
        let err = parse_source_err("int x");
        assert!(err.message.contains("Expected"));
    }

    #[test]
    fn missing_type() {
        let err = parse_source_err("x;");
        assert!(err.message.contains("type specifier"));
    }

    #[test]
    fn missing_rparen() {
        let err = parse_source_err("void main( {}");
        assert!(err.message.contains("Expected") || err.message.contains("expected"));
    }

    #[test]
    fn missing_rbrace() {
        let err = parse_source_err("void main() {");
        assert!(err.message.contains("Expected") || err.message.contains("expected"));
    }

    #[test]
    fn invalid_token_in_decl() {
        let err = parse_source_err("int 123;");
        assert!(err.message.contains("identifier") || err.message.contains("Expected"));
    }
}
