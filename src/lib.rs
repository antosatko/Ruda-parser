pub mod lexer;
pub mod parser;
pub mod preprocesor;

pub struct Parser<'a> {
    text: &'a str,
    pub lexer: lexer::Lexer<'a>,
    pub parser: parser::Parser<'a>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Parser<'static> {
        let text = "";
        Parser {
            text,
            lexer: lexer::Lexer::new(text),
            parser: parser::Parser::new(text),
        }
    }

    pub fn set_text(&mut self, text: &'a str) {
        self.text = text;
        self.lexer.text = text;
        self.parser.text = text;
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, hash::Hash, vec};

    use crate::lexer::TokenKinds;

    use self::parser::{Parameters, Rule, VariableKind};

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
        parser.set_text("fun");
        parser.lexer.add_token("=".to_string());
        parser.lexer.add_token(";".to_string());
        parser.lexer.add_token(":".to_string());

        let mut variables = HashMap::new();
        variables.insert("ident".to_string(), VariableKind::Node);
        variables.insert("type".to_string(), VariableKind::Node);
        variables.insert("value".to_string(), VariableKind::Node);

        parser.parser.nodes.insert(
            "KWLet".to_string(),
            parser::Node {
                name: "KWLet".to_string(),
                rules: vec![
                    parser::Rule::Is {
                        token: parser::MatchToken::Word("Let".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::HardError(true)],
                    },
                    parser::Rule::Is {
                        token: parser::MatchToken::Token(TokenKinds::Text),
                        rules: vec![],
                        parameters: vec![Parameters::Set("ident".to_string())],
                    },
                    parser::Rule::Maybe {
                        token: parser::MatchToken::Token(TokenKinds::Token(":".to_string())),
                        is: vec![parser::Rule::Is {
                            token: parser::MatchToken::Token(TokenKinds::Text),
                            rules: vec![],
                            parameters: vec![Parameters::Set("type".to_string())],
                        }],
                        isnt: vec![],
                        parameters: vec![],
                    },
                    parser::Rule::Maybe {
                        token: parser::MatchToken::Token(TokenKinds::Text),
                        is: vec![parser::Rule::Is {
                            token: parser::MatchToken::Token(TokenKinds::Text),
                            rules: vec![],
                            parameters: vec![Parameters::Set("value".to_string())],
                        }],
                        isnt: vec![],
                        parameters: vec![],
                    },
                ],
                variables,
            },
        );
    }
}
