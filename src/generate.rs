use super::parse::AST;
use super::parse::Node;
use super::parse::Operand;
use super::parse::Leaf;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Register(usize),
    Immediate(i32)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub op: Operand,
    pub ret: usize,
    pub args: Vec<Value>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub retnum: usize,
    pub statements: Vec<Statement>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub funcs: HashMap<String, Function>
}

fn expression(ast: &AST, program: &Program,
              vars: &HashMap<String, usize>, statements: &mut Vec<Statement>)
        -> Option<Value> {
    match ast {
        AST::Node(node) => {
            match node.op {
                Operand::Call{name:ref funcname} => {
                    match program.funcs.get(funcname) {
                        Some(func) => {
                            if node.children.len() != func.args.len() {
                                println!("Function {} expected {} args, but {} provided",
                                         funcname, func.args.len(), node.children.len());
                                return None;
                            }
                        }
                        None => {
                            println!("Undefined function {}", funcname);
                        }
                    }
                }
                Operand::Add => {
                    if node.children.len() != 2 {
                        println!("Add operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                _ => return None
            }
            let mut id_vec = Vec::new();
            for child in &node.children {
                let id = expression(&child, program, vars, statements)?;
                id_vec.push(id);
            }
            let id = statements.len();
            statements.push(Statement {
                op: node.op.clone(),
                ret: id,
                args: id_vec
            });
            Some(Value::Register(id))
        }
        AST::Leaf(leaf) => {
            match leaf {
                Leaf::Identifier(name) => {
                    match vars.get(name) {
                        Some(id) => {
                            Some(Value::Register(*id))
                        }
                        None => {
                            println!("Undefined variable {}", name);
                            None
                        }
                    }
                }
                Leaf::Constant(imm) => {
                    Some(Value::Immediate(*imm))
                }
            }
        }
    }
}

fn substitute(children: &Vec<AST>, program: &Program,
              vars: &mut HashMap<String, usize>, statements: &mut Vec<Statement>) 
        -> Option<Value> {
    match &children[0] {
        AST::Leaf(leaf) => {
            match leaf {
                Leaf::Identifier(lhs) => {
                    if vars.contains_key(lhs) {
                        println!("Variable {} is already defined.", lhs);
                        return None;
                    }
                    let exp_id = expression(&children[1], program, vars, statements)?;
                    let id = statements.len();
                    statements.push(Statement {
                        op: Operand::Substitute,
                        ret: id,
                        args: vec![exp_id; 1]
                    });
                    vars.insert(lhs.to_string(), id);
                    Some(Value::Register(id))
                }
                Leaf::Constant(constant) => {
                    println!("Unexpected constant {}, expected identifier", constant);
                    None
                }
            }
        }
        AST::Node(_node) => {
            println!("Unexpected Node, expected identifier or constant");
            None
        }
    }
}

fn call(name: &str, children: &Vec<AST>, program: &Program,
        vars: &HashMap<String, usize>, statements: &mut Vec<Statement>)
        -> Option<Value> {
    match program.funcs.get(name) {
        Some(func) => {
            if func.args.len() != children.len() {
                println!("Function {}: expected {} args, but {} provided",
                         name, func.args.len(), children.len());
                return None;
            }
            let mut id_vec = Vec::new();
            for child in children {
                let id = expression(child, program, vars, statements)?;
                id_vec.push(id);
            }
            let id = statements.len();
            statements.push(Statement {
                op: Operand::Call{name:name.to_string()},
                ret: id,
                args: id_vec
            });
            Some(Value::Register(id))
        }
        None => {
            println!("Function {} is not defined.", name);
            None
        }
    }
}

fn statement_impl(ast: &AST, program: &Program,
                  vars: &mut HashMap<String, usize>, statements: &mut Vec<Statement>)
        -> bool {
    match ast {
        AST::Node(node) => {
            match node.op {
                Operand::Substitute => {
                    substitute(&node.children, program, vars, statements).is_some()
                }
                Operand::Call{ref name} => {
                    call(&name, &node.children, program, vars, statements).is_some()
                }
                _ => {
                    println!("Unknwon operand: {:?}", node.op);
                    false
                }
            }
        }
        AST::Leaf(_leaf) => {
            println!("Invalid identifier or constant");
            false
        }
    }
}

fn statement(ast: &AST, program: &Program, vars: &mut HashMap<String, usize>)
        -> Option<Vec<Statement>> {
    let mut statement_vec = Vec::<Statement>::new();
    match ast {
        AST::Node(node) => {
            match node.op {
                Operand::Statement => {
                    for child in &node.children {
                        if !statement_impl(&child, program, vars, &mut statement_vec) {
                            return None;
                        }
                    }
                    Some(statement_vec)
                }
                _ => {
                    println!("Unexpected operand: {:?}, expected Statement", node.op);
                    None
                }
            }
        }
        AST::Leaf(_leaf) => {
            println!("Invalid identifier or constant");
            None
        }
    }
}

fn function(node: &Node, program: &Program) -> Option<Function> {
    match node.op {
        Operand::FunctionDeclare{ref name, ref args, retnum} => {
            let mut vars = HashMap::<String, usize>::new();
            for (i, arg) in args.iter().enumerate() {
                vars.insert(arg.to_string(), i);
            }
            match statement(&node.children[0], program, &mut vars) {
                Some(statements) => Some(Function { name: name.to_string(), args: args.to_vec(), retnum, statements }),
                None => None
            }
        }
        _ => {
            println!("Unexpected operand: {:?}, expected FunctionDeclare", node.op);
            None
        }
    }
}

fn declare(ast: &AST, program: &mut Program) -> bool {
    match ast {
        AST::Node(node) => {
            match function(node, program) {
                Some(func) => {
                    program.funcs.insert(func.name.clone(), func);
                    true
                }
                None => false
            }
        }
        AST::Leaf(_leaf) => {
            println!("Invalid identifier or constant");
            false
        }
    }
}

impl Program {
    fn new() -> Program {
        let getnum = Function {
            name: "getnum".to_string(),
            args: Vec::new(),
            retnum: 1,
            statements: Vec::new()
        };
        let getchar = Function {
            name: "getchar".to_string(),
            args: Vec::new(),
            retnum: 1,
            statements: Vec::new()
        };
        let putnum = Function {
            name: "putnum".to_string(),
            args: vec!["x".to_string()],
            retnum: 0,
            statements: Vec::new()
        };
        let putchar = Function {
            name: "putchar".to_string(),
            args: vec!["x".to_string()],
            retnum: 0,
            statements: Vec::new()
        };
        let halt = Function {
            name: "halt".to_string(),
            args: Vec::new(),
            retnum: 0,
            statements: Vec::new()
        };
        let mut funcs = HashMap::<String, Function>::new();
        funcs.insert(getnum.name.clone(), getnum);
        funcs.insert(getchar.name.clone(), getchar);
        funcs.insert(putnum.name.clone(), putnum);
        funcs.insert(putchar.name.clone(), putchar);
        funcs.insert(halt.name.clone(), halt);
        Program { funcs }
    }
}

pub fn generate(ast: &AST) -> Option<Program> {
    let mut program = Program::new();
    match ast {
        AST::Node(node) => {
            match node.op {
                Operand::Declare => {
                    for child in &node.children {
                        if !declare(child, &mut program) {
                            return None;
                        }
                    }
                    Some(program)
                }
                _ => {
                    println!("Unexpected operand {:?}, expected Declare", node.op);
                    None
                }
            }
        }
        AST::Leaf(_leaf) => {
            println!("Invalid identifier or constant");
            None
        }
    }
}

