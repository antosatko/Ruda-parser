use std::collections::HashMap;

const DEFAULT_ENTRY: &str = "entry";


use crate::{
    grammar::{self, Grammar},
    lexer::{Lexer, TokenKinds},
};

pub struct Parser<'a> {
    pub text: &'a str,
    pub entry: String,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Parser<'a> {
        Parser { text, entry: DEFAULT_ENTRY.to_string() }
    }

    pub fn set_text(&mut self, text: &'a str) {
        self.text = text;
    }

    pub(crate) fn parse(&self, grammar: &Grammar, lexer: &Lexer) -> Result<ParseResult, ParseError> {
        let mut cursor = Cursor { idx: 0 };
        let globals = Node::variables_from_grammar(&grammar.globals)?;

        let mut entry = Node::parse(grammar, lexer, &self.entry, &mut cursor)?;

        Ok(ParseResult { entry, globals })
    }
}

pub struct ParseResult {
    pub entry: Node,
    pub globals: HashMap<String, VariableKind>,
}

pub struct Node {
    pub name: String,
    pub variables: HashMap<String, VariableKind>,
}

impl Node {
    pub fn new(name: String) -> Node {
        Node {
            name,
            variables: HashMap::new(),
        }
    }

    pub fn from_grammar(grammar: &Grammar, name: &str) -> Result<Node, ParseError> {
        let found = match grammar.nodes.get(name) {
            Some(node) => node,
            None => return Err(ParseError::NodeNotFound(name.to_string())),
        };
        let mut node = Node::new(found.name.clone());
        node.variables = Self::variables_from_grammar(&found.variables)?;
        Ok(node)
    }

    pub fn variables_from_grammar(
        variables: &HashMap<String, grammar::VariableKind>,
    ) -> Result<HashMap<String, VariableKind>, ParseError> {
        let mut result = HashMap::new();
        for (key, value) in variables {
            let var = match value {
                crate::grammar::VariableKind::Node => VariableKind::Node(None),
                crate::grammar::VariableKind::NodeList => VariableKind::NodeList(Vec::new()),
                crate::grammar::VariableKind::Boolean => VariableKind::Boolean(false),
                crate::grammar::VariableKind::Number => VariableKind::Number(0),
                crate::grammar::VariableKind::Count => VariableKind::Count(0),
            };
            result.insert(key.clone(), var);
        }
        Ok(result)
    }

    pub fn parse(grammar: &Grammar, lexer: &Lexer, name: &str, cursor: &mut Cursor) -> Result<Node, ParseError> {
        let mut node = Self::from_grammar(grammar, name)?;
        

        Ok(node)
    }
}

pub enum VariableKind {
    Node(Option<Node>),
    NodeList(Vec<Node>),
    Boolean(bool),
    Number(i32),
    Count(i32),
}

pub enum ParseError {
    ParserNotFullyImplemented,
    NodeNotFound(String),
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::ParserNotFullyImplemented => write!(f, "Parser not fully implemented"),
            ParseError::NodeNotFound(name) => write!(f, "Node not found: {}", name),
        }
    }
}

/// A cursor is used to keep track of the current position in the token stream and other useful information
struct Cursor {
    /// Current index in the token stream
    idx: usize,
}