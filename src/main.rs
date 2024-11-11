use std::env;
use std::fs;
mod interpreter;
mod lex;
mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    // let (command, filename) = ("parse", "test.lox");

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });

    let (tokens, errors) = lex::Tokenizer::new(&file_contents).tokenize();
    if command == "tokenize" {
        for token in tokens.iter() {
            println!("{}", token);
        }
    }
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error);
        }
        std::process::exit(65);
    }

    let (exprs, _errors) = parse::Parser::from_tokens(tokens).parse();
    if command == "parse" {
        for expr in exprs.iter() {
            println!("{}", expr);
        }
    }
}
