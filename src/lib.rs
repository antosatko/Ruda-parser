pub mod lexer;
pub mod parser;
pub mod preprocesor;

pub struct Parser {
    pub tokens: Vec<lexer::Token>,
}

impl Parser {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for token in &self.tokens {
            result.push_str(&token.to_string());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_stringify() {
        let mut parser = Parser::new();
        use lexer::Token::*;
        use lexer::ControlToken::*;
        parser.tokens = vec![
            Char('a'),
            Char('b'),
            Char('c'),
            String("def".to_string()),
            Whitespace(' '),
            Control(Eol),
        ];
        assert_eq!(parser.to_string(), "abcdef \n");
    }
}
