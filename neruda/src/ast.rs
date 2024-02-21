//! A module for generating Neruda AST after it is done parsing
use rparse::parser::*;

/// Returns all imports in tree 
///
/// Always make sure to get all the imports of each file before analysis
pub fn find_imports(tree: &Result<ParseResult, ParseError>) -> Vec<ImportKind> {
    let mut result = Vec::new();

    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Core(String),
    Std(String),
    File(String),
}