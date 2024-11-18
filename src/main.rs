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

    // let (command, filename) = ("run", "test.lox");

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });
    let mut exit_code = 0;
    let (tokens, errors) = lex::Tokenizer::new(&file_contents).tokenize();
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error);
        }
        exit_code = 65;
    }
    if command == "tokenize" {
        for token in tokens.iter() {
            println!("{}", token);
        }
        std::process::exit(exit_code);
    }

    if command == "parse" {
        let (exprs, errors) = parse::Parser::from_tokens(tokens).parse_expr();
        if !errors.is_empty() {
            for error in errors {
                eprintln!("{}", error);
            }
            exit_code = 65;
        }
        for expr in exprs.iter() {
            println!("{}", expr);
        }
        std::process::exit(exit_code);
    }

    if command == "evaluate" {
        let (exprs, errors) = parse::Parser::from_tokens(tokens).parse_expr();
        if !errors.is_empty() {
            for error in errors {
                eprintln!("{}", error);
            }
            std::process::exit(65);
        }

        let interpreter = interpreter::Interpreter::new();
        for expr in exprs.iter() {
            if let Err(e) = interpreter.eval(expr) {
                eprintln!("{}", e);
                exit_code = 70;
                break;
            }
        }

        std::process::exit(exit_code);
    }

    if command == "run" {
        let (exprs, errors) = parse::Parser::from_tokens(tokens).parse();
        if !errors.is_empty() {
            for error in errors {
                eprintln!("{}", error);
            }
            std::process::exit(65);
        }

        let interpreter = interpreter::Interpreter::new();
        if let Err(e) = interpreter.interpret(&exprs) {
            eprintln!("{}", e);
            exit_code = 70;
        }

        std::process::exit(exit_code);
    }
}
