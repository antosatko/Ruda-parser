extern crate alloc;
use alloc::string::*;
use alloc::vec;

type Map = std::collections::HashMap<String, VariableKind>;

use rparse::grammar;
use rparse::{grammar::*, lexer::*, Parser};

pub fn gen_parser() -> Parser {
    let mut parser = Parser::new();

    let tokens = vec![
        "+=".to_string(),
        "-=".to_string(),
        "*=".to_string(),
        "/=".to_string(),
        "+".to_string(),
        "-".to_string(),
        "*".to_string(),
        "//".to_string(),
        "/".to_string(),
        "(".to_string(),
        ")".to_string(),
        "{".to_string(),
        "}".to_string(),
        "[".to_string(),
        "]".to_string(),
        "<=".to_string(),
        ">=".to_string(),
        "<".to_string(),
        ">".to_string(),
        "==".to_string(),
        "=".to_string(),
        "!=".to_string(),
        "!".to_string(),
        "&&".to_string(),
        "&".to_string(),
        "||".to_string(),
        "?".to_string(),
        ":".to_string(),
        ".".to_string(),
        ";".to_string(),
        ",".to_string(),
        "\"".to_string(),
        "'".to_string(),
        "#".to_string(),
    ];
    parser.lexer.add_tokens(&tokens);

    let preprocessor: Preprocessor = |text_, tokens| {
        let mut new_tokens = vec![];
        let mut i = 0;
        'main: while i < tokens.len() {
            let token = &tokens[i];
            match &token.kind {
                TokenKinds::Text => {
                    let text = &text_[token.index..token.index + token.len];

                    // test for number
                    // strip suffix character (u, i, f, etc.)
                    let c = text.chars().last().unwrap();
                    let text1 = if c.is_alphabetic() {
                        &text[..text.len() - text.chars().last().unwrap().len_utf8()]
                    } else {
                        text
                    };
                    match text1.parse::<u64>() {
                        Ok(_) => {
                            if tokens[i + 1].kind != TokenKinds::Token(".".to_string()) {
                                // it's an integer (but could be another type if it has a suffix)
                                new_tokens.push(Token {
                                    kind: TokenKinds::Complex(
                                        match c {
                                            'u' => "uint",
                                            'i' => "int",
                                            'f' => "float",
                                            'c' => "char",
                                            _ => "int",
                                        }
                                        .to_string(),
                                    ),
                                    index: token.index,
                                    len: token.len,
                                    location: token.location.clone(),
                                });
                                i += 1;
                                continue 'main;
                            }
                            // it's a float (suffix is not allowed)
                            match tokens[i + 2].kind {
                                TokenKinds::Text => {
                                    let token = &tokens[i + 2];
                                    let text = &text_[token.index..token.index + token.len];
                                    match text.parse::<u64>() {
                                        Ok(_) => {
                                            // it's a float with a decimal value
                                            new_tokens.push(Token {
                                                index: tokens[i].index,
                                                len: tokens[i].len
                                                    + tokens[i + 1].len
                                                    + tokens[i + 2].len,
                                                location: token.location.clone(),
                                                kind: TokenKinds::Complex("float".to_string()),
                                            });
                                            i += 3;
                                            continue 'main;
                                        }
                                        Err(_) => {
                                            // it's a float without a decimal value even though it has a decimal point (error)
                                            Err(PreprocessorError {
                                                message: "Expected a float".to_string(),
                                                location: token.location.clone(),
                                                len: token.len
                                                    + tokens[i + 1].len
                                                    + tokens[i + 2].len,
                                            })?
                                        }
                                    }
                                }
                                _ => {
                                    // it's a float without a decimal value
                                    new_tokens.push(Token {
                                        index: token.index,
                                        len: token.len + tokens[i + 1].len,
                                        location: token.location.clone(),
                                        kind: TokenKinds::Complex("float".to_string()),
                                    });
                                    i += 2;
                                    continue 'main;
                                }
                            }
                        }
                        Err(_) => (),
                    }
                    new_tokens.push(token.clone());
                }
                TokenKinds::Token(tok) => match tok.as_str() {
                    "\"" => {
                        let mut j = i + 1;
                        while j < tokens.len() {
                            let current = &tokens[j];
                            if current.kind == TokenKinds::Token("\"".to_string())
                                && tokens[j - 1].kind != TokenKinds::Token("\\".to_string())
                            {
                                new_tokens.push(Token {
                                    index: token.index,
                                    len: current.index - token.index + current.len,
                                    location: token.location.clone(),
                                    kind: TokenKinds::Complex("string".to_string()),
                                });
                                i = j + 1;
                                continue 'main;
                            }
                            j += 1;
                        }
                        let current = &tokens[j - 1];
                        Err(PreprocessorError {
                            message: "Expected a closing quote".to_string(),
                            location: token.location.clone(),
                            len: current.index - token.index + current.len,
                        })?;
                    }
                    "//" => {
                        i += 1;
                        // first check if it's a doc comment
                        if tokens[i].kind == TokenKinds::Token("/".to_string()) {
                            i += 1;
                            let start = i;
                            loop {
                                match &tokens[i].kind {
                                    TokenKinds::Control(_) => {
                                        i += 1;
                                        let doc_comment = Token {
                                            index: tokens[start].index,
                                            len: tokens[i - 1].index + tokens[i - 1].len
                                                - tokens[start].index,
                                            location: tokens[start].location.clone(),
                                            kind: TokenKinds::Complex("doc_comment".to_string()),
                                        };
                                        new_tokens.push(doc_comment);
                                        continue 'main;
                                    }
                                    _ => (),
                                }
                                i += 1;
                            }
                        }
                        // it's a normal comment
                        loop {
                            match &tokens[i].kind {
                                TokenKinds::Control(_) => {
                                    i += 1;
                                    continue 'main;
                                }
                                _ => (),
                            }
                            i += 1;
                        }
                    }
                    _ => {
                        new_tokens.push(token.clone());
                    }
                },
                TokenKinds::Whitespace => (),
                TokenKinds::Control(ControlTokenKind::Eol) => (),
                _ => {
                    new_tokens.push(token.clone());
                }
            }
            i += 1;
        }
        Ok(new_tokens)
    };
    parser.lexer.preprocessors.push(preprocessor);

    let operators = Enumerator {
        name: "operators".to_string(),
        values: vec![
            MatchToken::Token(TokenKinds::Token("+=".to_string())),
            MatchToken::Token(TokenKinds::Token("-=".to_string())),
            MatchToken::Token(TokenKinds::Token("*=".to_string())),
            MatchToken::Token(TokenKinds::Token("/=".to_string())),
            MatchToken::Token(TokenKinds::Token("+".to_string())),
            MatchToken::Token(TokenKinds::Token("-".to_string())),
            MatchToken::Token(TokenKinds::Token("*".to_string())),
            MatchToken::Token(TokenKinds::Token("/".to_string())),
            MatchToken::Token(TokenKinds::Token("<=".to_string())),
            MatchToken::Token(TokenKinds::Token(">=".to_string())),
            MatchToken::Token(TokenKinds::Token("<".to_string())),
            MatchToken::Token(TokenKinds::Token(">".to_string())),
            MatchToken::Token(TokenKinds::Token("==".to_string())),
            MatchToken::Token(TokenKinds::Token("=".to_string())),
            MatchToken::Token(TokenKinds::Token("!=".to_string())),
            MatchToken::Token(TokenKinds::Token("&&".to_string())),
            MatchToken::Token(TokenKinds::Token("||".to_string())),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(operators.name.clone(), operators);

    let keywords = Enumerator {
        name: "keywords".to_string(),
        values: vec![
            // done:
            MatchToken::Word("if".to_string()),
            MatchToken::Word("else".to_string()),
            MatchToken::Word("while".to_string()),
            MatchToken::Word("use".to_string()),
            MatchToken::Word("for".to_string()),
            MatchToken::Word("return".to_string()),
            MatchToken::Word("break".to_string()),
            MatchToken::Word("continue".to_string()),
            MatchToken::Word("fun".to_string()),
            MatchToken::Word("let".to_string()),
            MatchToken::Word("enum".to_string()),
            MatchToken::Word("class".to_string()),
            // todo:
            MatchToken::Word("impl".to_string()),
            MatchToken::Word("const".to_string()),
            MatchToken::Word("trait".to_string()),
            MatchToken::Word("type".to_string()),
            MatchToken::Word("as".to_string()),
            MatchToken::Word("delete".to_string()),
            MatchToken::Word("switch".to_string()),
            MatchToken::Word("new".to_string()),
            // in consideration:
            MatchToken::Word("error".to_string()),
            MatchToken::Word("try".to_string()),
            MatchToken::Word("yeet".to_string()),
            MatchToken::Word("catch".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(keywords.name.clone(), keywords);

    let unary_operators = Enumerator {
        name: "unary_operators".to_string(),
        values: vec![
            MatchToken::Token(TokenKinds::Token("!".to_string())),
            MatchToken::Token(TokenKinds::Token("-".to_string())),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(unary_operators.name.clone(), unary_operators);

    let numbers = Enumerator {
        name: "numbers".to_string(),
        values: vec![
            MatchToken::Token(TokenKinds::Complex("int".to_string())),
            MatchToken::Token(TokenKinds::Complex("float".to_string())),
            MatchToken::Token(TokenKinds::Complex("uint".to_string())),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(numbers.name.clone(), numbers);

    let literals = Enumerator {
        name: "literals".to_string(),
        values: vec![
            MatchToken::Token(TokenKinds::Complex("string".to_string())),
            MatchToken::Token(TokenKinds::Complex("char".to_string())),
            MatchToken::Enumerator("numbers".to_string()),
            MatchToken::Node("array".to_string()),
            MatchToken::Node("tuple".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(literals.name.clone(), literals);

    let mut variables = Map::new();
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    let array = Node {
        name: "array".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("[".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("array_types".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("]".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(array.name.clone(), array);

    // this is work in progress
    //
    // this will hold many different ways to initialize an array
    let array_types = Enumerator {
        name: "array_types".to_string(),
        values: vec![
            MatchToken::Node("array_builder".to_string()),
            MatchToken::Node("values_list".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(array_types.name.clone(), array_types);

    let mut variables = Map::new();
    variables.insert("value".to_string(), grammar::VariableKind::Node);
    variables.insert("times".to_string(), grammar::VariableKind::Node);
    let array_builder = Node {
        name: "array_builder".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("value".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("times".to_string())],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(array_builder.name.clone(), array_builder);

    let mut variables = Map::new();
    variables.insert("list".to_string(), grammar::VariableKind::NodeList);
    let entry = Node {
        name: "entry".to_string(),
        rules: vec![Rule::While {
            token: MatchToken::Enumerator("entry_nodes".to_string()),
            rules: vec![],
            parameters: vec![Parameters::Set("list".to_string())],
        }],
        variables,
    };
    parser.grammar.nodes.insert(entry.name.clone(), entry);

    let entry_nodes = Enumerator {
        name: "entry_nodes".to_string(),
        values: vec![
            MatchToken::Node("KWImport".to_string()),
            MatchToken::Node("KWFunction".to_string()),
            MatchToken::Node("KWClass".to_string()),
            MatchToken::Node("KWUse".to_string()),
            MatchToken::Node("KWEnum".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(entry_nodes.name.clone(), entry_nodes);

    let mut variables = Map::new();
    variables.insert("file".to_string(), grammar::VariableKind::Node);
    variables.insert("alias".to_string(), grammar::VariableKind::Node);
    let import = Node {
        name: "KWImport".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("import".to_string()),
                rules: vec![Rule::Is {
                    token: MatchToken::Token(TokenKinds::Complex("string".to_string())),
                    rules: vec![],
                    parameters: vec![
                        Parameters::Set("file".to_string()),
                        Parameters::Global("imports".to_string()),
                    ],
                }],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Word("as".to_string()),
                is: vec![Rule::Is {
                    token: MatchToken::Token(TokenKinds::Text),
                    rules: vec![],
                    parameters: vec![Parameters::Set("alias".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(import.name.clone(), import);

    let mut variables = Map::new();
    variables.insert("root".to_string(), grammar::VariableKind::Node);
    variables.insert("path".to_string(), grammar::VariableKind::Node);
    let kw_use = Node {
        name: "KWUse".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("use".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            // first needs to be a text
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("root".to_string())],
            },
            // then it can be a path
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Node("use_path".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("path".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_use.name.clone(), kw_use);

    let mut variables = Map::new();
    variables.insert("path".to_string(), grammar::VariableKind::NodeList);
    let use_path = Node {
        name: "use_path".to_string(),
        rules: vec![
            Rule::Loop {
                rules: vec![
                    Rule::IsOneOf {
                        tokens: vec![
                            OneOf {
                                token: MatchToken::Token(TokenKinds::Text),
                                rules: vec![],
                                parameters: vec![Parameters::Set("path".to_string())],
                            },
                            OneOf {
                                token: MatchToken::Token(TokenKinds::Token("*".to_string())),
                                rules: vec![],
                                parameters: vec![Parameters::Set("path".to_string())],
                            },
                            OneOf {
                                token: MatchToken::Node("use_multiple_paths".to_string()),
                                rules: vec![Rule::Command {
                                    command: Commands::Goto {
                                        label: "end_path".to_string(),
                                    },
                                }],
                                parameters: vec![Parameters::Set("path".to_string())],
                            },
                        ],
                    },
                    Rule::Maybe {
                        token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                        is: vec![],
                        isnt: vec![Rule::Command {
                            command: Commands::Goto {
                                label: "end_path".to_string(),
                            },
                        }],
                        parameters: vec![],
                    },
                ],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end_path".to_string(),
                },
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(use_path.name.clone(), use_path);

    let mut variables = Map::new();
    variables.insert("paths".to_string(), grammar::VariableKind::NodeList);
    let use_multiple_paths = Node {
        name: "use_multiple_paths".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("{".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Node("use_path".to_string()),
                is: vec![Rule::While {
                    token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                    rules: vec![Rule::Maybe {
                        token: MatchToken::Node("use_path".to_string()),
                        is: vec![],
                        isnt: vec![],
                        parameters: vec![Parameters::Set("paths".to_string())],
                    }],
                    parameters: vec![],
                }],
                isnt: vec![],
                parameters: vec![Parameters::Set("paths".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("}".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(use_multiple_paths.name.clone(), use_multiple_paths);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("parameters".to_string(), grammar::VariableKind::NodeList);
    variables.insert("return_type".to_string(), grammar::VariableKind::Node);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("docs".to_string(), grammar::VariableKind::NodeList);
    variables.insert("public".to_string(), grammar::VariableKind::Boolean);
    let function = Node {
        name: "KWFunction".to_string(),
        rules: vec![
            Rule::While {
                token: MatchToken::Token(TokenKinds::Complex("doc_comment".to_string())),
                rules: vec![],
                parameters: vec![Parameters::Set("docs".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Word("pub".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::True("public".to_string())],
            },
            Rule::Is {
                token: MatchToken::Word("fun".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Node("parameter".to_string()),
                is: vec![Rule::While {
                    token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                    rules: vec![Rule::Is {
                        token: MatchToken::Node("parameter".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::Set("parameters".to_string())],
                    }],
                    parameters: vec![],
                }],
                isnt: vec![],
                parameters: vec![Parameters::Set("parameters".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Enumerator("types".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("return_type".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(function.name.clone(), function);

    let mut variables = Map::new();
    variables.insert("nodes".to_string(), grammar::VariableKind::NodeList);
    let block = Node {
        name: "block".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("{".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::While {
                token: MatchToken::Enumerator("block_line".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("nodes".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("}".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(block.name.clone(), block);

    let block_line = Enumerator {
        name: "block_line".to_string(),
        values: vec![
            MatchToken::Node("KWFunction".to_string()),
            MatchToken::Node("KWImport".to_string()),
            MatchToken::Node("KWLet".to_string()),
            MatchToken::Node("KWIf".to_string()),
            MatchToken::Node("KWWhile".to_string()),
            MatchToken::Node("KWFor".to_string()),
            MatchToken::Node("KWClass".to_string()),
            MatchToken::Node("KWLoop".to_string()),
            MatchToken::Node("KWEnum".to_string()),
            MatchToken::Node("KWReturn".to_string()),
            MatchToken::Node("KWBreak".to_string()),
            MatchToken::Node("KWContinue".to_string()),
            MatchToken::Node("statement".to_string()),
            MatchToken::Token(TokenKinds::Token(";".to_string())),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(block_line.name.clone(), block_line);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("type".to_string(), grammar::VariableKind::Node);
    variables.insert("rest".to_string(), grammar::VariableKind::Boolean);
    let type_specifier = Node {
        name: "parameter".to_string(),
        rules: vec![
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::True("rest".to_string())],
                }],
                isnt: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("parameter_idents".to_string()),
                rules: vec![],
                parameters: vec![
                    Parameters::Set("identifier".to_string()),
                    Parameters::HardError(true),
                ],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Enumerator("types".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("type".to_string())],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(type_specifier.name.clone(), type_specifier);

    let mut variables = Map::new();
    variables.insert("refs".to_string(), grammar::VariableKind::Number);
    variables.insert("path".to_string(), grammar::VariableKind::Node);
    let type_ = Node {
        name: "type".to_string(),
        rules: vec![
            Rule::Loop {
                rules: vec![Rule::MaybeOneOf {
                    is_one_of: vec![
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&".to_string())),
                            rules: vec![],
                            parameters: vec![Parameters::Increment("refs".to_string())],
                        },
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&&".to_string())),
                            rules: vec![],
                            parameters: vec![
                                Parameters::Increment("refs".to_string()),
                                Parameters::Increment("refs".to_string()),
                            ],
                        },
                    ],
                    isnt: vec![Rule::Command {
                        command: Commands::Goto {
                            label: "end_refs".to_string(),
                        },
                    }],
                }],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end_refs".to_string(),
                },
            },
            Rule::Is {
                token: MatchToken::Node("path".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("path".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(type_.name.clone(), type_);

    let mut variables = Map::new();
    variables.insert("refs".to_string(), grammar::VariableKind::Number);
    variables.insert("type".to_string(), grammar::VariableKind::Node);
    let array_type = Node {
        name: "array_type".to_string(),
        rules: vec![
            Rule::Loop {
                rules: vec![Rule::MaybeOneOf {
                    is_one_of: vec![
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&".to_string())),
                            rules: vec![],
                            parameters: vec![Parameters::Increment("refs".to_string())],
                        },
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&&".to_string())),
                            rules: vec![],
                            parameters: vec![
                                Parameters::Increment("refs".to_string()),
                                Parameters::Increment("refs".to_string()),
                            ],
                        },
                    ],
                    isnt: vec![Rule::Command {
                        command: Commands::Goto {
                            label: "end_refs".to_string(),
                        },
                    }],
                }],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end_refs".to_string(),
                },
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("[".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Node("type".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("type".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("]".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(array_type.name.clone(), array_type);

    let mut variables = Map::new();
    variables.insert("refs".to_string(), grammar::VariableKind::Number);
    variables.insert("types".to_string(), grammar::VariableKind::NodeList);
    let tuple_type = Node {
        name: "tuple_type".to_string(),
        rules: vec![
            Rule::Loop {
                rules: vec![Rule::MaybeOneOf {
                    is_one_of: vec![
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&".to_string())),
                            rules: vec![],
                            parameters: vec![Parameters::Increment("refs".to_string())],
                        },
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&&".to_string())),
                            rules: vec![],
                            parameters: vec![
                                Parameters::Increment("refs".to_string()),
                                Parameters::Increment("refs".to_string()),
                            ],
                        },
                    ],
                    isnt: vec![Rule::Command {
                        command: Commands::Goto {
                            label: "end_refs".to_string(),
                        },
                    }],
                }],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end_refs".to_string(),
                },
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Node("type_list".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("types".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(tuple_type.name.clone(), tuple_type);

    let mut variables = Map::new();
    variables.insert("types".to_string(), grammar::VariableKind::NodeList);
    let type_list = Node {
        name: "type_list".to_string(),
        rules: vec![Rule::Maybe {
            token: MatchToken::Enumerator("types".to_string()),
            is: vec![Rule::While {
                token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                rules: vec![Rule::Is {
                    token: MatchToken::Enumerator("types".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("types".to_string())],
                }],
                parameters: vec![Parameters::Set("types".to_string())],
            }],
            isnt: vec![],
            parameters: vec![
                Parameters::Set("types".to_string()),
                Parameters::HardError(true),
            ],
        }],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(type_list.name.clone(), type_list);

    let types_enum = Enumerator {
        name: "types".to_string(),
        values: vec![
            MatchToken::Node("type".to_string()),
            MatchToken::Node("array_type".to_string()),
            MatchToken::Node("tuple_type".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(types_enum.name.clone(), types_enum);

    let mut variables = Map::new();
    variables.insert("path".to_string(), grammar::VariableKind::NodeList);
    let path = Node {
        name: "path".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![
                    Parameters::Set("path".to_string()),
                    Parameters::HardError(true),
                ],
            },
            Rule::While {
                token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                rules: vec![Rule::Is {
                    token: MatchToken::Token(TokenKinds::Text),
                    rules: vec![],
                    parameters: vec![Parameters::Set("path".to_string())],
                }],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(path.name.clone(), path);

    let mut variables = Map::new();
    variables.insert("nodes".to_string(), grammar::VariableKind::NodeList);
    variables.insert("closure".to_string(), grammar::VariableKind::Node);
    let expression = Node {
        name: "expression".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Node("value".to_string()),
                rules: vec![],
                parameters: vec![
                    Parameters::Set("nodes".to_string()),
                    Parameters::HardError(true),
                ],
            },
            Rule::While {
                token: MatchToken::Enumerator("operators".to_string()),
                rules: vec![Rule::Is {
                    token: MatchToken::Node("value".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("nodes".to_string())],
                }],
                parameters: vec![Parameters::Set("nodes".to_string())],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(expression.name.clone(), expression);

    // expression can have many forms
    let expressions = Enumerator {
        name: "expressions".to_string(),
        values: vec![
            MatchToken::Node("KWIf".to_string()),
            MatchToken::Node("KWLoop".to_string()),
            MatchToken::Node("closure".to_string()),
            MatchToken::Node("expression".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(expressions.name.clone(), expressions);

    let mut variables = Map::new();
    variables.insert("expression".to_string(), grammar::VariableKind::Node);
    // just a wrapper over expression with a semicolon
    let statement = Node {
        name: "statement".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("expression".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(statement.name.clone(), statement);

    let mut variables = Map::new();
    variables.insert("unaries".to_string(), grammar::VariableKind::NodeList);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("tail".to_string(), grammar::VariableKind::Node);
    variables.insert("refs".to_string(), grammar::VariableKind::Node);
    variables.insert("alloc".to_string(), grammar::VariableKind::Boolean);
    variables.insert("dealloc".to_string(), grammar::VariableKind::Boolean);
    let value = Node {
        name: "value".to_string(),
        rules: vec![
            Rule::MaybeOneOf {
                is_one_of: vec![
                    OneOf {
                        token: MatchToken::Word("new".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::True("alloc".to_string())],
                    },
                    OneOf {
                        token: MatchToken::Word("delete".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::True("dealloc".to_string())],
                    },
                ],
                isnt: vec![],
            },
            Rule::While {
                token: MatchToken::Enumerator("unary_operators".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("unaries".to_string())],
            },
            Rule::Is {
                token: MatchToken::Node("value_refs".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("refs".to_string())],
            },
            Rule::IsOneOf {
                tokens: vec![
                    OneOf {
                        token: MatchToken::Node("anonymous_function".to_string()),
                        rules: vec![],
                        parameters: vec![
                            Parameters::Set("body".to_string()),
                            Parameters::HardError(true),
                        ],
                    },
                    OneOf {
                        token: MatchToken::Node("path".to_string()),
                        rules: vec![],
                        parameters: vec![
                            Parameters::Set("body".to_string()),
                            Parameters::HardError(true),
                        ],
                    },
                    OneOf {
                        token: MatchToken::Enumerator("literals".to_string()),
                        rules: vec![],
                        parameters: vec![
                            Parameters::Set("body".to_string()),
                            Parameters::HardError(true),
                        ],
                    },
                    OneOf {
                        token: MatchToken::Node("parenthesis".to_string()),
                        rules: vec![],
                        parameters: vec![
                            Parameters::Set("body".to_string()),
                            Parameters::HardError(true),
                        ],
                    },
                ],
            },
            Rule::Maybe {
                token: MatchToken::Enumerator("tail_options".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("tail".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(value.name.clone(), value);

    let mut variables = Map::new();
    variables.insert("parameters".to_string(), grammar::VariableKind::NodeList);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    let closure = Node {
        name: "closure".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Node("closure_parameter".to_string()),
                is: vec![Rule::While {
                    token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                    rules: vec![Rule::Maybe {
                        token: MatchToken::Node("closure_parameter".to_string()),
                        is: vec![],
                        isnt: vec![Rule::Command {
                            command: Commands::Goto {
                                label: "end".to_string(),
                            },
                        }],
                        parameters: vec![Parameters::Set("parameters".to_string())],
                    }],
                    parameters: vec![],
                }],
                isnt: vec![],
                parameters: vec![Parameters::Set("parameters".to_string())],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end".to_string(),
                },
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(closure.name.clone(), closure);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("rest".to_string(), grammar::VariableKind::Boolean);
    let closure_parameter = Node {
        name: "closure_parameter".to_string(),
        rules: vec![
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                    rules: vec![],
                    parameters: vec![Parameters::True("rest".to_string())],
                }],
                isnt: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("parameter_idents".to_string()),
                rules: vec![],
                parameters: vec![
                    Parameters::Set("identifier".to_string()),
                    Parameters::HardError(true),
                ],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(closure_parameter.name.clone(), closure_parameter);

    // tuple parameter name
    let mut variables = Map::new();
    variables.insert("identifiers".to_string(), grammar::VariableKind::NodeList);
    let tuple_parameter = Node {
        name: "tuple_parameter".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::IsOneOf {
                tokens: vec![
                    OneOf {
                        token: MatchToken::Token(TokenKinds::Text),
                        rules: vec![Rule::While {
                            token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                            rules: vec![Rule::Is {
                                token: MatchToken::Enumerator("parameter_idents".to_string()),
                                rules: vec![],
                                parameters: vec![Parameters::Set("identifiers".to_string())],
                            }],
                            parameters: vec![],
                        }],
                        parameters: vec![Parameters::Set("identifiers".to_string())],
                    },
                    OneOf {
                        token: MatchToken::Node("tuple_parameter".to_string()),
                        rules: vec![Rule::While {
                            token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                            rules: vec![Rule::Is {
                                token: MatchToken::Enumerator("parameter_idents".to_string()),
                                rules: vec![],
                                parameters: vec![Parameters::Set("identifiers".to_string())],
                            }],
                            parameters: vec![],
                        }],
                        parameters: vec![Parameters::Set("identifiers".to_string())],
                    },
                ],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(tuple_parameter.name.clone(), tuple_parameter);

    let parameter_idents = Enumerator {
        name: "parameter_idents".to_string(),
        values: vec![
            MatchToken::Token(TokenKinds::Text),
            MatchToken::Node("tuple_parameter".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(parameter_idents.name.clone(), parameter_idents);

    let mut variables = Map::new();
    variables.insert("parameters".to_string(), grammar::VariableKind::NodeList);
    variables.insert("return_type".to_string(), grammar::VariableKind::Node);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    let anonymous_function = Node {
        name: "anonymous_function".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("fun".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Node("parameter".to_string()),
                is: vec![Rule::While {
                    token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                    rules: vec![Rule::Is {
                        token: MatchToken::Node("parameter".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::Set("parameters".to_string())],
                    }],
                    parameters: vec![],
                }],
                isnt: vec![],
                parameters: vec![Parameters::Set("parameters".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Enumerator("types".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("return_type".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(anonymous_function.name.clone(), anonymous_function);

    let mut variables = Map::new();
    variables.insert("refs".to_string(), grammar::VariableKind::Number);

    let value_refs = Node {
        name: "value_refs".to_string(),
        rules: vec![
            Rule::Loop {
                rules: vec![Rule::MaybeOneOf {
                    is_one_of: vec![
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&".to_string())),
                            rules: vec![],
                            parameters: vec![Parameters::Increment("refs".to_string())],
                        },
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("&&".to_string())),
                            rules: vec![],
                            parameters: vec![
                                Parameters::Increment("refs".to_string()),
                                Parameters::Increment("refs".to_string()),
                            ],
                        },
                        OneOf {
                            token: MatchToken::Token(TokenKinds::Token("*".to_string())),
                            rules: vec![],
                            parameters: vec![Parameters::Decrement("refs".to_string())],
                        },
                    ],
                    isnt: vec![Rule::Command {
                        command: Commands::Goto {
                            label: "end_refs".to_string(),
                        },
                    }],
                }],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end_refs".to_string(),
                },
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(value_refs.name.clone(), value_refs);

    // parenthesis are using the values_list node because it could be a tuple
    let mut variables = Map::new();
    variables.insert("values".to_string(), grammar::VariableKind::Node);
    let parenthesis = Node {
        name: "parenthesis".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Node("values_list".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("values".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(parenthesis.name.clone(), parenthesis);

    // tail options start
    let tail_options = Enumerator {
        name: "tail_options".to_string(),
        values: vec![
            MatchToken::Node("field".to_string()),
            MatchToken::Node("index".to_string()),
            MatchToken::Node("call".to_string()),
            MatchToken::Node("instance".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(tail_options.name.clone(), tail_options);

    let mut variables = Map::new();
    variables.insert("fields".to_string(), grammar::VariableKind::NodeList);
    let instance = Node {
        name: "instance".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("{".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Node("instance_field".to_string()),
                is: vec![Rule::While {
                    token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                    rules: vec![Rule::Maybe {
                        token: MatchToken::Node("instance_field".to_string()),
                        is: vec![],
                        isnt: vec![Rule::Command {
                            command: Commands::Goto {
                                label: "end".to_string(),
                            },
                        }],
                        parameters: vec![Parameters::Set("fields".to_string())],
                    }],
                    parameters: vec![],
                }],
                isnt: vec![],
                parameters: vec![Parameters::Set("fields".to_string())],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end".to_string(),
                },
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("}".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(instance.name.clone(), instance);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("expression".to_string(), grammar::VariableKind::Node);
    let instance_field = Node {
        name: "instance_field".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![
                    Parameters::Set("identifier".to_string()),
                    Parameters::HardError(true),
                ],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("expression".to_string())],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(instance_field.name.clone(), instance_field);

    let mut variables = Map::new();
    variables.insert("field".to_string(), grammar::VariableKind::Node);
    variables.insert("next".to_string(), grammar::VariableKind::Node);
    let field = Node {
        name: "field".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(".".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("field".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Enumerator("tail_options".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("next".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(field.name.clone(), field);

    let mut variables = Map::new();
    variables.insert("index".to_string(), grammar::VariableKind::Node);
    let index = Node {
        name: "index".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("[".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("index".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("]".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(index.name.clone(), index);

    let mut variables = Map::new();
    variables.insert("arguments".to_string(), grammar::VariableKind::Node);
    let call = Node {
        name: "call".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Node("values_list".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("arguments".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(call.name.clone(), call);

    let mut variables = Map::new();
    variables.insert("values".to_string(), grammar::VariableKind::NodeList);
    let values_list = Node {
        name: "values_list".to_string(),
        rules: vec![
            Rule::Maybe {
                token: MatchToken::Enumerator("expressions".to_string()),
                is: vec![Rule::While {
                    token: MatchToken::Token(TokenKinds::Token(",".to_string())),
                    rules: vec![Rule::Maybe {
                        token: MatchToken::Enumerator("expressions".to_string()),
                        is: vec![],
                        isnt: vec![Rule::Command {
                            command: Commands::Goto {
                                label: "end".to_string(),
                            },
                        }],
                        parameters: vec![Parameters::Set("values".to_string())],
                    }],
                    parameters: vec![],
                }],
                isnt: vec![],
                parameters: vec![
                    Parameters::Set("values".to_string()),
                    Parameters::HardError(true),
                ],
            },
            Rule::Command {
                command: Commands::Label {
                    name: "end".to_string(),
                },
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(values_list.name.clone(), values_list);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("type".to_string(), grammar::VariableKind::Node);
    variables.insert("value".to_string(), grammar::VariableKind::Node);
    let kw_let = Node {
        name: "KWLet".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("let".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Enumerator("types".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("type".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token("=".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Enumerator("expressions".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("value".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_let.name.clone(), kw_let);

    let mut variables = Map::new();
    variables.insert("condition".to_string(), grammar::VariableKind::Node);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("next".to_string(), grammar::VariableKind::Node);
    let kw_if = Node {
        name: "KWIf".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("if".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("condition".to_string())],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Node("KWElseIf".to_string()),
                is: vec![],
                isnt: vec![Rule::Maybe {
                    token: MatchToken::Node("KWElse".to_string()),
                    is: vec![],
                    isnt: vec![],
                    parameters: vec![Parameters::Set("next".to_string())],
                }],
                parameters: vec![Parameters::Set("next".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_if.name.clone(), kw_if);

    let mut variables = Map::new();
    variables.insert("condition".to_string(), grammar::VariableKind::Node);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("next".to_string(), grammar::VariableKind::Node);
    let kw_else_if = Node {
        name: "KWElseIf".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("else".to_string()),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Word("if".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("condition".to_string())],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Node("KWElseIf".to_string()),
                is: vec![],
                isnt: vec![Rule::Maybe {
                    token: MatchToken::Node("KWElse".to_string()),
                    is: vec![],
                    isnt: vec![],
                    parameters: vec![Parameters::Set("next".to_string())],
                }],
                parameters: vec![Parameters::Set("next".to_string())],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(kw_else_if.name.clone(), kw_else_if);

    let mut variables = Map::new();
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    let kw_else = Node {
        name: "KWElse".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("else".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_else.name.clone(), kw_else);

    let mut variables = Map::new();
    variables.insert("condition".to_string(), grammar::VariableKind::Node);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("label".to_string(), grammar::VariableKind::Node);
    let kw_while = Node {
        name: "KWWhile".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("while".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Node("loop_label".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("label".to_string())],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("condition".to_string())],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_while.name.clone(), kw_while);

    let mut variables = Map::new();
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("label".to_string(), grammar::VariableKind::Node);
    let kw_loop = Node {
        name: "KWLoop".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("loop".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Node("loop_label".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("label".to_string())],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_loop.name.clone(), kw_loop);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("expression".to_string(), grammar::VariableKind::Node);
    variables.insert("body".to_string(), grammar::VariableKind::Node);
    variables.insert("label".to_string(), grammar::VariableKind::Node);
    let kw_for = Node {
        name: "KWFor".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("for".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Node("loop_label".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("label".to_string())],
            },
            Rule::Is {
                token: MatchToken::Enumerator("parameter_idents".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Is {
                token: MatchToken::Word("in".to_string()),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Enumerator("expressions".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("expression".to_string())],
            },
            Rule::Is {
                token: MatchToken::Node("block".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("body".to_string())],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_for.name.clone(), kw_for);

    let mut variables = Map::new();
    variables.insert("expression".to_string(), grammar::VariableKind::Node);
    let kw_return = Node {
        name: "KWReturn".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("return".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Enumerator("expressions".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("expression".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(kw_return.name.clone(), kw_return);

    let mut variables = Map::new();
    variables.insert("expression".to_string(), grammar::VariableKind::Node);
    variables.insert("label".to_string(), grammar::VariableKind::Node);
    let kw_break = Node {
        name: "KWBreak".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("break".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Node("loop_label".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("label".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Enumerator("expressions".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("expression".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_break.name.clone(), kw_break);

    let mut variables = Map::new();
    variables.insert("label".to_string(), grammar::VariableKind::Node);
    let kw_continue = Node {
        name: "KWContinue".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Word("continue".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Maybe {
                token: MatchToken::Node("loop_label".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::Set("label".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(kw_continue.name.clone(), kw_continue);

    let mut variables = Map::new();
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    let loop_label = Node {
        name: "loop_label".to_string(),
        rules: vec![
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(loop_label.name.clone(), loop_label);

    let mut variables = Map::new();
    variables.insert("docs".to_string(), grammar::VariableKind::NodeList);
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("members".to_string(), grammar::VariableKind::NodeList);
    variables.insert("public".to_string(), grammar::VariableKind::Boolean);
    let kw_class = Node {
        name: "KWClass".to_string(),
        rules: vec![
            Rule::While {
                token: MatchToken::Token(TokenKinds::Complex("doc_comment".to_string())),
                rules: vec![],
                parameters: vec![Parameters::Set("docs".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Word("pub".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::True("public".to_string())],
            },
            Rule::Is {
                token: MatchToken::Word("class".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("{".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::While {
                token: MatchToken::Enumerator("class_members".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("members".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("}".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_class.name.clone(), kw_class);

    let class_members = Enumerator {
        name: "class_members".to_string(),
        values: vec![
            MatchToken::Node("class_field".to_string()),
            MatchToken::Node("KWFunction".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(class_members.name.clone(), class_members);

    let mut variables = Map::new();
    variables.insert("docs".to_string(), grammar::VariableKind::NodeList);
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("type".to_string(), grammar::VariableKind::Node);
    let class_field = Node {
        name: "class_field".to_string(),
        rules: vec![
            Rule::While {
                token: MatchToken::Token(TokenKinds::Complex("doc_comment".to_string())),
                rules: vec![],
                parameters: vec![Parameters::Set("docs".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(":".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Enumerator("types".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("type".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(class_field.name.clone(), class_field);

    let mut variables = Map::new();
    variables.insert("docs".to_string(), grammar::VariableKind::NodeList);
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("members".to_string(), grammar::VariableKind::NodeList);
    variables.insert("public".to_string(), grammar::VariableKind::Boolean);
    let kw_enum = Node {
        name: "KWEnum".to_string(),
        rules: vec![
            Rule::While {
                token: MatchToken::Token(TokenKinds::Complex("doc_comment".to_string())),
                rules: vec![],
                parameters: vec![Parameters::Set("docs".to_string())],
            },
            Rule::Maybe {
                token: MatchToken::Word("pub".to_string()),
                is: vec![],
                isnt: vec![],
                parameters: vec![Parameters::True("public".to_string())],
            },
            Rule::Is {
                token: MatchToken::Word("enum".to_string()),
                rules: vec![],
                parameters: vec![Parameters::HardError(true)],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![Parameters::Set("identifier".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("{".to_string())),
                rules: vec![],
                parameters: vec![],
            },
            Rule::While {
                token: MatchToken::Enumerator("enum_members".to_string()),
                rules: vec![],
                parameters: vec![Parameters::Set("members".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token("}".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser.grammar.nodes.insert(kw_enum.name.clone(), kw_enum);

    let enum_members = Enumerator {
        name: "enum_members".to_string(),
        values: vec![
            MatchToken::Node("KWFunction".to_string()),
            MatchToken::Node("enum_variant".to_string()),
        ],
    };
    parser
        .grammar
        .enumerators
        .insert(enum_members.name.clone(), enum_members);

    let mut variables = Map::new();
    variables.insert("docs".to_string(), grammar::VariableKind::NodeList);
    variables.insert("identifier".to_string(), grammar::VariableKind::Node);
    variables.insert("value".to_string(), grammar::VariableKind::Node);
    variables.insert("fields".to_string(), grammar::VariableKind::NodeList);
    let enum_variant = Node {
        name: "enum_variant".to_string(),
        rules: vec![
            Rule::While {
                token: MatchToken::Token(TokenKinds::Complex("doc_comment".to_string())),
                rules: vec![],
                parameters: vec![Parameters::Set("docs".to_string())],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Text),
                rules: vec![],
                parameters: vec![
                    Parameters::Set("identifier".to_string()),
                    Parameters::HardError(true),
                ],
            },
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token("(".to_string())),
                is: vec![
                    Rule::While {
                        token: MatchToken::Node("class_field".to_string()),
                        rules: vec![],
                        parameters: vec![Parameters::Set("fields".to_string())],
                    },
                    Rule::Is {
                        token: MatchToken::Token(TokenKinds::Token(")".to_string())),
                        rules: vec![],
                        parameters: vec![],
                    },
                ],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Maybe {
                token: MatchToken::Token(TokenKinds::Token("=".to_string())),
                is: vec![Rule::Is {
                    token: MatchToken::Enumerator("expressions".to_string()),
                    rules: vec![],
                    parameters: vec![Parameters::Set("value".to_string())],
                }],
                isnt: vec![],
                parameters: vec![],
            },
            Rule::Is {
                token: MatchToken::Token(TokenKinds::Token(";".to_string())),
                rules: vec![],
                parameters: vec![],
            },
        ],
        variables,
    };
    parser
        .grammar
        .nodes
        .insert(enum_variant.name.clone(), enum_variant);

    // keeps track of all the imported files for faster lookup
    parser
        .grammar
        .globals
        .insert("imports".to_string(), VariableKind::NodeList);

    parser
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn it_works() {
        let start = std::time::Instant::now();
        let parser = gen_parser();

        let validation = parser.grammar.validate(&parser.lexer);

        for error in validation.errors.iter() {
            println!("{:?}", error);
        }

        for warning in validation.warnings.iter() {
            println!("{:?}", warning);
        }

        assert!(validation.pass(), "Grammar is not valid"); // change .pass() to .success() for production

        println!("Parser generation took: {:?}", start.elapsed());
        let start = std::time::Instant::now();

        let test_string = r##"
import "#io"

use io.print.{ahoj.{sedm.*}, *};

/// danda Římani
/// utf8 je zlo na této planetě
pub fun main((a, b): (int, int)) {
    io.přiňtLnffž("Hello, World!", 600. + (9, 8, "ble",), Danda:{
        a: !!!!!!!!!!!!!!!!!!!!!!!!!!!!5c,
        b: 6f,
        c: [10, 20, **&&&30,],
    },);

    do5times(fun(): d {
        io.println();
    });

    closure5times((a,..b, (df,f)):{
        io.println("Hello, World!",
                    5,
                    9,
                    9,
                );
    });

    let a: int = fun (a:int): int {
        a + 5;
    }(5);

    let a = if 9f {
        io.println("Hello, World!");
    } else if a {
        io.println("Hello, World!");
    } else {
        for (a, idx) in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10,] {
            io.println("Hello, World!");
        }
        let a = loop :var_a: {
            io.println("Hello, World!");
            break :var_a: 5c;
        };
        while true {
            break;
        }
    };
}

fun sum_args(..numbers: int): int {}

fun sum_array(numbers: &[&&&int]): int {}


/// Tahleta třída je pro testování
/// 
/// Ať tě ani nenapadne ji použít
class Danda {
    a: int;
    b: int;
    c: int;
    fun sum(a: int, b: char): int {
        a + b + !**5c;
    }
}

pub enum A {
    a; // = 0
    b = 7;
    c(
        /// První parametr
        first: int;
        second: &(int, float);
    ) = 5;

    fun new() {
        let option1 = A.c(5, (5, 5.5));
        let option2 = new A.c:{
            first: 5,
            second: (5, 5.5),
        };
        return option1;
    }
}


"##;

        let tokens = parser.lexer.lex_utf8(test_string).unwrap();

        /*for token in &tokens {
            println!(
                "{}",
                test_string[token.index..token.index + token.len].to_string()
            );
        }*/

        println!("Lexer took: {:?}", start.elapsed());
        let start = std::time::Instant::now();

        //println!("{:?}", tokens.as_ref().unwrap());
        let ast = parser.parse(&tokens, test_string);
        println!("Parser took: {:?}", start.elapsed());

        let str = serde_json::to_string(&parser).unwrap();
        let mut file = std::fs::File::create("ruda_grammar.json").unwrap();
        file.write_all(str.as_bytes()).unwrap();

        panic!("{:?}", ast.unwrap());
    }
}
