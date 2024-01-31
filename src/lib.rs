pub mod lexer;
pub mod grammar;
pub mod preprocesor;
pub mod parser;

pub struct Parser<'a> {
    text: &'a str,
    pub lexer: lexer::Lexer<'a>,
    pub grammar: grammar::Grammar<'a>,
    pub parser: parser::Parser<'a>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Parser<'static> {
        let text = "";
        let lexer = lexer::Lexer::new(text);
        let grammar = grammar::Grammar::new(text);
        Parser {
            text,
            lexer,
            grammar,
            parser: parser::Parser::new(text),
        }
    }

    pub fn set_text(&mut self, text: &'a str) {
        self.text = text;
        self.lexer.text = text;
        self.grammar.text = text;
    }

    pub fn parse(&mut self) -> Result<parser::ParseResult, parser::ParseError> {
        self.parser.parse(&self.grammar, &self.lexer)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, vec};

    use crate::lexer::TokenKinds;

    use self::grammar::{Parameters, VariableKind};

    use super::*;

    #[test]
    fn arithmetic_tokens() {
        let mut parser = Parser::new();
        parser.set_text("Function 1 +\n 2 * 3 - 4 /= 5");
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
        let tokens = parser.lexer.lex();

        assert_eq!(tokens.len(), 21);
    }

    #[test]
    fn stringify() {
        let mut parser = Parser::new();
        let txt = "Functiond\t 1 +\n 2 * 3 - 4 /= 5";
        parser.set_text(txt);
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
        let tokens = parser.lexer.lex();

        assert_eq!(parser.lexer.stringify_slice(&tokens), txt);
        assert_eq!(parser.lexer.stringify_slice(&tokens[0..1]), "Function");
        assert_eq!(parser.lexer.stringify_slice(&tokens[1..5]), "d\t 1");

        // invalidate the result by changing the text
        parser.set_text("bad text");

        assert_ne!(parser.lexer.stringify_slice(&tokens), txt);
    }

    #[test]
    fn unfinished_token() {
        let mut parser = Parser::new();
        parser.set_text("fun");
        parser.lexer.add_token("function".to_string());
        let tokens = parser.lexer.lex();
        assert_eq!(tokens[0].kind, TokenKinds::Text);
    }

    #[test]
    fn rules() {
        let mut parser = Parser::new();
        parser.set_text("let a=1+60;");
        parser.lexer.add_token("=".to_string());
        parser.lexer.add_token(";".to_string());
        parser.lexer.add_token(":".to_string());
        parser.lexer.add_token("+".to_string());
        parser.lexer.add_token("-".to_string());
        parser.lexer.add_token("*".to_string());
        parser.lexer.add_token("/".to_string());

        let tokens = parser.lexer.lex();

        assert_eq!(tokens.len(), 9);


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

        parser.grammar.nodes.insert(
            "KWLet".to_string(),
            grammar::Node {
                name: "KWLet".to_string(),
                rules: vec![
                    // detect the keyword
                    grammar::Rule::Is {
                        token: grammar::MatchToken::Word("Let".to_string()),
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
            },
        );
        let mut variables = HashMap::new();
        variables.insert("nodes".to_string(), VariableKind::NodeList);
        parser.grammar.nodes.insert(
            "value".to_string(),
            grammar::Node {
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
                        token: grammar::MatchToken::Enumerator("operator".to_string()),
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
            },
        );

        parser.parser.entry = String::from("KWLet");
        let result = parser.parse().unwrap();
    }
}
