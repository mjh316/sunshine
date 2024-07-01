mod ast;
mod interpreter;
mod lexer;
mod parser;
mod stdlib;

use std::{cell::RefCell, collections::HashMap, env, rc::Rc};

use ast::{Ast, Literal};
use interpreter::Interpreter;

// use interpreter::Interpreter;

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
    if debug {
        println!("Debug: {}", debug);
    }
    if debug {
        argv.retain(|x| x != "--dbg");
    }

    let location = argv.get(1);

    match location {
        Some(location) => {
            if debug {
                println!("Reading file: {}", location);
            }
            let program = read_file(location);
            let mut lexer = lexer::Lexer::new(program);
            let tokens = lexer.scan_tokens();

            if debug {
                write_file(
                    "tokens.txt",
                    format!("{:#?}", serde_json::to_string(&tokens.clone()).unwrap()).as_str(),
                );
            }

            let mut parser = parser::Parser::new(tokens);

            let ast = parser.parse();

            if debug {
                write_file(
                    "ast.txt",
                    format!(
                        "{:#?}",
                        serde_json::to_string(&ast.clone()).unwrap().as_str()
                    )
                    .as_str(),
                );
            }
            // println!("{}", program);

            let standardLibraryFunctions: Rc<
                RefCell<HashMap<String, Box<dyn Fn(Vec<Ast>) -> Ast>>>,
            > = Rc::new(RefCell::new(HashMap::new()));

            let scope = Rc::new(RefCell::new(HashMap::new()));
            let structScope = Rc::new(RefCell::new(HashMap::new()));

            let borrowedScope = Rc::clone(&scope);
            let borrowedStandardLibraryFunctions = Rc::clone(&standardLibraryFunctions);
            let borrowedStructScope = Rc::clone(&structScope);

            standardLibraryFunctions.borrow_mut().insert(
                "print".to_string(),
                Box::new(move |args: Vec<Ast>| {
                    let scope = Rc::clone(&borrowedScope);
                    let standardLibraryFunctions = Rc::clone(&borrowedStandardLibraryFunctions);
                    let structScope = Rc::clone(&borrowedStructScope);
                    for arg in args {
                        println!(
                            "{}",
                            Interpreter::toPrint(
                                arg,
                                Rc::clone(&scope),
                                Rc::clone(&standardLibraryFunctions),
                                Rc::clone(&structScope)
                            )
                        );
                    }
                    Ast::None
                }),
            );

            standardLibraryFunctions.borrow_mut().insert(
                "input".to_string(),
                Box::new(move |_| {
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");
                    Ast::Literal(Literal {
                        content: input.trim().to_string().into(),
                    })
                }),
            );

            Interpreter::run(ast, scope, standardLibraryFunctions, structScope);
        }
        None => {
            // No file provided, go to REPL?
            println!("Usage: {} <file>", argv[0]);
            return;
        }
    }
}
