use displaydoc::Display;
use std::fmt;

/// 类型说明符
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum TypeSpec {
    /// int
    Int,
    /// void
    Void,
}

/// 程序：声明列表
#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

/// 声明
#[derive(Debug, Clone)]
pub enum Declaration {
    Var(VarDecl),
    Func(FuncDecl),
}

/// 变量声明
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub type_spec: TypeSpec,
    pub name: String,
    pub array_size: Option<u32>, //None表示不是数组,Some表示数组大小
}

/// 函数声明
#[derive(Debug, Clone)]
pub struct FuncDecl {
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
    pub array_size: Option<u32>, //None表示不是数组,Some表示数组大小
}

/// 复合语句
#[derive(Debug, Clone)]
pub struct CompoundStmt {
    pub local_decls: Vec<Declaration>,
    pub stmts: Vec<Stmt>,
}

/// 语句
#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Option<Expression>), // expression-stmt,None表示空表达式
    Compound(CompoundStmt),         // compound-stmt
    Selection(SelectionStmt),       // selection-stmt
    Iteration(IterationStmt),       // iteration-stmt
    Return(Option<Expression>),     // return-stmt,None表示没有返回值
    Empty,                          // 空语句
}

/// if语句
#[derive(Debug, Clone)]
pub struct SelectionStmt {
    pub condition: Expression,
    pub then_brach: Box<Stmt>,
    pub else_brach: Option<Box<Stmt>>, // None表示没有else分支
}

/// while语句
#[derive(Debug, Clone)]
pub struct IterationStmt {
    pub condition: Expression,
    pub body: Box<Stmt>,
}

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    Assign {
        lvar: LVar,
        expr: Box<Expression>,
    },
    BinOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    LVar(LVar),
    Number(i32),
    String(String),
}

/// 变量(左值）
#[derive(Debug, Clone)]
pub struct LVar {
    pub name: String,
    pub index: Option<Box<Expression>>, //None表示普通变量,Some表示数组下标表达式
}

/// 二元运算符
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum BinaryOp {
    /// +
    Add,
    /// -
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// ==
    Eq,
    /// !=
    Ne,
    /// <
    Lt,
    /// >
    Gt,
    /// <=
    Le,
    /// >=
    Ge,
}

// ===== Display implementations =====

const INDENT: &str = "    ";

/// 辅助：写入指定层级的缩进
fn write_indent(f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        f.write_str(INDENT)?;
    }
    Ok(())
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for decl in &self.declarations {
            writeln!(f, "{decl}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Declaration::Var(v) => write!(f, "{v}"),
            Declaration::Func(func) => write!(f, "{func}"),
        }
    }
}

impl fmt::Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.type_spec, self.name)?;
        if let Some(size) = self.array_size {
            write!(f, "[{size}]")?;
        }
        write!(f, ";")
    }
}

impl fmt::Display for FuncDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}(", self.return_type, self.name)?;
        if self.params.is_empty() {
            write!(f, "void")?;
        } else {
            let mut first = true;
            for p in &self.params {
                if !first {
                    write!(f, ", ")?;
                }
                first = false;
                write!(f, "{p}")?;
            }
        }
        writeln!(f, ")")?;
        write_compound_stmt(f, &self.body, 0)
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.type_spec, self.name)?;
        if self.array_size.is_some() {
            write!(f, "[]")?;
        }
        Ok(())
    }
}

/// CompoundStmt 需要缩进上下文，所以使用独立函数
fn write_compound_stmt(
    f: &mut fmt::Formatter<'_>,
    cs: &CompoundStmt,
    depth: usize,
) -> fmt::Result {
    for decl in &cs.local_decls {
        write_indent(f, depth + 1)?;
        writeln!(f, "{decl}")?;
    }
    for stmt in &cs.stmts {
        write_stmt(f, stmt, depth + 1)?;
    }
    Ok(())
}

fn write_stmt(f: &mut fmt::Formatter<'_>, stmt: &Stmt, depth: usize) -> fmt::Result {
    match stmt {
        Stmt::Empty => {
            write_indent(f, depth)?;
            writeln!(f, ";")
        }
        Stmt::Expression(expr) => {
            write_indent(f, depth)?;
            match expr {
                Some(e) => writeln!(f, "{e};"),
                None => writeln!(f, ";"),
            }
        }
        Stmt::Compound(cs) => {
            write_compound_stmt(f, cs, depth)
        }
        Stmt::Selection(sel) => {
            write_indent(f, depth)?;
            writeln!(f, "if ({})", sel.condition)?;
            write_stmt(f, &sel.then_brach, depth)?;
            if let Some(else_br) = &sel.else_brach {
                write_indent(f, depth)?;
                writeln!(f, "else")?;
                write_stmt(f, else_br, depth)?;
            }
            Ok(())
        }
        Stmt::Iteration(iter) => {
            write_indent(f, depth)?;
            writeln!(f, "while ({})", iter.condition)?;
            write_stmt(f, &iter.body, depth)
        }
        Stmt::Return(expr) => {
            write_indent(f, depth)?;
            match expr {
                Some(e) => writeln!(f, "return {e};"),
                None => writeln!(f, "return;"),
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Assign { lvar, expr } => write!(f, "{lvar} = {expr}"),
            Expression::BinOp { op, left, right } => write!(f, "({left} {op} {right})"),
            Expression::Call { name, args } => {
                write!(f, "{name}(")?;
                let mut first = true;
                for a in args {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{a}")?;
                }
                write!(f, ")")
            }
            Expression::LVar(v) => write!(f, "{v}"),
            Expression::Number(n) => write!(f, "{n}"),
            Expression::String(s) => write!(f, "\"{s}\""),
        }
    }
}

impl fmt::Display for LVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(idx) = &self.index {
            write!(f, "[{idx}]")?;
        }
        Ok(())
    }
}
