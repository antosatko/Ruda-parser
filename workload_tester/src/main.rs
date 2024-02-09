//! Test the parser with a large file
//! 

use std::collections::BTreeMap;

use rparse::{grammar::*, lexer::TokenKinds, *};
/// Fields are ordered according to the order of the lines in the meta file
struct Meta {
    lines: usize,
    line_length: usize
}

fn read_dotmeta() -> Meta {
    use std::fs;
    let meta = fs::read_to_string("../workload.meta").unwrap();
    let mut lns = meta.lines();
    let lines = lns.next().unwrap().parse().unwrap();
    let line_length = lns.next().unwrap().parse().unwrap();
    Meta {
        lines,
        line_length,
    }
}

fn main() {
    let mut time = std::time::Instant::now();
    let meta = read_dotmeta();
    println!("meta read time: {:?}", time.elapsed());
    time = std::time::Instant::now();
    let mut parser = Parser::new();
    // let txt = include_str!("../workload.txt"); // The size of the file is 100MB which would make it impractical to include it in the tests
    use std::fs;
    let txt = fs::read_to_string("../workload.txt").unwrap();
    parser.lexer.add_token("\"".to_string());
    println!("lexer generated: {:?}", time.elapsed());

    let lex_start = std::time::Instant::now();
    let tokens = parser.lexer.lex_ascii(&txt);
    println!("lex time: {:?}", lex_start.elapsed());


    // gramamr generation
    let time = std::time::Instant::now();
    let variables = BTreeMap::new();
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

    let mut variables = BTreeMap::new();
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
    println!("grammar generation time: {:?}", time.elapsed());

    let parse_start = std::time::Instant::now();
    let result = parser.parse(&tokens, &txt).unwrap();
    let strings = result.entry.get_list("strings");
    println!("parse time: {:?}", parse_start.elapsed());
    // verify the result
    assert_eq!(strings.len(), meta.lines);
    for s in strings {
        assert_eq!(result.stringify_node(s, &txt).len(), meta.line_length);
    }
}