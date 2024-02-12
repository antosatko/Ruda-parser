use std::collections::HashMap;

use crate::lexer::TokenKinds;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Grammar {
    pub nodes: HashMap<String, Node>,
    pub enumerators: HashMap<String, Enumerator>,
    pub globals: HashMap<String, VariableKind>,
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar {
            nodes: HashMap::new(),
            enumerators: HashMap::new(),
            globals: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.name.clone(), node);
    }
}

/// A collection of rules
pub type Rules = Vec<Rule>;

/// A rule defines how a token will be matched and what will happen if it is matched
///
/// It also contains parameters that can be used if the rule is matched
///
/// Special kind of rules are commands that can be executed without matching a token
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Rule {
    /// Matches a token
    ///
    /// If the token is matched, the rules will be executed
    ///
    /// If the token is not matched, the node will end with an error
    Is {
        token: MatchToken,
        rules: Rules,
        parameters: Vec<Parameters>,
    },
    /// Matches a token
    ///
    /// If the token is matched, the node will end with an error
    ///
    /// If the token is not matched, the rules will be executed
    Isnt {
        token: MatchToken,
        rules: Rules,
        parameters: Vec<Parameters>,
    },
    /// Matches one of the tokens
    ///
    /// If one of the tokens is matched, the rules will be executed
    ///
    /// If none of the tokens is matched, the node will end with an error
    IsOneOf { tokens: Vec<OneOf> },
    /// Matches a token
    ///
    /// If the token is matched, the rules will be executed
    ///
    /// If the token is not matched, the rules for the else branch will be executed
    Maybe {
        /// Token that will be matched
        token: MatchToken,
        /// Rules that will be executed if the token is matched
        is: Rules,
        /// Rules that will be executed if the token is not matched
        isnt: Rules,
        /// Parameters that can be used if the token is matched
        parameters: Vec<Parameters>,
    },
    /// Matches one of the tokens
    ///
    /// If one of the tokens is matched, the rules will be executed
    ///
    /// If none of the tokens is matched, the rules for the else branch will be executed
    MaybeOneOf {
        /// Tokens that will be matched
        is_one_of: Vec<(MatchToken, Rules, Vec<Parameters>)>,
        /// Rules that will be executed if none of the tokens is matched
        isnt: Rules,
    },
    /// Matches a token
    ///
    /// If the token is matched, the rules will be executed
    ///
    /// After the rules are executed, the token will be matched again
    /// and the rules will be executed again (if the token is matched)
    While {
        token: MatchToken,
        rules: Rules,
        /// Parameters that can be used if the token is matched
        ///
        /// The parameters will be used once every time the token is matched
        parameters: Vec<Parameters>,
    },
    /// Loop that will be executed until a break command is executed
    Loop { rules: Rules },
    /// Searches in the tokens until a token is matched
    Until {
        token: MatchToken,
        rules: Rules,
        parameters: Vec<Parameters>,
    },
    /// Searches in the tokens until one of the tokens is matched
    UntilOneOf { tokens: Vec<OneOf> },
    /// Performs a command
    ///
    /// The command will be executed without matching a token
    Command { command: Commands },
}

/// One of the tokens that will be matched
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OneOf {
    pub token: MatchToken,
    pub rules: Rules,
    pub parameters: Vec<Parameters>,
}

/// Commands that can be executed
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Commands {
    /// Compares two variables/numbers and executes rules if the comparison is true
    Compare {
        /// Left side of the comparison
        left: String,
        /// Right side of the comparison
        right: String,
        /// Comparison operator
        comparison: Comparison,
        /// Rules that will be executed if the comparison is true
        rules: Rules,
    },
    /// Returns an error from node
    Error {
        message: String,
    },
    HardError {
        set: bool,
    },
    Goto {
        label: String,
    },
    Label {
        name: String,
    },
    Print {
        message: String,
    },
}

/// Comparison operators
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Comparison {
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// >
    GreaterThan,
    /// <
    LessThan,
    /// >=
    GreaterThanOrEqual,
    /// <=
    LessThanOrEqual,
}

/// A token that will be matched
///
/// Can be a token kind or a node name
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MatchToken {
    /// A token kind
    Token(TokenKinds),
    /// A node name
    Node(String),
    /// A constant word
    Word(String),
    /// An enumerator
    Enumerator(String),
    /// Any token
    Any,
}

/// A node is a collection of rules that will be executed when the node is matched
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    /// Name of the node
    pub name: String,
    /// Rules that will be executed when the node is matched
    pub rules: Rules,
    /// Variables that can be used in the node and will be accessible from the outside
    pub variables: HashMap<String, VariableKind>,
}

/// A variable that can be used in a node
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableKind {
    /// Holds a single node
    Node,
    /// Holds a list of nodes
    NodeList,
    /// Holds a boolean
    Boolean,
    /// Holds a number
    Number,
}

/// Parameters that can be used on a rule if it is matched
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Parameters {
    /// Sets a variable to a value
    Set(String),
    /// Sets a global variable to a value
    Global(String),
    /// Adds 1 to a variable of type Count
    Increment(String),
    /// Subtracts 1 from a variable of type Count
    Decrement(String),
    /// Adds 1 to a global variable of type Count
    IncrementGlobal(String),
    /// Sets a variable to true
    True(String),
    /// Sets a variable to false
    False(String),
    /// Sets a global variable to true
    TrueGlobal(String),
    /// Sets a global variable to false
    FalseGlobal(String),
    /// Prints string
    Print(String),
    /// Prints current token or variable
    Debug(Option<String>),
    /// Goes back in rules
    Back(u8),
    /// Returns from node
    Return,
    /// Breaks from rule blocks(n)
    Break(usize),
    /// If the node ends with an error, it will be a hard error
    /// resulting in the parent node to also end with an error
    ///
    /// This is a way of telling the parser that the current node MUST match
    ///
    /// This is useful for using nodes in optional rules
    HardError(bool),
    /// Sets the current node to the label with the given name
    Goto(String),
    /// Hints to the parser that the node starts here
    ///
    /// This should be used at the start of every node
    /// because it will prevent the parser from counting
    /// whitespace in front of the node
    NodeStart,
    /// Hints to the parser that the node ends here
    NodeEnd,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Enumerator {
    pub name: String,
    pub values: Vec<MatchToken>,
}

/// validation module for grammar that is otherwise dynamically typed
///
/// This module is used to validate the grammar and make sure that it is correct
///
/// The grammar is validated by checking if the rules are correct and if the variables are used correctly
///
/// > note: Grammar errors have caused me a lot of headache in the past so using this module is highly recommended
pub mod validator {
    use super::*;
    use crate::lexer::*;

    impl Lexer {
        pub fn validate_tokens(&self, result: &mut ValidationResult) {
            let mut tokens = Vec::new();
            for token in &self.token_kinds {
                // tokens that have already been validated can be ignored
                if tokens.contains(token) {
                    continue;
                }
                tokens.push(token.clone());
                // check for collisions
                if self.token_kinds.iter().filter(|t| *t == token).count() > 1 {
                    result.errors.push(ValidationError {
                        kind: ValidationErrors::TokenCollision(token.clone()),
                        node_name: "__lexer__".to_string(),
                    });
                }
                // check if token is empty
                if token.is_empty() {
                    result.errors.push(ValidationError {
                        kind: ValidationErrors::EmptyToken,
                        node_name: "__lexer__".to_string(),
                    });
                }
                // check if it starts with a number
                let first = token.chars().next().unwrap();
                if first.is_numeric() {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarnings::UnusualToken(
                            token.clone(),
                            TokenErrors::StartsNumeric,
                        ),
                        node_name: "__lexer__".to_string(),
                    });
                }

                // check if it contains a whitespace
                if token.chars().any(|c| c.is_whitespace()) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarnings::UnusualToken(
                            token.clone(),
                            TokenErrors::ContainsWhitespace,
                        ),
                        node_name: "__lexer__".to_string(),
                    });
                }

                // check if it is longer than 2 characters
                if token.len() > 2 {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarnings::UnusualToken(token.clone(), TokenErrors::TooLong),
                        node_name: "__lexer__".to_string(),
                    });
                }

                // check if it is not ascii
                if !token.chars().all(|c| c.is_ascii()) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarnings::UnusualToken(
                            token.clone(),
                            TokenErrors::NotAscii,
                        ),
                        node_name: "__lexer__".to_string(),
                    });
                }
            }
        }
    }

    impl Grammar {
        /// Validates the grammar
        pub fn validate(&self, lexer: &Lexer) -> ValidationResult {
            let mut result = ValidationResult::new();
            lexer.validate_tokens(&mut result);

            for node in self.nodes.values() {
                self.validate_node(node, lexer, &mut result);
            }

            result
        }

        pub fn validate_node(&self, node: &Node, lexer: &Lexer, result: &mut ValidationResult) {
            let mut laf = LostAndFound::new();
            for rule in &node.rules {
                self.validate_rule(rule, node, lexer, &mut laf, result);
            }
            laf.pass(result, &node.name);
        }

        pub fn validate_rule(
            &self,
            rule: &Rule,
            node: &Node,
            lexer: &Lexer,
            laf: &mut LostAndFound,
            result: &mut ValidationResult,
        ) {
            match rule {
                Rule::Is {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, laf, result);
                    self.validate_parameters(parameters, node, laf, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::Isnt {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, laf, result);
                    self.validate_parameters(parameters, node, laf, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::IsOneOf { tokens } => {
                    for one_of in tokens {
                        self.validate_token(&one_of.token, node, lexer, laf, result);
                        self.validate_parameters(&one_of.parameters, node, laf, result);
                        for rule in &one_of.rules {
                            self.validate_rule(rule, node, lexer, laf, result);
                        }
                    }
                }
                Rule::Maybe {
                    token,
                    is,
                    isnt,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, laf, result);
                    self.validate_parameters(parameters, node, laf, result);
                    for rule in is {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                    for rule in isnt {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::MaybeOneOf { is_one_of, isnt } => {
                    for (token, rules, parameters) in is_one_of {
                        self.validate_token(token, node, lexer, laf, result);
                        self.validate_parameters(parameters, node, laf, result);
                        for rule in rules {
                            self.validate_rule(rule, node, lexer, laf, result);
                        }
                    }
                    for rule in isnt {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::While {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, laf, result);
                    self.validate_parameters(parameters, node, laf, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::Loop { rules } => {
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::Until {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, laf, result);
                    self.validate_parameters(parameters, node, laf, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, laf, result);
                    }
                }
                Rule::UntilOneOf { tokens } => {
                    for one_of in tokens {
                        self.validate_token(&one_of.token, node, lexer, laf, result);
                        self.validate_parameters(&one_of.parameters, node, laf, result);
                        for rule in &one_of.rules {
                            self.validate_rule(rule, node, lexer, laf, result);
                        }
                    }
                }
                Rule::Command { command } => match command {
                    Commands::Compare {
                        left,
                        right,
                        comparison: _,
                        rules,
                    } => {
                        match self.globals.get(left) {
                            Some(var) => match var {
                                VariableKind::Number => (),
                                _ => result.errors.push(ValidationError {
                                    kind: ValidationErrors::CantUseVariable(left.clone()),
                                    node_name: node.name.clone(),
                                }),
                            },
                            None => {
                                result.errors.push(ValidationError {
                                    kind: ValidationErrors::GlobalNotFound(left.clone()),
                                    node_name: node.name.clone(),
                                });
                            }
                        }
                        match self.globals.get(right) {
                            Some(var) => match var {
                                VariableKind::Number => (),
                                _ => result.errors.push(ValidationError {
                                    kind: ValidationErrors::CantUseVariable(right.clone()),
                                    node_name: node.name.clone(),
                                }),
                            },
                            None => {
                                result.errors.push(ValidationError {
                                    kind: ValidationErrors::GlobalNotFound(right.clone()),
                                    node_name: node.name.clone(),
                                });
                            }
                        }
                        for rule in rules {
                            self.validate_rule(rule, node, lexer, laf, result);
                        }
                    }
                    Commands::Error { message: _ } => (),
                    Commands::HardError { set: _ } => (),
                    Commands::Goto { label } => {
                        laf.lost_labels.push(label.clone());
                    }
                    Commands::Label { name } => {
                        if laf.found_labels.contains(&name) {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::DuplicateLabel(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                        laf.found_labels.push(name.clone());
                    }
                    Commands::Print { message: _ } => (),
                },
            }
        }

        pub fn validate_token(
            &self,
            token: &MatchToken,
            node: &Node,
            lexer: &Lexer,
            _laf: &mut LostAndFound,
            result: &mut ValidationResult,
        ) {
            match token {
                MatchToken::Node(name) => {
                    if !self.nodes.contains_key(name) {
                        result.errors.push(ValidationError {
                            kind: ValidationErrors::NodeNotFound(name.clone()),
                            node_name: node.name.clone(),
                        });
                    }
                }
                MatchToken::Enumerator(enumerator) => {
                    if !self.enumerators.contains_key(enumerator) {
                        result.errors.push(ValidationError {
                            kind: ValidationErrors::EnumeratorNotFound(enumerator.clone()),
                            node_name: node.name.clone(),
                        });
                    }
                }
                MatchToken::Any => result.warnings.push(ValidationWarning {
                    kind: ValidationWarnings::UsedDepricated(Depricated::Any),
                    node_name: node.name.clone(),
                }),
                MatchToken::Token(kind) => match kind {
                    TokenKinds::Token(txt) => {
                        if txt.is_empty() {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::EmptyToken,
                                node_name: node.name.clone(),
                            });
                            return;
                        }
                        // check if token is in the lexer
                        if !lexer.token_kinds.iter().any(|k| k == txt) {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::TokenNotFound(txt.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        pub fn validate_parameters(
            &self,
            parameters: &Vec<Parameters>,
            node: &Node,
            laf: &mut LostAndFound,
            result: &mut ValidationResult,
        ) {
            for parameter in parameters {
                match parameter {
                    Parameters::Set(name) => match node.variables.get(name) {
                        Some(var) => match var {
                            VariableKind::Node => (),
                            VariableKind::NodeList => (),
                            VariableKind::Boolean => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Number => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::VariableNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::Global(name) => match self.globals.get(name) {
                        Some(var) => match var {
                            VariableKind::Node => (),
                            VariableKind::NodeList => (),
                            VariableKind::Boolean => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Number => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::GlobalNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::Increment(name) => match node.variables.get(name) {
                        Some(var) => match var {
                            VariableKind::Number => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Boolean => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::VariableNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::Decrement(name) => match node.variables.get(name) {
                        Some(var) => match var {
                            VariableKind::Number => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Boolean => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::VariableNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::IncrementGlobal(name) => match self.globals.get(name) {
                        Some(var) => match var {
                            VariableKind::Number => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Boolean => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::GlobalNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::True(name) => match node.variables.get(name) {
                        Some(var) => match var {
                            VariableKind::Boolean => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Number => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::VariableNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::False(name) => match node.variables.get(name) {
                        Some(var) => match var {
                            VariableKind::Boolean => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Number => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::VariableNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::TrueGlobal(name) => match self.globals.get(name) {
                        Some(var) => match var {
                            VariableKind::Boolean => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Number => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::GlobalNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::FalseGlobal(name) => match self.globals.get(name) {
                        Some(var) => match var {
                            VariableKind::Boolean => (),
                            VariableKind::Node => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::NodeList => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                            VariableKind::Number => result.errors.push(ValidationError {
                                kind: ValidationErrors::CantUseVariable(name.clone()),
                                node_name: node.name.clone(),
                            }),
                        },
                        None => {
                            result.errors.push(ValidationError {
                                kind: ValidationErrors::GlobalNotFound(name.clone()),
                                node_name: node.name.clone(),
                            });
                        }
                    },
                    Parameters::Print(_) => {
                        result.warnings.push(ValidationWarning {
                            kind: ValidationWarnings::UsedPrint,
                            node_name: node.name.clone(),
                        });
                    }
                    Parameters::Debug(node_option) => {
                        match node_option {
                            Some(name) => match node.variables.get(name) {
                                Some(_) => (),
                                None => {
                                    result.errors.push(ValidationError {
                                        kind: ValidationErrors::VariableNotFound(name.clone()),
                                        node_name: node.name.clone(),
                                    });
                                }
                            },
                            None => (),
                        }
                        result.warnings.push(ValidationWarning {
                            kind: ValidationWarnings::UsedDebug,
                            node_name: node.name.clone(),
                        });
                    }
                    Parameters::Back(_) => {
                        result.warnings.push(ValidationWarning {
                            kind: ValidationWarnings::UsedDepricated(Depricated::Back),
                            node_name: node.name.clone(),
                        });
                    }
                    Parameters::Return => (),
                    Parameters::Break(_) => (),
                    Parameters::HardError(_) => (),
                    Parameters::Goto(label) => {
                        laf.lost_labels.push(label.clone());
                    }
                    Parameters::NodeStart => (),
                    Parameters::NodeEnd => (),
                }
            }
        }
    }

    pub struct ValidationResult {
        pub errors: Vec<ValidationError>,
        pub warnings: Vec<ValidationWarning>,
    }

    impl ValidationResult {
        pub fn new() -> Self {
            Self {
                errors: Vec::new(),
                warnings: Vec::new(),
            }
        }

        /// Returns true if there are no errors and no warnings
        ///
        /// Choose this over `pass` for production code
        ///
        /// ```rust
        /// let result = grammar.validate(&lexer);
        /// if result.success() {
        ///    println!("Grammar is valid and production ready");
        /// } else {
        ///   println!("Grammar is not valid");
        /// }
        /// ```
        ///
        pub fn success(&self) -> bool {
            self.errors.is_empty() && self.warnings.is_empty()
        }

        /// Returns true if there are no errors
        ///
        /// Choose this over `success` for testing code
        ///
        /// ```rust
        /// let result = grammar.validate(&lexer);
        /// if result.pass() {
        ///   println!("Grammar is valid and good for testing");
        /// } else {
        ///  println!("Grammar is not valid");
        /// }
        /// ```
        ///
        pub fn pass(&self) -> bool {
            self.errors.is_empty()
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ValidationError {
        pub kind: ValidationErrors,
        pub node_name: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ValidationErrors {
        NodeNotFound(String),
        EnumeratorNotFound(String),
        VariableNotFound(String),
        GlobalNotFound(String),
        CantUseVariable(String),
        EmptyToken,
        TokenNotFound(String),
        DuplicateLabel(String),
        LabelNotFound(String),
        TokenCollision(String),
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ValidationWarning {
        pub kind: ValidationWarnings,
        pub node_name: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ValidationWarnings {
        UnusedVariable(String),
        UsedDebug,
        UsedPrint,
        UsedDepricated(Depricated),
        UnusualToken(String, TokenErrors),
        UnusedLabel(String),
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum TokenErrors {
        NotAscii,
        ContainsWhitespace,
        TooLong,
        StartsNumeric,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Depricated {
        /// The node is depricated
        ///
        /// It is advised to use Goto instead
        Back,
        /// Maybe you should use a different approach
        Any,
    }

    /// This is a structure that keeps track of things that are hard to find
    pub struct LostAndFound {
        pub lost_labels: Vec<String>,
        pub found_labels: Vec<String>,
    }

    impl LostAndFound {
        pub fn new() -> Self {
            Self {
                lost_labels: Vec::new(),
                found_labels: Vec::new(),
            }
        }

        pub fn pass(&self, result: &mut ValidationResult, node_name: &str) {
            for looking_for in &self.lost_labels {
                if !self.found_labels.contains(looking_for) {
                    result.errors.push(ValidationError {
                        kind: ValidationErrors::LabelNotFound(looking_for.clone()),
                        node_name: node_name.to_string(),
                    });
                }
            }
            for found in &self.found_labels {
                if !self.lost_labels.contains(found) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarnings::UnusedLabel(found.clone()),
                        node_name: node_name.to_string(),
                    });
                }
            }
        }
    }
}
