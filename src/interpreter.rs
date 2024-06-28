use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::{Ast, Literal},
    lexer::TokenContentType,
};

pub struct Interpreter {}

impl Interpreter {
    pub fn run(ast: Vec<Ast>, scope: Cell<HashMap<String, Ast>>) {}

    fn inScope(scope: Rc<RefCell<HashMap<String, Ast>>>, name: String) -> bool {
        scope.borrow_mut().contains_key(&name)
    }

    pub fn evaluate(value: Box<Ast>, scope: Rc<RefCell<HashMap<String, Ast>>>) -> Ast {
        Ast::Literal(Literal::from(TokenContentType::String("".to_string())))
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
            Ast::Set(_, _, _)
            | Ast::Struct(_, _)
            | Ast::Func(_, _, _)
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
