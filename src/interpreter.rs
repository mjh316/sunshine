use std::{
    borrow::Borrow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    string,
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
    pub fn toPrint(
        ast: Ast,
        scope: Scope,
        functionScope: FunctionScope,
        structScope: StructScope,
    ) -> String {
        match ast {
            Ast::Literal(literal) => match literal.content {
                TokenContentType::String(s) => s,
                TokenContentType::Number(n) => n.to_string(),
                TokenContentType::Boolean(b) => b.to_string(),
            },
            Ast::Var(name, _) => {
                if !Interpreter::inScope(scope.clone(), name.clone())
                    && !Interpreter::isFuncInScope(functionScope.clone(), name.clone())
                {
                    panic!("Variable {} not found in scope", name);
                }
                if Interpreter::inScope(scope.clone(), name.clone()) {
                    return Interpreter::toPrint(
                        scope.borrow_mut().get(&name).unwrap().clone(),
                        scope.clone(),
                        functionScope.clone(),
                        structScope.clone(),
                    );
                } else if Interpreter::isFuncInScope(functionScope.clone(), name.clone()) {
                    return format!("function {}", name);
                } else {
                    panic!("Variable {} not found in scope", name);
                }
            }
            Ast::Array(array) => {
                let mut result = "[".to_string();
                for (i, item) in array.content.iter().enumerate() {
                    result.push_str(&Interpreter::toPrint(
                        item.clone(),
                        scope.clone(),
                        functionScope.clone(),
                        structScope.clone(),
                    ));
                    if i != array.content.len() - 1 {
                        result.push_str(", ");
                    }
                }
                result.push_str("]");
                return result;
            }
            Ast::Instance(name, members) => {
                let mut result = format!("{} {{", name);
                for (i, (field, value)) in members.iter().enumerate() {
                    result.push_str(&format!(
                        "{}: {}",
                        field,
                        Interpreter::toPrint(
                            value.clone(),
                            scope.clone(),
                            functionScope.clone(),
                            structScope.clone(),
                        )
                    ));
                    if i != members.len() - 1 {
                        result.push_str(", ");
                    }
                }
                result.push_str("}");
                return result;
            }
            _ => {
                panic!("Expected expression but got statement {:?}", ast);
            }
        }
    }

    pub fn run(
        ast: Vec<Ast>,
        scope: Scope,
        functionScope: FunctionScope,
        structScope: StructScope,
    ) -> (Scope, Option<Ast>) {
        let mut retScope = scope.clone();
        for node in ast {
            // println!("running node {:?}", node);
            // println!("calling execute from run");
            let (newScope, _ret) = Interpreter::execute(
                node,
                retScope.clone(),
                functionScope.clone(),
                structScope.clone(),
            );
            // println!("new scope: {:?}", newScope.clone());
            // println!("ret: {:?}", _ret.clone());
            retScope = newScope;
            match _ret {
                Some(ret) => match ret {
                    Ast::Return(value) => {
                        return (retScope, Some(*value));
                    }
                    _ => {}
                },
                None => {}
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
        // println!("evaluating {:?}", value);
        match *value {
            Ast::Var(name, _) => {
                if !Interpreter::inScope(scope.clone(), name.clone())
                    && !Interpreter::isFuncInScope(functionScope.clone(), name.clone())
                {
                    panic!("Variable {} not found in scope", name);
                }
                if Interpreter::inScope(scope.clone(), name.clone()) {
                    return scope.borrow_mut().get(&name).unwrap().clone();
                } else if Interpreter::isFuncInScope(functionScope.clone(), name.clone()) {
                    // println!("found function {}", name);
                    return Ast::Func(name, vec![], vec![]);
                } else {
                    panic!("Variable {} not found in scope", name);
                }
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

                // println!("left: {:?}, op: {:?}, right: {:?}", left, op, right);

                if !operations.contains_key(&op) {
                    panic!("Unknown binary operator {:?}", op);
                }

                let resultOfOp = operations.get(&op).unwrap()(left, right);
                // println!("result of op: {:?}", resultOfOp);

                return resultOfOp;
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

                // println!("function caller: {:?}", caller);

                match caller {
                    Ast::Func(name, _, _) => {
                        if !Interpreter::isFuncInScope(functionScope.clone(), name.clone()) {
                            panic!("Function {} not found in scope", name);
                        } else {
                            // println!("found function {}", name);
                            let mut functionScopeMap = functionScope.borrow_mut();
                            let function = functionScopeMap
                                .get_mut(&name)
                                .expect(format!("Function {} not found in scope", name).as_str());
                            return function(args);
                        }
                    }
                    _ => {
                        panic!("Expected function but got {:?}", caller);
                    }
                }
            }
            Ast::Get(caller_ast, property_ast, is_expr) => {
                // Evaluate the caller
                let caller = Interpreter::evaluate(
                    caller_ast,
                    scope.clone(),
                    functionScope.clone(),
                    structScope.clone(),
                );

                // Determine the property
                let property = if is_expr {
                    Interpreter::evaluate(
                        property_ast,
                        scope.clone(),
                        functionScope.clone(),
                        structScope.clone(),
                    )
                } else {
                    *property_ast
                };

                match caller {
                    Ast::Array(array) => {
                        if !matches!(
                            property,
                            Ast::Literal(Literal {
                                content: TokenContentType::Number(_)
                            })
                        ) {
                            panic!("Expected number as index, got {:?}", property);
                        }
                        if let Ast::Literal(Literal {
                            content: TokenContentType::Number(n),
                        }) = property
                        {
                            return array.content.get(n as usize).unwrap().clone();
                        } else {
                            panic!("Expected number as index, got {:?}", property);
                        }
                    }
                    Ast::Instance(name, members) => {
                        let propertyKey = match property {
                            Ast::Literal(Literal {
                                content: TokenContentType::String(s),
                            }) => s,
                            _ => panic!("Expected string as property, got {:?}", property),
                        };

                        if !members.contains_key(&propertyKey.to_string()) {
                            panic!("Property {} not found in instance {}", propertyKey, name);
                        }
                        return members.get(&propertyKey).unwrap().clone(); // WILL be a value or a function or SOMETHING
                    }
                    Ast::Var(name, _value) => {
                        if !Interpreter::inScope(scope.clone(), name.clone()) {
                            panic!("Variable {} not found in scope", name);
                        }
                        let value = scope.borrow_mut().get(&name).unwrap().clone();
                        match value {
                            Ast::Instance(_, _) => {
                                let instance = value;
                                let propertyKey = match property {
                                    Ast::Literal(Literal {
                                        content: TokenContentType::String(s),
                                    }) => s,
                                    _ => panic!("Expected string as property, got {:?}", property),
                                };

                                if !matches!(instance, Ast::Instance(_, _)) {
                                    panic!("Expected instance but got {:?}", instance);
                                }

                                if let Ast::Instance(_, members) = instance {
                                    if !members.contains_key(&propertyKey.to_string()) {
                                        panic!(
                                            "Property {} not found in instance {}",
                                            propertyKey, name
                                        );
                                    }
                                    return members.get(&propertyKey).unwrap().clone();
                                // WILL be a value or a function or SOMETHING
                                } else {
                                    panic!("Expected instance but got {:?}", instance);
                                }
                            }
                            _ => {
                                panic!("Expected instance but got {:?}", value);
                            }
                        }
                    }
                    _ => {
                        panic!("Expected instance but got {:?}", caller);
                    }
                }
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
        // println!("executing {:?}", node);
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
                let functionScope = Rc::clone(&retFunctionScope);
                let valueScope = Rc::clone(&retScope);
                let structureScope = Rc::clone(&retStructScope);
                let functionBody = body.clone();
                let function = Box::new(move |args: Vec<Ast>| {
                    let mut localScope = valueScope.clone();
                    for (i, param) in params.iter().enumerate() {
                        localScope
                            .borrow_mut()
                            .insert(param.clone(), args.get(i).unwrap().clone());
                    }

                    let functionScope = Rc::clone(&functionScope);

                    let result = Interpreter::run(
                        functionBody.clone(),
                        localScope,
                        functionScope,
                        structureScope.clone(),
                    );

                    match result.1 {
                        Some(value) => value,
                        None => Ast::None,
                    }
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
                // println!("STARTING WHILE LOOP!");
                loop {
                    // println!("condition {:?}", condition);
                    // println!("calling execute from while loop");
                    if !matches!(
                        Interpreter::evaluate(
                            condition.clone(),
                            retScope.clone(),
                            retFunctionScope.clone(),
                            retStructScope.clone(),
                        ),
                        Ast::Literal(Literal {
                            content: TokenContentType::Boolean(true)
                        })
                    ) {
                        // println!("BREAKING WHILE LOOP!");
                        break;
                    }

                    // println!("{:?}", format!("condition {:?}", condition));
                    let result = Interpreter::execute(
                        *condition.clone(),
                        retScope.clone(),
                        retFunctionScope.clone(),
                        retStructScope.clone(),
                    );
                    // println!("result in while {:?}", result);
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
                return (retScope, retValue);
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
                        println!("calling execute from conditional");
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
            Ast::Set(caller, _, value) => {
                if !Interpreter::inScope(retScope.clone(), caller.clone()) {
                    panic!("Instance {} not found in scope", caller);
                }

                let mut instance = retScope.borrow_mut().get(&caller).unwrap().clone();
                if let Ast::Instance(name, mut members) = instance {
                    let value = Interpreter::evaluate(
                        value,
                        retScope.clone(),
                        retFunctionScope.clone(),
                        retStructScope.clone(),
                    );

                    let propertyKey = match value.clone() {
                        Ast::Literal(Literal {
                            content: TokenContentType::String(s),
                        }) => s,
                        _ => panic!("Expected string as property, got {:?}", value),
                    };

                    if !members.contains_key(&propertyKey.to_string()) {
                        panic!("Property {} not found in instance {}", propertyKey, name);
                    }
                    members.insert(propertyKey, value);
                    retScope
                        .borrow_mut()
                        .insert(name.clone(), Ast::Instance(name, members));
                    return (retScope, None);
                } else {
                    panic!("Expected instance but got {:?}", instance);
                }
            }

            _ => {
                return (
                    retScope.clone(),
                    Some(Interpreter::evaluate(
                        Box::new(node),
                        retScope.clone(),
                        retFunctionScope.clone(),
                        retStructScope.clone(),
                    )),
                );
            }
        }
        (retScope, retValue)
    }
}
