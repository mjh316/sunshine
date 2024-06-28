use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::{Array, Ast, Literal},
    lexer::{TokenContentType, TokenType},
};

pub struct Interpreter {}

impl Interpreter {
    pub fn run(ast: Vec<Ast>, scope: Cell<HashMap<String, Ast>>) {}

    fn inScope(scope: Rc<RefCell<HashMap<String, Ast>>>, name: String) -> bool {
        scope.borrow_mut().contains_key(&name)
    }

    pub fn evaluate(value: Box<Ast>, scope: Rc<RefCell<HashMap<String, Ast>>>) -> Ast {
        match *value {
            Ast::Var(name, _) => {
                if !Interpreter::inScope(scope.clone(), name.clone()) {
                    panic!("Variable {} not found in scope", name);
                }
                return scope.borrow_mut().get(&name).unwrap().clone();
            }
            Ast::Unary(operator, value) => {
                let value = Interpreter::evaluate(value, scope.clone());
                match operator {
                    TokenType::Not => return !value,
                    _ => {
                        panic!("Unknown unary operator {:?}", operator);
                    }
                }
            }
            Ast::Binary(left, op, right) => {
                unimplemented!("Binary operator not implemented yet");
            }
            Ast::Literal(literal) => Ast::Literal(literal),
            Ast::Array(array) => Ast::Array(Array {
                content: array
                    .content
                    .into_iter()
                    .map(|x| Interpreter::evaluate(Box::new(x.clone()), scope.clone()))
                    .collect::<Vec<_>>(),
            }),
            Ast::Instance(name, members) => {
                if !Interpreter::inScope(scope.clone(), name.clone()) {
                    panic!("Instance {} not found in scope", name);
                }

                /**
                 * TODO: fix this like in the struct rep lmao
                 */
                let constructor = scope.borrow_mut().get(&name).unwrap();
                let mut instance = HashMap::new();
                for (key, value) in members {
                    instance.insert(key, Interpreter::evaluate(Box::new(value), scope.clone()));
                }
                return Ast::Instance(name, instance);
            }
            _ => {
                panic!("Expected expression but got statement {:?}", value);
            }
        }
    }

    pub fn execute(
        node: Ast,
        scope: Rc<RefCell<HashMap<String, Ast>>>,
    ) -> Rc<RefCell<HashMap<String, Ast>>> {
        match node {
            Ast::Var(name, Some(value)) => {
                let value = Interpreter::evaluate(value, scope.clone()); // pray to god scope.clone works here
                scope.borrow_mut().insert(name, value);
            }
            // TODO: lookup correct impl, see why it returns a function
            Ast::Struct(id, params) => {
                let mut struct_scope = scope.borrow_mut().clone();
                struct_scope.insert(id.clone(), Ast::Struct(id.clone(), params.clone()));
                scope
                    .borrow_mut()
                    .insert(id.clone(), Ast::Struct(id.clone(), params));
            }
            Ast::Func(name, params, body) => {}
            Ast::Set(_, _, _)
            | Ast::Return(_)
            | Ast::While(_, _)
            | Ast::For(_, _, _)
            | Ast::Conditional(_, _, _)
            | _ => {
                // Interpreter::evaluate(Box::new(node), &mut scope);
            }
        }
        scope
    }
}
