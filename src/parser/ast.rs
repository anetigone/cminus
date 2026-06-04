use displaydoc::Display;

/// 类型说明符
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    Int,
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
