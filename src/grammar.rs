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
    use crate::lexer::{self, *};

    impl Grammar {
        /// Validates the grammar
        pub fn validate(&self, lexer: &Lexer) -> ValidationResult {
            let mut result = ValidationResult::new();

            for node in self.nodes.values() {
                self.validate_node(node, lexer, &mut result);
            }

            result
        }

        pub fn validate_node(&self, node: &Node, lexer: &Lexer, result: &mut ValidationResult) {
            for rule in &node.rules {
                self.validate_rule(rule, node, lexer, result);
            }
        }

        pub fn validate_rule(
            &self,
            rule: &Rule,
            node: &Node,
            lexer: &Lexer,
            result: &mut ValidationResult,
        ) {
            match rule {
                Rule::Is {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, result);
                    self.validate_parameters(parameters, node, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::Isnt {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, result);
                    self.validate_parameters(parameters, node, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::IsOneOf { tokens } => {
                    for one_of in tokens {
                        self.validate_token(&one_of.token, node, lexer, result);
                        self.validate_parameters(&one_of.parameters, node, result);
                        for rule in &one_of.rules {
                            self.validate_rule(rule, node, lexer, result);
                        }
                    }
                }
                Rule::Maybe {
                    token,
                    is,
                    isnt,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, result);
                    self.validate_parameters(parameters, node, result);
                    for rule in is {
                        self.validate_rule(rule, node, lexer, result);
                    }
                    for rule in isnt {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::MaybeOneOf { is_one_of, isnt } => {
                    for (token, rules, parameters) in is_one_of {
                        self.validate_token(token, node, lexer, result);
                        self.validate_parameters(parameters, node, result);
                        for rule in rules {
                            self.validate_rule(rule, node, lexer, result);
                        }
                    }
                    for rule in isnt {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::While {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, result);
                    self.validate_parameters(parameters, node, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::Loop { rules } => {
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::Until {
                    token,
                    rules,
                    parameters,
                } => {
                    self.validate_token(token, node, lexer, result);
                    self.validate_parameters(parameters, node, result);
                    for rule in rules {
                        self.validate_rule(rule, node, lexer, result);
                    }
                }
                Rule::UntilOneOf { tokens } => {
                    for one_of in tokens {
                        self.validate_token(&one_of.token, node, lexer, result);
                        self.validate_parameters(&one_of.parameters, node, result);
                        for rule in &one_of.rules {
                            self.validate_rule(rule, node, lexer, result);
                        }
                    }
                }
                Rule::Command { command } => match command {
                    Commands::Compare {
                        left,
                        right,
                        comparison,
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
                            self.validate_rule(rule, node, lexer, result);
                        }
                    }
                    Commands::Error { message } => (),
                    Commands::HardError { set } => (),
                    Commands::Goto { label } => (),
                    Commands::Label { name } => (),
                    Commands::Print { message } => (),
                },
            }
        }

        pub fn validate_token(
            &self,
            token: &MatchToken,
            node: &Node,
            lexer: &Lexer,
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

                        // check if it starts with a number
                        let first = txt.chars().next().unwrap();
                        if first.is_numeric() {
                            result.warnings.push(ValidationWarning {
                                kind: ValidationWarnings::UnusualToken(txt.clone()),
                                node_name: node.name.clone(),
                            });
                        }

                        // check if it contains a whitespace
                        if txt.chars().any(|c| c.is_whitespace()) {
                            result.warnings.push(ValidationWarning {
                                kind: ValidationWarnings::UnusualToken(txt.clone()),
                                node_name: node.name.clone(),
                            });
                        }

                        // check if it is longer than 2 characters
                        if txt.len() > 2 {
                            result.warnings.push(ValidationWarning {
                                kind: ValidationWarnings::UnusualToken(txt.clone()),
                                node_name: node.name.clone(),
                            });
                        }

                        // check if it is not ascii
                        if !txt.chars().all(|c| c.is_ascii()) {
                            result.warnings.push(ValidationWarning {
                                kind: ValidationWarnings::UnusualToken(txt.clone()),
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
                        // check if label exists anywhere in the node (this is tedious so maybe next time)
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

        pub fn success(&self) -> bool {
            self.errors.is_empty() && self.warnings.is_empty()
        }

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
        UnusualToken(String),
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
}
