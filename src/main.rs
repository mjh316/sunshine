mod ast;
mod interpreter;
mod lexer;
mod parser;
mod stdlib;

use std::{cell::RefCell, collections::HashMap, env, rc::Rc};

use interpreter::Interpreter;

fn read_file(location: &str) -> String {
    std::fs::read_to_string(location)
        .expect("Failed to read file")
        .trim()
        .to_string()
}

#[allow(dead_code)]
fn write_file(location: &str, data: &str) -> () {
    std::fs::write(location, data).expect("Failed to write file");
}

fn main() {
    let mut argv = env::args().collect::<Vec<String>>();
    let debug = argv.iter().any(|x| x == "--dbg");
    println!("Debug: {}", debug);
    if debug {
        argv.retain(|x| x != "--dbg");
    }

    let location = argv.get(1);

    match location {
        Some(location) => {
            println!("Reading file: {}", location);
            let program = read_file(location);
            let mut lexer = lexer::Lexer::new(program);
            let tokens = lexer.scan_tokens();

            if debug {
                write_file(
                    "tokens.txt",
                    format!("{:#?}", serde_json::to_string(&tokens.clone()).unwrap()).as_str(),
                );
            }

            // let mut parser = parser::Parser::new(tokens);

            // let ast = parser.parse();

            // if debug {
            //     write_file(
            //         "ast.txt",
            //         format!(
            //             "{:#?}",
            //             ast.clone()
            //                 .into_iter()
            //                 .map(|x| String::from(x))
            //                 .collect::<Vec<String>>()
            //         )
            //         .as_str(),
            //     );
            // }

            // Interpreter::run(ast, Rc::new(RefCell::new(HashMap::new())));
            // println!("{}", program);
        }
        None => {
            // No file provided, go to REPL?
            println!("Usage: {} <file>", argv[0]);
            return;
        }
    }
}
