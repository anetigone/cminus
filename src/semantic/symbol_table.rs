use std::collections::HashMap;

use super::symbol::Symbol;

#[derive(Debug, Clone)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
}

/// 单个作用域的符号映射
pub struct Scope {
    kind: ScopeKind,
    parent: Option<Box<Scope>>,
    symbols: HashMap<String, Symbol>,
}

/// 符号表，支持嵌套作用域
pub struct SymbolTable {
    current: Scope,
    depth: usize, // 当前作用域深度
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            current: Scope {
                kind: ScopeKind::Global,
                parent: None,
                symbols: HashMap::new(),
            },
            depth: 0,
        }
    }
}

impl SymbolTable {
    /// 插入符号，如果当前作用域已存在同名符号则返回 None
    pub fn insert(&mut self, symbol: Symbol) -> Option<()> {
        if self.current.symbols.contains_key(&symbol.name) {
            None
        } else {
            self.current.symbols.insert(symbol.name.clone(), symbol);
            Some(())
        }
    }

    /// 查找符号，逐层向上查找，找不到返回 None
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope = &self.current;
        loop {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
            scope = match &scope.parent {
                Some(parent) => parent,
                None => break,
            };
        }
        None
    }
}
