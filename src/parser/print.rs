use super::ast::*;

/// 将语法树以树形结构打印为字符串
pub fn print_tree(program: &Program) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (i, decl) in program.declarations.iter().enumerate() {
        let is_last = i == program.declarations.len() - 1;
        print_declaration(decl, "", is_last, &mut lines);
    }
    lines.join("\n")
}

fn print_declaration(decl: &Declaration, prefix: &str, is_last: bool, lines: &mut Vec<String>) {
    let connector = if is_last { "└──" } else { "├──" };
    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    match decl {
        Declaration::Var(var) => {
            lines.push(format!(
                "{}{}VarK: {} {}",
                prefix, connector, var.name, type_str(&var.type_spec)
            ));
            if let Some(size) = var.array_size {
                lines.push(format!("{}└──ArraySize: {}", child_prefix, size));
            }
        }
        Declaration::Func(func) => {
            lines.push(format!(
                "{}{}FuncK: {} -> {}",
                prefix, connector, func.name, type_str(&func.return_type)
            ));

            // 返回类型
            lines.push(format!("{}├──ReturnType: {}", child_prefix, type_str(&func.return_type)));

            // 参数
            if func.params.is_empty() {
                lines.push(format!("{}├──ParamsK: VoidK", child_prefix));
            } else {
                lines.push(format!("{}├──ParamsK", child_prefix));
                for (j, param) in func.params.iter().enumerate() {
                    let param_last = j == func.params.len() - 1;
                    let param_connector = if param_last { "└──" } else { "├──" };
                    let param_prefix = format!("{}│   ", child_prefix);
                    lines.push(format!(
                        "{}{}ParamK: {} {}",
                        param_prefix, param_connector, param.name, type_str(&param.type_spec)
                    ));
                }
            }

            // 函数体
            lines.push(format!("{}└──CompK", child_prefix));
            let body_prefix = format!("{}    ", child_prefix);
            print_compound_stmt(&func.body, &body_prefix, lines);
        }
    }
}

fn print_compound_stmt(comp: &CompoundStmt, prefix: &str, lines: &mut Vec<String>) {
    let total = comp.local_decls.len() + comp.stmts.len();
    if total == 0 {
        return;
    }

    for (i, decl) in comp.local_decls.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = if is_last { "└──" } else { "├──" };
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        match decl {
            Declaration::Var(var) => {
                lines.push(format!(
                    "{}{}VarDeclK: {} {}",
                    prefix, connector, var.name, type_str(&var.type_spec)
                ));
                if let Some(size) = var.array_size {
                    lines.push(format!("{}└──ArraySize: {}", child_prefix, size));
                }
            }
            Declaration::Func(_) => {
                print_declaration(decl, prefix, is_last, lines);
            }
        }
    }

    for (i, stmt) in comp.stmts.iter().enumerate() {
        let idx = comp.local_decls.len() + i;
        let is_last = idx == total - 1;
        print_stmt(stmt, prefix, is_last, lines);
    }
}

fn print_stmt(stmt: &Stmt, prefix: &str, is_last: bool, lines: &mut Vec<String>) {
    let connector = if is_last { "└──" } else { "├──" };
    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    match stmt {
        Stmt::Empty => {
            lines.push(format!("{}{}EmptyK", prefix, connector));
        }
        Stmt::Expression(None) => {
            lines.push(format!("{}{}ExprK: (empty)", prefix, connector));
        }
        Stmt::Expression(Some(expr)) => {
            lines.push(format!("{}{}ExprK", prefix, connector));
            print_expr(expr, &child_prefix, true, lines);
        }
        Stmt::Compound(comp) => {
            lines.push(format!("{}{}CompK", prefix, connector));
            print_compound_stmt(comp, &child_prefix, lines);
        }
        Stmt::Selection(sel) => {
            lines.push(format!("{}{}IfK", prefix, connector));
            lines.push(format!("{}├──CondK", child_prefix));
            let cond_prefix = format!("{}│   ", child_prefix);
            print_expr(&sel.condition, &cond_prefix, true, lines);
            lines.push(format!("{}├──ThenK", child_prefix));
            let then_prefix = format!("{}│   ", child_prefix);
            print_stmt(&sel.then_brach, &then_prefix, true, lines);
            if let Some(else_br) = &sel.else_brach {
                lines.push(format!("{}└──ElseK", child_prefix));
                let else_prefix = format!("{}    ", child_prefix);
                print_stmt(else_br, &else_prefix, true, lines);
            }
        }
        Stmt::Iteration(iter) => {
            lines.push(format!("{}{}WhileK", prefix, connector));
            lines.push(format!("{}├──CondK", child_prefix));
            let cond_prefix = format!("{}│   ", child_prefix);
            print_expr(&iter.condition, &cond_prefix, true, lines);
            lines.push(format!("{}└──BodyK", child_prefix));
            let body_prefix = format!("{}    ", child_prefix);
            print_stmt(&iter.body, &body_prefix, true, lines);
        }
        Stmt::Return(None) => {
            lines.push(format!("{}{}ReturnK: void", prefix, connector));
        }
        Stmt::Return(Some(expr)) => {
            lines.push(format!("{}{}ReturnK", prefix, connector));
            print_expr(expr, &child_prefix, true, lines);
        }
    }
}

fn print_expr(expr: &Expression, prefix: &str, is_last: bool, lines: &mut Vec<String>) {
    let connector = if is_last { "└──" } else { "├──" };
    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    match expr {
        Expression::Assign { lvar, expr } => {
            lines.push(format!("{}{}AssignK: {}", prefix, connector, lvar.name));
            print_expr(expr, &child_prefix, true, lines);
        }
        Expression::BinOp { op, left, right } => {
            lines.push(format!("{}{}OpK: {}", prefix, connector, op_str(op)));
            print_expr(left, &child_prefix, false, lines);
            print_expr(right, &child_prefix, true, lines);
        }
        Expression::Call { name, args, .. } => {
            lines.push(format!("{}{}CallK: {}", prefix, connector, name));
            if args.is_empty() {
                lines.push(format!("{}└──ArgsK: (empty)", child_prefix));
            } else {
                lines.push(format!("{}└──ArgsK", child_prefix));
                let args_prefix = format!("{}    ", child_prefix);
                for (i, arg) in args.iter().enumerate() {
                    let arg_is_last = i == args.len() - 1;
                    print_expr(arg, &args_prefix, arg_is_last, lines);
                }
            }
        }
        Expression::LVar(lvar) => {
            if let Some(idx) = &lvar.index {
                lines.push(format!("{}{}ArrayK: {}", prefix, connector, lvar.name));
                print_expr(idx, &child_prefix, true, lines);
            } else {
                lines.push(format!("{}{}IdK: {}", prefix, connector, lvar.name));
            }
        }
        Expression::Number(n) => {
            lines.push(format!("{}{}ConstK: {}", prefix, connector, n));
        }
        Expression::String(s) => {
            lines.push(format!("{}{}StringK: \"{}\"", prefix, connector, s));
        }
    }
}

fn type_str(t: &TypeSpec) -> &'static str {
    match t {
        TypeSpec::Int => "int",
        TypeSpec::Void => "void",
    }
}

fn op_str(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
    }
}
