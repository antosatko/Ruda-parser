
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Token {
    Char(char),
    String(String),
    Text(String),
    Whitespace(char),
    Control(ControlToken),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ControlToken {
    Eof,
    Eol,
}

impl Token {
    pub fn to_string(&self) -> String {
        match self {
            Self::Char(c) => c.to_string(),
            Self::String(s) => s.clone(),
            Self::Text(s) => s.clone(),
            Self::Whitespace(c) => c.to_string(),
            Self::Control(control_token) => match control_token {
                ControlToken::Eol => "\n".to_string(),
                ControlToken::Eof => "".to_string(),
            },
        }
    }
}