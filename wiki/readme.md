# Parser wiki

This section contains guide for using this library.

## Steps

Parsing pipeline contains a few steps. Each step will have an output that is directly fed to the next one.

steps:

 - Lexing
 - token proccessing
 - AST generation

### Lexing

First part of parsing is called lexing. It takes the input text and converts it into an array of predefined tokens based on a few criteria:

The tokens are always defined by the user

1. Is it a newline?
2. Is it a different whitespace?
3. Is it a predefined token?
4. If not. It is a word.

To give an example:

```
tokens [ = ; : + - * / ]

text "let a: int = 10;\n"

tokenized text [(word let), (ws space), (word a), (token :), (ws space), (word int), (ws space), (token =), (ws space), (word 10), (token ;), (control newLine)]
```

Notice how the token for number ten is still a word. This is because the base lexer does not do any intrusive logic for the user. This is left for the next phase:

### Token proccessing

