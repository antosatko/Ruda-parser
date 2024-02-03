use std::collections::HashMap;

use crate::lexer::TokenKinds;

pub struct Grammar<'a> {
    pub(crate) text: &'a str,
    pub(crate) nodes: HashMap<String, Node>,
    pub(crate) enumerators: HashMap<String, Enumerator>,
    pub(crate) globals: HashMap<String, VariableKind>,
}

impl<'a> Grammar<'a> {
    pub fn new(text: &'a str) -> Grammar<'a> {
        Grammar {
            text,
            nodes: HashMap::new(),
            enumerators: HashMap::new(),
            globals: HashMap::new(),
        }
    }
}

/// A collection of rules
pub type Rules = Vec<Rule>;

/// A rule defines how a token will be matched and what will happen if it is matched
///
/// It also contains parameters that can be used if the rule is matched
///
/// Special kind of rules are commands that can be executed without matching a token
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
    IsOneOf {
        tokens: Vec<(MatchToken, Rules, Vec<Parameters>)>,
    },
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
    /// Performs a command
    ///
    /// The command will be executed without matching a token
    Command { command: Commands },
}

/// Commands that can be executed
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
}

/// Comparison operators
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
#[derive(Clone, Debug)]
pub enum MatchToken {
    /// A token kind
    Token(TokenKinds),
    /// A node name
    Node(String),
    /// A constant word
    Word(String),
    /// An enumerator
    Enumerator(String),
}

/// A node is a collection of rules that will be executed when the node is matched
pub struct Node {
    /// Name of the node
    pub name: String,
    /// Rules that will be executed when the node is matched
    pub rules: Rules,
    /// Variables that can be used in the node and will be accessible from the outside
    pub variables: HashMap<String, VariableKind>,
}

/// A variable that can be used in a node
pub enum VariableKind {
    /// Holds a single node
    Node,
    /// Holds a list of nodes
    NodeList,
    /// Holds a boolean
    Boolean,
    /// Holds a number
    Number,
    /// Can be used to count the number of times a rule is matched
    Count,
}

/// Parameters that can be used on a rule if it is matched
pub enum Parameters {
    /// Sets a variable to a value
    Set(String),
    /// Sets a global variable to a value
    Global(String),
    /// Adds 1 to a variable of type Count
    Count(String),
    /// Adds 1 to a global variable of type Count
    CountGlobal(String),
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
    /// If the node ends with an error, it will be a hard error
    /// resulting in the parent node to also end with an error
    ///
    /// This is a way of telling the parser that the current node MUST match
    ///
    /// This is useful for using nodes in optional rules
    HardError(bool),
}

pub struct Enumerator {
    pub name: String,
    pub values: Vec<MatchToken>,
}
