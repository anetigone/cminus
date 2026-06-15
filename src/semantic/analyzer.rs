use super::symbol_table::SymbolTable;

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    semantic_errors: Vec<String>,

}