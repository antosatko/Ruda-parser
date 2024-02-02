use std::collections::HashMap;

const DEFAULT_ENTRY: &str = "entry";

use crate::{
    grammar::{self, Grammar},
    lexer::{self, Lexer, Token, TokenKinds},
};

pub struct Parser<'a> {
    pub text: &'a str,
    pub entry: String,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Parser<'a> {
        Parser {
            text,
            entry: DEFAULT_ENTRY.to_string(),
        }
    }

    pub fn set_text(&mut self, text: &'a str) {
        self.text = text;
    }

    pub(crate) fn parse(
        &self,
        grammar: &Grammar,
        lexer: &Lexer,
    ) -> Result<ParseResult, ParseError> {
        println!("{:?}", lexer.tokens.iter().map(|t| t.kind.clone()).collect::<Vec<TokenKinds>>());
        let mut cursor = Cursor { idx: 0 };
        let mut globals = Node::variables_from_grammar(&grammar.globals)?;
        let entry = self.parse_node(grammar, lexer, &self.entry, &mut cursor, &mut globals)?;

        Ok(ParseResult {
            entry,
            globals,
            text: self.text,
        })
    }

    fn parse_node(
        &self,
        grammar: &Grammar,
        lexer: &Lexer,
        name: &str,
        cursor: &mut Cursor,
        globals: &mut HashMap<String, VariableKind>,
    ) -> Result<Node, ParseError> {
        let mut node = Node::from_grammar(grammar, name)?;
        // In case the node fails to parse, we want to restore the cursor to its original position
        let cursor_clone = cursor.clone();
        let rules = match grammar.nodes.get(name) {
            Some(node) => &node.rules,
            None => return Err(ParseError::NodeNotFound(name.to_string())),
        };
        self.parse_rules(grammar, lexer, rules, cursor, globals, &cursor_clone, &mut node)?;
        cursor.idx -= 2;

        Ok(node)
    }

    fn parse_rules(
        &self,
        grammar: &Grammar,
        lexer: &Lexer,
        rules: &Vec<grammar::Rule>,
        cursor: &mut Cursor,
        globals: &mut HashMap<String, VariableKind>,
        cursor_clone: &Cursor,
        node: &mut Node,
    ) -> Result<(), ParseError> {
        let mut i = 0;
        while i < rules.len() {
            let rule = &rules[i];
            match rule {
                grammar::Rule::Is {
                    token,
                    rules,
                    parameters,
                } => {
                    match self.match_token(grammar, lexer, token, cursor, globals, cursor_clone)? {
                        TokenCompare::Is => {
                            // parameters go here
                            if rules.len() > 0 {
                                cursor.idx += 1;
                                self.parse_rules(grammar, lexer, rules, cursor, globals, cursor_clone, node)?;
                            }
                        }
                        TokenCompare::IsNot(err) => {
                            return Err(err);
                        }
                    };
                }
                grammar::Rule::Isnt {
                    token,
                    rules,
                    parameters,
                } => {
                    match self.match_token(grammar, lexer, token, cursor, globals, cursor_clone)? {
                        TokenCompare::Is => {
                            err(ParseError::ExpectedToNotBe(lexer.tokens[cursor.idx].kind.clone()), cursor, cursor_clone)?;
                        }
                        TokenCompare::IsNot(_) => {
                            self.parse_rules(grammar, lexer, rules, cursor, globals, cursor_clone, node)?;
                        }
                    }
                }
                grammar::Rule::IsOneOf { tokens } => {
                    let mut found = false;
                    for (token, rules, parameters) in tokens {
                        use TokenCompare::*;
                        match self.match_token(grammar, lexer, token, cursor, globals, cursor_clone)? {
                            Is => {
                                cursor.idx += 1;
                                found = true;
                                // parameters go here
                                self.parse_rules(grammar, lexer, rules, cursor, globals, cursor_clone, node)?;
                                break;
                            }
                            IsNot(_) => {}
                        }
                    }
                    if !found {
                        err(ParseError::ExpectedToken {
                            expected: TokenKinds::Text,
                            found: lexer.tokens[cursor.idx].kind.clone(),
                        }, cursor, cursor_clone)?;
                    }
                }
                grammar::Rule::Maybe {
                    token,
                    is,
                    isnt,
                    parameters,
                } => {
                    println!("//maybe//");
                    use TokenCompare::*;
                    match self.match_token(grammar, lexer, token, cursor, globals, cursor_clone)? {
                        Is => {
                            // parameters go here
                            println!("--------");
                            cursor.idx += 1;
                            self.parse_rules(grammar, lexer, is, cursor, globals, cursor_clone, node)?;
                        }
                        IsNot(err) => {
                            if isnt.len() == 0 {
                                cursor.idx -= 1;
                            }else {
                                self.parse_rules(grammar, lexer, isnt, cursor, globals, cursor_clone, node)?;
                            }
                        }
                    }
                }
                grammar::Rule::MaybeOneOf { is_one_of, isnt } => todo!(),
                grammar::Rule::While {
                    token,
                    rules,
                    parameters,
                } => {
                    println!("//while//");
                    while let TokenCompare::Is = self.match_token(grammar, lexer, token, cursor, globals, cursor_clone)? {
                        cursor.idx += 1;
                        self.parse_rules(grammar, lexer, rules, cursor, globals, cursor_clone, node)?;
                    }
                    cursor.idx -= 1;
                }
                grammar::Rule::Command { command } => todo!(),
            }
            i += 1;
            cursor.idx += 1;
        }
        Ok(())
    }

    fn match_token(
        &self,
        grammar: &Grammar,
        lexer: &Lexer,
        token: &grammar::MatchToken,
        cursor: &mut Cursor,
        globals: &mut HashMap<String, VariableKind>,
        cursor_clone: &Cursor,
    ) -> Result<TokenCompare, ParseError> {
        match token {
            grammar::MatchToken::Token(tok) => {
                let mut current_token = &lexer.tokens[cursor.idx];
                while current_token.kind == TokenKinds::Whitespace {
                    cursor.idx += 1;
                    current_token = &lexer.tokens[cursor.idx];
                }
                println!("{:?}", token);
                println!("{:?}", lexer.stringify(current_token));
                if *tok != current_token.kind {
                    return Ok(TokenCompare::IsNot(ParseError::ExpectedToken {
                                            expected: tok.clone(),
                                            found: current_token.kind.clone(),
                                        }));
                }
            }
            grammar::MatchToken::Node(node_name) => {
                println!("--{:?}--", node_name);
                let node = match self.parse_node(grammar, lexer, node_name, cursor, globals) {
                    Ok(node) => node,
                    Err(err) => return Ok(TokenCompare::IsNot(err)),
                };
            }
            grammar::MatchToken::Word(word) => {
                let mut current_token = &lexer.tokens[cursor.idx];
                while current_token.kind == TokenKinds::Whitespace {
                    cursor.idx += 1;
                    current_token = &lexer.tokens[cursor.idx];
                }
                println!("{:?}", token);
                println!("{:?}", lexer.stringify(current_token));
                if let TokenKinds::Text = current_token.kind {
                    if word != &lexer.stringify(&current_token) {
                        return Ok(TokenCompare::IsNot(ParseError::ExpectedToken {
                                                    expected: TokenKinds::Text,
                                                    found: current_token.kind.clone(),
                                                }));
                    }
                } else {
                    return Ok(TokenCompare::IsNot(ParseError::ExpectedWord {
                                            expected: word.clone(),
                                            found: current_token.kind.clone(),
                                        }));
                }
            }
            grammar::MatchToken::Enumerator(enumerator) => {
                println!("--{:?}--", enumerator);
                let enumerator = match grammar.enumerators.get(enumerator) {
                    Some(enumerator) => enumerator,
                    None => return Err(ParseError::EnumeratorNotFound(enumerator.to_string())),
                };
                let mut current_token = &lexer.tokens[cursor.idx];
                while current_token.kind == TokenKinds::Whitespace {
                    cursor.idx += 1;
                    current_token = &lexer.tokens[cursor.idx];
                }
                println!("{:?}", lexer.tokens[cursor.idx].kind);
                let mut i = 0; 
                let token = loop {
                    if i >= enumerator.values.len() {
                        return Ok(TokenCompare::IsNot(ParseError::EnumeratorNotFound(enumerator.name.clone())));
                    }
                    let token = &enumerator.values[i];
                    if let TokenCompare::Is = self.match_token(grammar, lexer, token, cursor, globals, cursor_clone)? {
                        break token;
                    }
                    i += 1;
                };

            }
        }
        Ok(TokenCompare::Is)
    }
}

enum TokenCompare {
    Is,
    IsNot(ParseError),
}

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub entry: Node,
    pub globals: HashMap<String, VariableKind>,
    pub text: &'a str,
}

#[derive(Debug)]
pub enum Nodes {
    Node(Node),
    Token(Token),
}

#[derive(Debug)]
pub struct Node {
    name: String,
    variables: HashMap<String, VariableKind>,
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
}

fn err(error: ParseError, cursor: &mut Cursor, cursor_clone: &Cursor) -> Result<(), ParseError> {
    *cursor = cursor_clone.clone();
    Err(error)
}

#[derive(Debug)]
pub enum VariableKind {
    Node(Option<Nodes>),
    NodeList(Vec<Nodes>),
    Boolean(bool),
    Number(i32),
    Count(i32),
}

pub enum ParseError {
    /// Parser not fully implemented - My fault
    ParserNotFullyImplemented,
    /// Node not found - Developer error
    NodeNotFound(String),
    /// Expected a token, found a token
    ExpectedToken {
        expected: TokenKinds,
        found: TokenKinds,
    },
    /// Expected a word, found a token
    ExpectedWord {
        expected: String,
        found: TokenKinds,
    },
    /// Enumerator not found - Developer error
    EnumeratorNotFound(String),
    /// Expected to not be
    ExpectedToNotBe(TokenKinds),
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::ParserNotFullyImplemented => write!(f, "Parser not fully implemented"),
            ParseError::NodeNotFound(name) => write!(f, "Node not found: {}", name),
            ParseError::ExpectedToken { expected, found } => {
                write!(f, "Expected token {:?}, found {:?}", expected, found)
            }
            ParseError::ExpectedWord { expected, found } => {
                write!(f, "Expected word {}, found {:?}", expected, found)
            }
            ParseError::EnumeratorNotFound(name) => write!(f, "Enumerator not found: {}", name),
            ParseError::ExpectedToNotBe(kind) => write!(f, "Expected to not be {:?}", kind),
        }
    }
}

/// A cursor is used to keep track of the current position in the token stream and other useful information
#[derive(Clone)]
struct Cursor {
    /// Current index in the token stream
    idx: usize,
}
