#[cfg(all(feature = "no_std", feature = "serde"))]
compile_error!("feature `no_std` and `serde` are mutually exclusive");

pub mod api;
pub mod grammar;
pub mod lexer;
pub mod parser;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parser {
    pub lexer: lexer::Lexer,
    pub grammar: grammar::Grammar,
    pub parser: parser::Parser,
}

impl Parser {
    pub fn new() -> Parser {
        let lexer = lexer::Lexer::new();
        let grammar = grammar::Grammar::new();
        Parser {
            lexer,
            grammar,
            parser: parser::Parser::new(),
        }
    }

    pub fn parse(
        &self,
        tokens: &Vec<lexer::Token>,
        text: &str,
    ) -> Result<parser::ParseResult, parser::ParseError> {
        self.parser.parse(&self.grammar, &self.lexer, text, tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Write, vec};


    use crate::lexer::TokenKinds;

    use self::grammar::{Parameters, VariableKind};

    use super::*;

    #[test]
    fn arithmetic_tokens() {
        let mut parser = Parser::new();
        let txt = "Function 1 +\n 2 * 3 - 4 /= 5";
        // Tokens that will be recognized by the lexer
        //
        // White space is ignored by default
        //
        // Everything else is a text token
        parser.lexer.add_tokens(&[
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/=".to_string(),
            "Function".to_string(),
        ]);

        // Parse the text
        let tokens = parser.lexer.lex_utf8(txt).unwrap();

        assert_eq!(tokens.len(), 21);
    }

    #[test]
    fn stringify() {
        let mut parser = Parser::new();
        let txt = "Functiond\t 1 +\n 2 * 3 - 4 /= 5";
        // Tokens that will be recognized by the lexer
        //
        // White space is ignored by default
        //
        // Everything else is a text token
        parser.lexer.add_tokens(&[
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/=".to_string(),
            "Function".to_string(),
        ]);

        // Parse the text
        let tokens = parser.lexer.lex_utf8(txt).unwrap();

        assert_eq!(parser.lexer.stringify_slice(&tokens, txt), txt);
        assert_eq!(parser.lexer.stringify_slice(&tokens[0..1], txt), "Function");
        assert_eq!(parser.lexer.stringify_slice(&tokens[1..5], txt), "d\t 1");
    }

    #[test]
    fn unfinished_token() {
        let mut parser = Parser::new();
        let txt = "fun";
        parser.lexer.add_token("function".to_string());
        let tokens = parser.lexer.lex_utf8(txt).unwrap();
        assert_eq!(tokens[0].kind, TokenKinds::Text);
    }

    #[test]
    fn rules() {
        let mut parser = Parser::new();
        let txt = "let   danda=  1+60;";
        parser.lexer.add_token("=".to_string());
        parser.lexer.add_token(":".to_string());
        parser.lexer.add_token("+".to_string());
        parser.lexer.add_token(";".to_string());
        parser.lexer.add_token("-".to_string());
        parser.lexer.add_token("*".to_string());
        parser.lexer.add_token("/".to_string());

        let tokens = parser.lexer.lex_utf8(txt).unwrap();

        let mut variables = HashMap::new();
        variables.insert("ident".to_string(), VariableKind::Node);
        variables.insert("type".to_string(), VariableKind::Node);
        variables.insert("value".to_string(), VariableKind::Node);

        parser.grammar.enumerators.insert(
            "operators".to_string(),
            grammar::Enumerator {
                name: "operators".to_string(),
                values: vec![
                    grammar::MatchToken::Token(TokenKinds::Token("+".to_string())),
                    grammar::MatchToken::Token(TokenKinds::Token("-".to_string())),
                    grammar::MatchToken::Token(TokenKinds::Token("*".to_string())),
                    grammar::MatchToken::Token(TokenKinds::Token("/".to_string())),
                ],
            },
        );

        parser.grammar.add_node(grammar::Node {
            name: "KWLet".to_string(),
            rules: vec![
                // detect the keyword
                grammar::Rule::Is {
                    token: grammar::MatchToken::Word("let".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::HardError(true)],
                },
                // detect the ident
                grammar::Rule::Is {
                    token: grammar::MatchToken::Token(TokenKinds::Text),
                    rules: vec![],
                    parameters: vec![Parameters::Set("ident".to_string())],
                },
                // detect the type if it exists
                grammar::Rule::Maybe {
                    token: grammar::MatchToken::Token(TokenKinds::Token(":".to_string())),
                    is: vec![grammar::Rule::Is {
                        token: grammar::MatchToken::Token(TokenKinds::Text),
                        rules: vec![],
                        parameters: vec![Parameters::Set("type".to_string())],
                    }],
                    isnt: vec![],
                    parameters: vec![],
                },
                // detect the value if it exists
                grammar::Rule::Maybe {
                    token: grammar::MatchToken::Token(TokenKinds::Token("=".to_string())),
                    is: vec![grammar::Rule::Is {
                        token: grammar::MatchToken::Node("value".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::Set("value".to_string())],
                    }],
                    isnt: vec![],
                    parameters: vec![],
                },
                // consume the semicolon (optional)
                grammar::Rule::Maybe {
                    token: grammar::MatchToken::Token(TokenKinds::Token(";".to_string())),
                    is: vec![],
                    isnt: vec![],
                    parameters: vec![],
                },
            ],
            variables,
        });
        let mut variables = HashMap::new();
        variables.insert("nodes".to_string(), VariableKind::NodeList);
        parser.grammar.add_node(grammar::Node {
            name: "value".to_string(),
            rules: vec![
                // detect the value[0]
                grammar::Rule::Is {
                    token: grammar::MatchToken::Token(TokenKinds::Text),
                    rules: vec![],
                    parameters: vec![Parameters::Set("nodes".to_string())],
                },
                // detect the operator
                grammar::Rule::While {
                    token: grammar::MatchToken::Enumerator("operators".to_string()),
                    // detect the value[n]
                    rules: vec![grammar::Rule::Is {
                        token: grammar::MatchToken::Token(TokenKinds::Text),
                        rules: vec![],
                        parameters: vec![Parameters::Set("nodes".to_string())],
                    }],
                    parameters: vec![Parameters::Set("nodes".to_string())],
                },
            ],
            variables,
        });
        parser.parser.entry = String::from("KWLet");

        let dump = serde_json::to_string(&parser);

        let mut file = std::fs::File::create("KWLet.json").unwrap();
        match dump {
            Ok(ref dump) => {
                file.write_all(dump.as_bytes()).unwrap();
            }
            Err(err) => panic!("Failed to dump grammar: {}", err),
        }

        parser.parse(&tokens, txt).unwrap();
    }

    #[test]
    fn string() {
        let txt = r#"


"úťf-8 štring"
"second string"
"#;

        let mut parser = Parser::new();
        parser.lexer.add_token("\"".to_string());

        // add random tokens to test the lexer
        parser.lexer.add_token("=".to_string());
        parser.lexer.add_token(";".to_string());
        parser.lexer.add_token(":".to_string());
        parser.lexer.add_token("+".to_string());
        parser.lexer.add_token("-".to_string());
        parser.lexer.add_token("*".to_string());
        parser.lexer.add_token("/".to_string());
        parser.lexer.add_token("let".to_string());
        parser.lexer.add_token("function".to_string());
        parser.lexer.add_token("danda".to_string());
        parser.lexer.add_token("1".to_string());
        parser.lexer.add_token("60".to_string());
        parser.lexer.add_token("string".to_string());
        parser.lexer.add_token(" ".to_string());

        let tokens = parser.lexer.lex_utf8(txt).unwrap();

        let mut variables = HashMap::new();
        variables.insert("start".to_string(), VariableKind::Node);
        variables.insert("end".to_string(), VariableKind::Node);
        parser.grammar.add_node(grammar::Node {
            name: "string".to_string(),
            rules: vec![
                // detect the start
                grammar::Rule::Is {
                    token: grammar::MatchToken::Token(TokenKinds::Token("\"".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::Set("start".to_string()), Parameters::NodeStart],
                },
                grammar::Rule::Until {
                    token: grammar::MatchToken::Token(TokenKinds::Token("\"".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::Set("end".to_string()), Parameters::NodeEnd],
                },
            ],
            variables,
        });

        let mut variables = HashMap::new();
        variables.insert("strings".to_string(), VariableKind::NodeList);
        variables.insert("count".to_string(), VariableKind::Number);
        variables.insert("zero".to_string(), VariableKind::Number);

        parser.grammar.add_node(grammar::Node {
            name: "entry".to_string(),
            rules: vec![
                grammar::Rule::While {
                    token: grammar::MatchToken::Node("string".to_string()),
                    rules: vec![],
                    parameters: vec![
                        Parameters::Set("strings".to_string()),
                        Parameters::Increment("count".to_string()),
                    ],
                },
                grammar::Rule::Command {
                    command: grammar::Commands::Compare {
                        left: "count".to_string(),
                        right: "zero".to_string(), // zero is not defined, so it will be 0
                        comparison: grammar::Comparison::Equal,
                        rules: vec![grammar::Rule::Command {
                            command: grammar::Commands::Error {
                                message: "No strings found".to_string(),
                            },
                        }],
                    },
                },
            ],
            variables,
        });

        let result = parser.parse(&tokens, txt).unwrap();
        let strings = result.entry.get_list("strings");
        assert_eq!(strings.len(), 2);

        // first string
        assert_eq!(result.stringify_node(&strings[0], txt), r#""úťf-8 štring""#);

        // second string
        assert_eq!(
            result.stringify_node(&strings[1], txt),
            r#""second string""#
        );
    }

    #[test]
    fn vec_char_eq() {
        let a = vec!['a', 'b', 'c'];
        let b = vec!['a', 'b', 'c'];
        let c = vec!['a', 'b', 'd'];
        assert_eq!(a, b);
        assert_eq!(true, a == b);
        assert_eq!(false, a == c);

        let slice_a = &a[0..2];
        let slice_b = &b[0..2];
        let slice_c = &c[1..3];
        assert_eq!(slice_a, slice_b);
        assert_eq!(true, slice_a == slice_b);
        assert_eq!(false, slice_a == slice_c);
    }

    /// Fields are ordered according to the order of the lines in the meta file
    struct Meta {
        lines: usize,
        line_length: usize,
    }

    fn read_dotmeta() -> Meta {
        use std::fs;
        let meta = fs::read_to_string("workload.meta").unwrap();
        let mut lns = meta.lines();
        let lines = lns.next().unwrap().parse().unwrap();
        let line_length = lns.next().unwrap().parse().unwrap();
        Meta { lines, line_length }
    }

    #[test]
    fn workload_file() {
        let meta = read_dotmeta();
        let mut parser = Parser::new();
        // let txt = include_str!("../workload.txt"); // The size of the file is 100MB which would make it impractical to include it in the tests
        use std::fs;
        let txt = fs::read_to_string("workload.txt").unwrap();
        parser.lexer.add_token("\"".to_string());

        let lex_start = std::time::Instant::now();
        let tokens = parser.lexer.lex_utf8(&txt).unwrap();
        println!("lex time: {:?}", lex_start.elapsed());

        let variables = HashMap::new();
        parser.grammar.add_node(grammar::Node {
            name: "string".to_string(),
            rules: vec![
                // detect the start
                grammar::Rule::Is {
                    token: grammar::MatchToken::Token(TokenKinds::Token("\"".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::NodeStart, Parameters::HardError(true)],
                },
                grammar::Rule::Until {
                    token: grammar::MatchToken::Token(TokenKinds::Token("\"".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::NodeEnd],
                },
            ],
            variables,
        });

        let mut variables = HashMap::new();
        variables.insert("strings".to_string(), VariableKind::NodeList);
        variables.insert("count".to_string(), VariableKind::Number);
        variables.insert("zero".to_string(), VariableKind::Number);

        parser.grammar.add_node(grammar::Node {
            name: "entry".to_string(),
            rules: vec![
                grammar::Rule::While {
                    token: grammar::MatchToken::Node("string".to_string()),
                    rules: vec![],
                    parameters: vec![
                        Parameters::Set("strings".to_string()),
                        Parameters::Increment("count".to_string()),
                    ],
                },
                grammar::Rule::Command {
                    command: grammar::Commands::Compare {
                        left: "count".to_string(),
                        right: "zero".to_string(), // zero is not defined, so it will be 0
                        comparison: grammar::Comparison::Equal,
                        rules: vec![grammar::Rule::Command {
                            command: grammar::Commands::Error {
                                message: "No strings found".to_string(),
                            },
                        }],
                    },
                },
            ],
            variables,
        });

        let parse_start = std::time::Instant::now();
        let result = parser.parse(&tokens, &txt).unwrap();
        let strings = result.entry.get_list("strings");
        println!("strings: {}", strings.len());
        println!("parse time: {:?}", parse_start.elapsed());
        // verify the result
        assert_eq!(strings.len(), meta.lines);
        for s in strings {
            assert_eq!(result.stringify_node(s, &txt).len(), meta.line_length);
        }
    }

    #[test]
    fn load_json() {
        use std::io::Read;

        let mut file = std::fs::File::open("KWLet.json").unwrap();
        let mut parser = String::new();
        file.read_to_string(&mut parser).unwrap();

        let parser: Parser = serde_json::from_str(&parser).unwrap();

        let txt = "let a: int = 500 * 9;";

        let tokens = parser.lexer.lex_utf8(txt).unwrap();

        println!("{:#?}", tokens);

        let result = parser.parse(&tokens, txt).unwrap();

        assert_eq!(
            result.stringify_node(result.entry.try_get_node("value").as_ref().unwrap(), txt),
            " 500 * 9"
        );
    }
}
