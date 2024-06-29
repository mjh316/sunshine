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

type Scope = Rc<RefCell<HashMap<String, Ast>>>;
impl Interpreter {
    pub fn run(ast: Vec<Ast>, scope: Scope) -> Scope {
        let mut retScope = scope.clone();
        for node in ast {
            let (newScope, _ret) = Interpreter::execute(node, retScope.clone());
            retScope = newScope;
            // if ret.is_some() {
            //     return ret;
            // }
        }
        retScope
    }

    fn inScope(scope: Scope, name: String) -> bool {
        scope.borrow_mut().contains_key(&name)
    }

    pub fn evaluate(value: Box<Ast>, scope: Scope) -> Ast {
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
                let mut instance = HashMap::new();
                for (key, value) in members {
                    instance.insert(key, Interpreter::evaluate(Box::new(value), scope.clone()));
                }
                return Ast::Instance(name, instance);
            }
            Ast::Call(caller, args) => {
                let caller = Interpreter::evaluate(caller, scope.clone());
                let args = args
                    .into_iter()
                    .map(|x| Interpreter::evaluate(Box::new(x), scope.clone()))
                    .collect::<Vec<_>>();

                // caller should be a function, so we can call it somehow
                unimplemented!("Call not implemented yet");
            }
            _ => {
                panic!("Expected expression but got statement {:?}", value);
            }
        }
    }

    pub fn execute(node: Ast, _scope: Scope) -> (Scope, Option<Ast>) {
        let mut retValue = None;
        let mut retScope = _scope.clone();
        match node {
            Ast::Var(name, Some(value)) => {
                let value = Interpreter::evaluate(value, retScope.clone()); // pray to god scope.clone works here
                retScope.borrow_mut().insert(name, value);
            }
            // TODO: lookup correct impl, see why it returns a function
            Ast::Struct(id, params) => {
                unimplemented!("Struct not implemented yet")
            }
            Ast::Func(name, params, body) => {
                unimplemented!("Function not implemented yet")
            }
            Ast::Return(value) => {
                let value = Interpreter::evaluate(value, retScope.clone());
                retValue = Some(value);
            }
            Ast::While(condition, body) => loop {
                let condition = Interpreter::evaluate(condition.clone(), retScope.clone());
                if matches!(
                    condition,
                    Ast::Literal(Literal {
                        content: TokenContentType::Boolean(false)
                    })
                ) {
                    break;
                }
                // if condition == Ast::Literal(Literal {
                //     content: TokenContentType::Boolean(false),
                // }) {
                //     break;
                // }
                for statement in body.clone() {
                    let (newScope, ret) = Interpreter::execute(statement, retScope.clone());
                    retScope = newScope;
                    if ret.is_some() {
                        retValue = ret;
                        break;
                    }
                }
            },
            Ast::For(id, range, body) => {
                assert!(range.len() == 2);
                let mut localScope = retScope.clone();

                let mut rangeBegin = 0;
                let mut rangeEnd = 0;

                let rangeBeginAst =
                    Interpreter::evaluate(Box::new(range[0].clone()), retScope.clone());
                if !matches!(
                    rangeBeginAst,
                    Ast::Literal(Literal {
                        content: TokenContentType::Number(_)
                    })
                ) {
                    panic!("Expected number as range begin, got {:?}", rangeBegin);
                } else {
                    if let Ast::Literal(Literal {
                        content: TokenContentType::Number(n),
                    }) = rangeBeginAst
                    {
                        rangeBegin = n as i64;
                    }
                }

                localScope.borrow_mut().insert(
                    id.clone(),
                    Ast::Literal(Literal {
                        content: rangeBegin.into(),
                    }),
                );

                let rangeEndAst =
                    Interpreter::evaluate(Box::new(range[1].clone()), retScope.clone());
                if !matches!(
                    rangeEndAst,
                    Ast::Literal(Literal {
                        content: TokenContentType::Number(_)
                    })
                ) {
                    panic!("Expected number as range end, got {:?}", rangeEnd);
                } else {
                    if let Ast::Literal(Literal {
                        content: TokenContentType::Number(n),
                    }) = rangeEndAst
                    {
                        rangeEnd = n as i64;
                    }
                }

                for _ in rangeBegin..rangeEnd {
                    Interpreter::run(body.clone(), localScope.clone());

                    // increment the loop variable
                    let copyId = id.clone();
                    let loopVar = localScope.borrow_mut().get(&copyId).unwrap().clone();
                    if let Ast::Literal(Literal {
                        content: TokenContentType::Number(n),
                    }) = loopVar
                    {
                        localScope.borrow_mut().insert(
                            id.clone(),
                            Ast::Literal(Literal {
                                content: (n as i64 + 1).into(),
                            }),
                        );
                    }
                }
            }
            Ast::Conditional(condition, ifBody, elseBody) => {
                let conditionEvaluated = Interpreter::evaluate(condition, retScope.clone());
                if matches!(
                    conditionEvaluated,
                    Ast::Literal(Literal {
                        content: TokenContentType::Boolean(false)
                    })
                ) {
                    for statement in elseBody {
                        Interpreter::execute(statement, retScope.clone());
                    }
                } else {
                    Interpreter::run(ifBody, retScope.clone());
                }
            }
            Ast::Set(caller, value, property) => {
                if !Interpreter::inScope(retScope.clone(), caller.clone()) {
                    panic!("Instance {} not found in scope", caller);
                }

                unimplemented!("Set not implemented yet");
            }
            Ast::Get(_, _, _) => {
                unimplemented!("Get not implemented yet");
            }

            _ => {
                Interpreter::evaluate(Box::new(node), retScope.clone());
            }
        }
        (retScope, retValue)
    }
}
