//! A module for generating Neruda AST after it is done parsing
use rparse::{grammar::validator::TokenErrors, lexer::TokenKinds, parser::*};

/// Returns all imports in tree 
///
/// Always make sure to get all the imports of each file before analysis
pub fn find_imports(tree: &ParseResult, text: &str) -> Vec<ImportKind> {
    let mut result = Vec::new();
    let imports = match tree.globals.get("imports") {
        Some(VariableKind::NodeList(imports)) => imports,
        _ => return result,
    };
    for node in imports {
        match node {
            Nodes::Node(_) => continue,
            Nodes::Token(tok) => match &tok.kind {
                TokenKinds::Complex(txt) => {
                    if txt != "string" {
                        continue;
                    }
                    let content = &text[tok.index + 1..tok.index + tok.len - 1];
                    if content.starts_with("#") {
                        result.push(ImportKind::Core(content[1..].to_string()));
                    } else if content.starts_with("%") {
                        result.push(ImportKind::Runtime(content[1..].to_string()));
                    } else {
                        result.push(ImportKind::File(content.to_string()));
                    }
                }
                _ => continue,
            }
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportKind {
    /// Importing a module from the standard library
    Core(String),
    /// Importing definitions from a runtime owner
    Runtime(String),
    /// Importing a file from the file system
    File(String),
}