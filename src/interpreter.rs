use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::{Array, Ast, Literal},
    lexer::{Token, TokenContentType, TokenType},
};

pub struct Interpreter {}

type Scope = Rc<RefCell<HashMap<String, Ast>>>;
type FunctionScope = Rc<RefCell<HashMap<String, Box<dyn FnMut(Vec<Ast>) -> Ast>>>>;
type StructScope = Rc<RefCell<HashMap<String, HashMap<String, Ast>>>>; // this only stores the params each thing has, not the actual instances
                                                                       // those go in the scope

impl Interpreter {
    pub fn run(
        ast: Vec<Ast>,
        scope: Scope,
        functionScope: FunctionScope,
        structScope: StructScope,
    ) -> (Scope, Option<Ast>) {
        let mut retScope = scope.clone();
        for node in ast {
            let (newScope, _ret) = Interpreter::execute(
                node,
                retScope.clone(),
                functionScope.clone(),
                structScope.clone(),
            );
            retScope = newScope;
            if _ret.is_some() {
                return (retScope, _ret);
            }
            // if ret.is_some() {
            //     return ret;
            // }
        }
        (retScope, None)
    }

    fn inScope(scope: Scope, name: String) -> bool {
        scope.borrow_mut().contains_key(&name)
    }

    fn isFuncInScope(functionScope: FunctionScope, name: String) -> bool {
        functionScope.borrow_mut().contains_key(&name)
    }

    fn isStructInScope(structScope: StructScope, name: String) -> bool {
        structScope.borrow_mut().contains_key(&name)
    }

    pub fn evaluate(
        value: Box<Ast>,
        scope: Scope,
        functionScope: FunctionScope,
        structScope: StructScope,
    ) -> Ast {
        match *value {
            Ast::Var(name, _) => {
                if !Interpreter::inScope(scope.clone(), name.clone()) {
                    panic!("Variable {} not found in scope", name);
                }
                return scope.borrow_mut().get(&name).unwrap().clone();
            }
            Ast::Unary(operator, value) => {
                let value = Interpreter::evaluate(
                    value,
                    scope.clone(),
                    functionScope.clone(),
                    structScope.clone(),
                );
                match operator {
                    TokenType::Not => return !value,
                    _ => {
                        panic!("Unknown unary operator {:?}", operator);
                    }
                }
            }
            Ast::Binary(left, op, right) => {
                let mut operations: HashMap<TokenType, Box<dyn Fn(Ast, Ast) -> Ast>> =
                    HashMap::new();
                operations.insert(TokenType::Plus, Box::new(|a, b| a + b));
                operations.insert(TokenType::Minus, Box::new(|a, b| a - b));
                operations.insert(TokenType::Asterisk, Box::new(|a, b| a * b));
                operations.insert(TokenType::Slash, Box::new(|a, b| a / b));
                operations.insert(TokenType::Modulo, Box::new(|a, b| a % b));
                operations.insert(
                    TokenType::Equiv,
                    Box::new(|a, b| {
                        Ast::Literal(Literal {
                            content: (a == b).into(),
                        })
                    }),
                );
                operations.insert(
                    TokenType::NotEquiv,
                    Box::new(|a, b| {
                        Ast::Literal(Literal {
                            content: (a != b).into(),
                        })
                    }),
                );
                operations.insert(
                    TokenType::Gt,
                    Box::new(|a, b| {
                        Ast::Literal(Literal {
                            content: (a > b).into(),
                        })
                    }),
                );
                operations.insert(
                    TokenType::Gte,
                    Box::new(|a, b| {
                        Ast::Literal(Literal {
                            content: (a >= b).into(),
                        })
                    }),
                );
                operations.insert(
                    TokenType::Lt,
                    Box::new(|a, b| {
                        Ast::Literal(Literal {
                            content: (a < b).into(),
                        })
                    }),
                );
                operations.insert(
                    TokenType::Lte,
                    Box::new(|a, b| {
                        Ast::Literal(Literal {
                            content: (a <= b).into(),
                        })
                    }),
                );
                operations.insert(
                    TokenType::And,
                    Box::new(|a, b| match [a, b] {
                        [Ast::Literal(Literal {
                            content: TokenContentType::Boolean(aBool),
                        }), Ast::Literal(Literal {
                            content: TokenContentType::Boolean(bBool),
                        })] => Ast::Literal(Literal {
                            content: TokenContentType::Boolean(aBool && bBool),
                        }),
                        _ => panic!("Expected boolean values for AND operation"),
                    }),
                );
                operations.insert(
                    TokenType::Or,
                    Box::new(|a, b| match [a.clone(), b.clone()] {
                        [Ast::Literal(Literal {
                            content: TokenContentType::Boolean(aBool),
                        }), Ast::Literal(Literal {
                            content: TokenContentType::Boolean(bBool),
                        })] => Ast::Literal(Literal {
                            content: TokenContentType::Boolean(aBool || bBool),
                        }),
                        _ => panic!("Expected boolean values for AND operation"),
                    }),
                );

                let left = Interpreter::evaluate(
                    left,
                    scope.clone(),
                    functionScope.clone(),
                    structScope.clone(),
                );
                let right = Interpreter::evaluate(
                    right,
                    scope.clone(),
                    functionScope.clone(),
                    structScope.clone(),
                );

                if !operations.contains_key(&op) {
                    panic!("Unknown binary operator {:?}", op);
                }

                return operations.get(&op).unwrap()(left, right);
            }
            Ast::Literal(literal) => Ast::Literal(literal),
            Ast::Array(array) => Ast::Array(Array {
                content: array
                    .content
                    .into_iter()
                    .map(|x| {
                        Interpreter::evaluate(
                            Box::new(x.clone()),
                            scope.clone(),
                            functionScope.clone(),
                            structScope.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
            }),
            Ast::Instance(name, members) => {
                if !Interpreter::isStructInScope(structScope.clone(), name.clone()) {
                    panic!("Instance {} not found in scope", name);
                }

                let mut structScopeMap = structScope.borrow_mut();
                let mut instanceConstructor = structScopeMap
                    .get(&name)
                    .expect(format!("Struct {} not found in scope", name).as_str());

                let mut fields: HashMap<String, Ast> = HashMap::new();
                for (field, fieldValue) in members {
                    if !instanceConstructor.contains_key(&field) {
                        panic!("Field {} not found in struct {}", field, name);
                    }
                    fields.insert(
                        field,
                        Interpreter::evaluate(
                            Box::new(fieldValue),
                            scope.clone(),
                            functionScope.clone(),
                            structScope.clone(),
                        ),
                    );
                }
                return Ast::Instance(name, fields);
            }
            Ast::Call(caller, args) => {
                let caller = Interpreter::evaluate(
                    caller,
                    scope.clone(),
                    functionScope.clone(),
                    structScope.clone(),
                );
                let args = args
                    .into_iter()
                    .map(|x| {
                        Interpreter::evaluate(
                            Box::new(x),
                            scope.clone(),
                            functionScope.clone(),
                            structScope.clone(),
                        )
                    })
                    .collect::<Vec<_>>();

                // caller should be a function, so we can call it somehow
                unimplemented!("Call not implemented yet");
            }
            _ => {
                panic!("Expected expression but got statement {:?}", value);
            }
        }
    }

    pub fn execute(
        node: Ast,
        _scope: Scope,
        _functionScope: FunctionScope,
        _structScope: StructScope,
    ) -> (Scope, Option<Ast>) {
        let mut retValue = None;
        let mut retScope = _scope.clone();
        let mut retFunctionScope = _functionScope.clone();
        let mut retStructScope = _structScope.clone();
        match node {
            Ast::Var(name, Some(value)) => {
                let value = Interpreter::evaluate(
                    value,
                    retScope.clone(),
                    retFunctionScope.clone(),
                    retStructScope.clone(),
                ); // pray to god scope.clone works here
                retScope.borrow_mut().insert(name, value);
            }
            // TODO: lookup correct impl, see why it returns a function
            Ast::Struct(id, params) => {
                // let constructor = Box::new(move |fields: HashMap<String, Ast>| {
                //     let mut instance = HashMap::new();
                //     for (field, fieldValue) in fields {
                //         if params.contains(&field) {
                //             instance.insert(field, fieldValue);
                //         } else {
                //             panic!("Field {} not found in struct {}", field, id);
                //         }
                //     }
                //     Ast::Instance(id.clone(), instance)
                // });
                let mut fields = HashMap::new();
                for field in &params {
                    fields.insert(field.to_owned(), Ast::None);
                }

                retStructScope.borrow_mut().insert(id, fields);
            }
            Ast::Func(name, params, body) => {
                let functionScope = retFunctionScope.clone();
                let function = Box::new(move |args: Vec<Ast>| {
                    let mut localScope = functionScope.clone();
                    for 
                });

                retFunctionScope.borrow_mut().insert(name, function);
            }
            Ast::Return(value) => {
                let value = Interpreter::evaluate(
                    value,
                    retScope.clone(),
                    retFunctionScope.clone(),
                    retStructScope.clone(),
                );
                retValue = Some(value);
                return (retScope, retValue);
            }
            Ast::While(condition, body) => loop {
                loop {
                    let result = Interpreter::execute(
                        *condition.clone(),
                        retScope.clone(),
                        retFunctionScope.clone(),
                        retStructScope.clone(),
                    );
                    let condition = result.1.unwrap();
                    match condition {
                        Ast::Literal(Literal {
                            content: TokenContentType::Boolean(false),
                        }) => break,
                        Ast::Literal(Literal {
                            content: TokenContentType::Boolean(true),
                        }) => {
                            Interpreter::run(
                                body.clone(),
                                retScope.clone(),
                                retFunctionScope.clone(),
                                retStructScope.clone(),
                            );
                        }
                        _ => {
                            panic!(
                                "Expected boolean value in while condition, got {:?}",
                                condition
                            );
                        }
                    }
                }
                // let condition = Interpreter::evaluate(
                //     condition.clone(),
                //     retScope.clone(),
                //     retFunctionScope.clone(), retStructScope.clone(),
                // );
                // if matches!(
                //     condition,
                //     Ast::Literal(Literal {
                //         content: TokenContentType::Boolean(false)
                //     })
                // ) {
                //     break;
                // }
                // // if condition == Ast::Literal(Literal {
                // //     content: TokenContentType::Boolean(false),
                // // }) {
                // //     break;
                // // }
                // for statement in body.clone() {
                //     let (newScope, ret) =
                //         Interpreter::execute(statement, retScope.clone(), retFunctionScope.clone(), retStructScope.clone());
                //     retScope = newScope;
                //     if ret.is_some_and(|x| match x {
                //         Ast::Literal(Literal {
                //             content: TokenContentType::Boolean(boolValue),
                //         }) => boolValue,
                //         _ => false,
                //     }) {
                //         break;
                //     }
                // }
            },
            Ast::For(id, range, body) => {
                assert!(range.len() == 2);
                let mut localScope = retScope.clone();

                let mut rangeBegin = 0;
                let mut rangeEnd = 0;

                let rangeBeginAst = Interpreter::evaluate(
                    Box::new(range[0].clone()),
                    retScope.clone(),
                    retFunctionScope.clone(),
                    retStructScope.clone(),
                );
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

                let rangeEndAst = Interpreter::evaluate(
                    Box::new(range[1].clone()),
                    retScope.clone(),
                    retFunctionScope.clone(),
                    retStructScope.clone(),
                );
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
                    Interpreter::run(
                        body.clone(),
                        localScope.clone(),
                        retFunctionScope.clone(),
                        retStructScope.clone(),
                    );

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
                let conditionEvaluated = Interpreter::evaluate(
                    condition,
                    retScope.clone(),
                    retFunctionScope.clone(),
                    retStructScope.clone(),
                );
                if matches!(
                    conditionEvaluated,
                    Ast::Literal(Literal {
                        content: TokenContentType::Boolean(false)
                    })
                ) {
                    for statement in elseBody {
                        Interpreter::execute(
                            statement,
                            retScope.clone(),
                            retFunctionScope.clone(),
                            retStructScope.clone(),
                        );
                    }
                } else {
                    Interpreter::run(
                        ifBody,
                        retScope.clone(),
                        retFunctionScope.clone(),
                        retStructScope.clone(),
                    );
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
                Interpreter::evaluate(
                    Box::new(node),
                    retScope.clone(),
                    retFunctionScope.clone(),
                    retStructScope.clone(),
                );
            }
        }
        (retScope, retValue)
    }
}
