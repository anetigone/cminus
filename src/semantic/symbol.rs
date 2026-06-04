use crate::parser::ast::TypeSpec;

/// 符号表相关定义
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Var(VarInfo),     // 变量(包括数组)
    Func(FuncInfo),    // 函数
    Param(ParamInfo),   // 参数
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: TypeSpec, // 类型
    pub is_array: bool, // 是否为数组
    pub array_size: Option<usize>, // 数组大小（如果是数组）
    pub offset: Option<usize>, // 在栈帧中的偏移量
}

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub return_type: TypeSpec, // 返回类型
    pub params: Vec<ParamInfo>, // 参数列表
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub ty: TypeSpec, // 类型
    pub is_array: bool, // 是否为数组
    pub array_size: Option<usize>, // 数组大小（如果是数组）
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub kind: SymbolKind,
}