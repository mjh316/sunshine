mod ast;
mod interpreter;
mod lexer;
mod parser;
mod stdlib;

use std::{cell::RefCell, collections::HashMap, env, rc::Rc};

use ast::{Array, Ast, Literal};
use interpreter::Interpreter;
use lexer::TokenContentType;

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

            // array functions
            standardLibraryFunctions.borrow_mut().insert(
                "STDLIB_ARRAY_PUSH".to_string(),
                Box::new(move |args| {
                    // println!("ARGS IN ARRAY PUSH {:?}", args);

                    return match args.get(0) {
                        Some(Ast::Array(array)) => {
                            let mut array = array.clone();
                            array.content.push(args.get(1).unwrap().clone());
                            Ast::Array(array)
                        }
                        _ => panic!("Expected array as first argument"),
                    };
                }),
            );

            standardLibraryFunctions.borrow_mut().insert(
                "STDLIB_ARRAY_POP".to_string(),
                Box::new(move |args| {
                    // println!("ARGS IN ARRAY POP {:?}", args);

                    return match args.get(0) {
                        Some(Ast::Array(array)) => {
                            let mut array = array.clone();
                            array.content.pop();
                            Ast::Array(array)
                        }
                        _ => panic!("Expected array as first argument"),
                    };
                }),
            );

            // reverse
            standardLibraryFunctions.borrow_mut().insert(
                "STDLIB_ARRAY_REVERSE".to_string(),
                Box::new(move |args| {
                    // println!("ARGS IN ARRAY REVERSE {:?}", args);

                    return match args.get(0) {
                        Some(Ast::Array(array)) => {
                            let mut array = array.clone();
                            array.content.reverse();
                            Ast::Array(array)
                        }
                        _ => panic!("Expected array as first argument"),
                    };
                }),
            );

            // sort
            standardLibraryFunctions.borrow_mut().insert(
                "STDLIB_ARRAY_SORT".to_string(),
                Box::new(move |args| {
                    // println!("ARGS IN ARRAY SORT {:?}", args);

                    return match args.get(0) {
                        Some(Ast::Array(array)) => {
                            let mut array = array.clone();
                            array.content.sort_by(|a, b| {
                                let ab = match a {
                                    Ast::Literal(Literal {
                                        content: TokenContentType::Number(a),
                                    }) => Some(a),
                                    _ => None,
                                };
                                let bb = match b {
                                    Ast::Literal(Literal {
                                        content: TokenContentType::Number(b),
                                    }) => Some(b),
                                    _ => None,
                                };
                                if ab.is_some() && bb.is_some() {
                                    return ab.unwrap().partial_cmp(&bb.unwrap()).unwrap();
                                }

                                let a = match a {
                                    Ast::Literal(Literal {
                                        content: TokenContentType::String(abs),
                                    }) => Some(abs),
                                    _ => None,
                                };
                                let b = match b {
                                    Ast::Literal(Literal {
                                        content: TokenContentType::String(b),
                                    }) => Some(b),
                                    _ => None,
                                };
                                if a.is_some() && b.is_some() {
                                    return a.partial_cmp(&b).unwrap();
                                }
                                panic!("Expected number or string as array elements!")
                            });
                            Ast::Array(array)
                        }
                        _ => panic!("Expected array as first argument"),
                    };
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
