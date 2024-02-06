use std::collections::HashMap;

use rparse::{grammar::*, lexer::TokenKinds, *};
/// Fields are ordered according to the order of the lines in the meta file
struct Meta {
    lines: usize,
    line_length: usize,
    size: usize,
    name: String,
}

fn read_dotmeta() -> Meta {
    use std::fs;
    let meta = fs::read_to_string("../workload.meta").unwrap();
    let mut lns = meta.lines();
    let lines = lns.next().unwrap().parse().unwrap();
    let line_length = lns.next().unwrap().parse().unwrap();
    let size = lns.next().unwrap().parse().unwrap();
    let name = lns.next().unwrap().to_string();
    Meta {
        lines,
        line_length,
        size,
        name,
    }
}

fn main() {
    let meta = read_dotmeta();
    let mut parser = Parser::new();
    // let txt = include_str!("../workload.txt"); // The size of the file is 100MB which would make it impractical to include it in the tests
    use std::fs;
    let txt = fs::read_to_string("../workload.txt").unwrap();
    parser.set_text(&txt);
    parser.lexer.add_token("\"".to_string());

    let lex_start = std::time::Instant::now();
    let tokens = parser.lexer.lex();
    parser.lexer.tokens = tokens;
    println!("lex time: {:?}", lex_start.elapsed());

    let variables = HashMap::new();
    parser.grammar.add_node(
        grammar::Node {
            name: "string".to_string(),
            rules: vec![
                // detect the start
                grammar::Rule::Is {
                    token: grammar::MatchToken::Token(TokenKinds::Token("\"".to_string())),
                    rules: vec![],
                    parameters: vec![
                        Parameters::NodeStart,
                        Parameters::HardError(true),
                    ],
                },
                grammar::Rule::Until {
                    token: grammar::MatchToken::Token(TokenKinds::Token("\"".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::NodeEnd],
                },
            ],
            variables,
        },
    );

    let mut variables = HashMap::new();
    variables.insert("strings".to_string(), VariableKind::NodeList);
    variables.insert("count".to_string(), VariableKind::Number);
    variables.insert("zero".to_string(), VariableKind::Number);

    parser.grammar.add_node(
        grammar::Node {
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
        },
    );

    let parse_start = std::time::Instant::now();
    let result = parser.parse().unwrap();
    let strings = result.entry.get_list("strings");
    println!("strings: {}", strings.len());
    println!("parse time: {:?}", parse_start.elapsed());
    // verify the result
    assert_eq!(strings.len(), meta.lines);
    for s in strings {
        assert_eq!(result.stringify_node(s).len(), meta.line_length);
    }
}
