use std::collections::HashMap;

use super::symbol::Symbol;

#[derive(Debug, Clone)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
}

/// 单个作用域层的符号映射
#[derive(Debug, Clone)]
pub struct Scope {
    kind: ScopeKind,
    symbols: HashMap<String, Symbol>,
}

/// 符号表：用 Vec 维护作用域栈，栈顶为当前作用域。
/// 栈底永远是 Global，不会被弹出。
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                kind: ScopeKind::Global,
                symbols: HashMap::new(),
            }],
        }
    }

    /// 当前作用域深度（全局为 0）
    pub fn depth(&self) -> usize {
        self.scopes.len() - 1
    }

    /// 当前作用域种类
    pub fn current_kind(&self) -> &ScopeKind {
        &self.scopes.last().unwrap().kind
    }
}

impl SymbolTable {
    /// 插入符号，如果当前作用域已存在同名符号则返回 None
    pub fn insert(&mut self, symbol: Symbol) -> Option<()> {
        let current = self.scopes.last_mut().unwrap();
        if current.symbols.contains_key(&symbol.name) {
            None
        } else {
            current.symbols.insert(symbol.name.clone(), symbol);
            Some(())
        }
    }

    /// 查找符号，从当前作用域逐层向上查找，找不到返回 None
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.symbols.get(name))
    }

    /// 仅在当前作用域查找，用于检查重定义
    pub fn lookup_current(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().unwrap().symbols.get(name)
    }

    /// 进入新作用域
    pub fn enter_scope(&mut self, kind: ScopeKind) {
        self.scopes.push(Scope {
            kind,
            symbols: HashMap::new(),
        });
    }

    /// 离开当前作用域，全局作用域不可弹出，返回 None
    pub fn leave_scope(&mut self) -> Option<()> {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            Some(())
        } else {
            None
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
