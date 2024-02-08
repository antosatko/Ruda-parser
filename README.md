# Ruda parser

## Description

Ruda parser is a general purpose parser based on the parser developed for the [Ruda programming language](https://github.com/it-2001/Ruda/tree/main). This parser is designed to be used in any project that requires a a custom parser.

It is up to the user to define the tokens and the grammar of the text to be parsed. The parser will then use the grammar to parse the text and return a parse tree. It can currently be configured using a dumped json file or by using the `Parser` class directly.

I intend to add a grammar file format in the future for easier configuration.

## Features

- [x] Parse text using a dumped json file
- [x] Parse text using the `Parser` class
- [ ] Parse text using a custom grammar
- [x] Switch between different file formats
    - [x] ASCII
    - [x] UTF-8
- [ ] WASM support
- [ ] Port to other languages
- [ ] CLI tool for parsing text

## Usage

This section will be updated once the parser is more stable.

Until then, you can look at the tests in the `src/lib.rs` file to see how to use the parser.

### Running the tests

Before running the tests, make sure to run:

```bash
py genWorkload.py
```

This will generate a file called `workload.txt` which is used by the tests to test the parser. If it takes too long to run the tests, you can reduce the size of the file by changing global variables in the `genWorkload.py` file. (It took about 4 minutes to generate the file on my machine with the default settings.). This will generate a file with a size of about 1GB by default. (The script could be optimized better. It is just that I don't care enough to do it. It is just a test file after all.)

Also run the tests more than once. This is because one of the tests creates a dump grammar file and other one uses it.

You can use the following command:

```bash
cargo test --release # --release is optional but recommended since it could take longer to run the tests without it
```

## Random thoughts

This needs to be said. During the development of the parser, I realized that UTF-8 is evil. it is fully supported and will be supported in the future, but it is evil.

Also I will probably rewrite Ruda to use this parser instead of the current one since this one is much better, faster and more flexible with the added benefit of adding UTF-8 support (not currently supported in Ruda).

And finally, Don't underestimate the power of a profiler. I spent a lot of time optimizing the parser and the lexer and I would have never been able to do it without a profiler. Just to put it to perspective. The lexer used to take about 30s to parse only 100kB of text. Now it takes 4s to parse 1GB of the same text. the first iteration is a result of poor design but having a profiler helped me find the bottlenecks and fix them as well as learn a lot about the performance of the code and CPU in general.

The fact that you are looking for a parser means that you are probably working on a programming language. If that is the case, I wish you the best of luck. It is a very difficult task but it is also very rewarding. I hope that if you choose to use this parser, it will help you in your journey. You are awesome, take care.