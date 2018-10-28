use super::parse::AST;
use super::parse::Node;
use super::parse::Operator;
use super::parse::Leaf;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Register(usize),
    Immediate(i32),
    Label(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub op: Operator,
    pub ret: Option<usize>,
    pub args: Vec<Value>
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub nexts: Vec<usize>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub retnum: usize,
    pub basicblocks: Vec<BasicBlock>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub funcs: HashMap<String, Function>
}

fn expression(ast: &AST, program: &Program,
              vars: &HashMap<String, usize>, statements: &mut Vec<Statement>,
              regcount: usize)
        -> Option<Value> {
    match ast {
        AST::Node(node) => {
            match node.op {
                Operator::Call{name:ref funcname} => {
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
                Operator::Add => {
                    if node.children.len() != 2 {
                        println!("Add operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::Sub => {
                    if node.children.len() != 2 {
                        println!("Sub operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::Multiply => {
                    if node.children.len() != 2 {
                        println!("Multiply operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::Division => {
                    if node.children.len() != 2 {
                        println!("Division operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::Modulo => {
                    if node.children.len() != 2 {
                        println!("Modulo operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::LessThan => {
                    if node.children.len() != 2 {
                        println!("LessThan operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::Greater => {
                    if node.children.len() != 2 {
                        println!("Greater operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                Operator::Equal => {
                    if node.children.len() != 2 {
                        println!("Equal operation take 2 args, but {} provided",
                                 node.children.len());
                        return None;
                    }
                }
                _ => {
                    println!("Unsupported operation {:?}", node.op);
                    return None
                }
            }
            let mut id_vec = Vec::new();
            for child in &node.children {
                let id = expression(&child, program, vars, statements, regcount)?;
                id_vec.push(id);
            }
            let id = statements.len() + regcount;
            statements.push(Statement {
                op: node.op.clone(),
                ret: Some(id),
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
              vars: &mut HashMap<String, usize>,
              statements: &mut Vec<Statement>, regcount: usize) 
        -> Option<Value> {
    match &children[0] {
        AST::Leaf(leaf) => {
            match leaf {
                Leaf::Identifier(lhs) => {
                    if vars.contains_key(lhs) {
                        println!("Variable {} is already defined.", lhs);
                        return None;
                    }
                    let exp_id = expression(&children[1], program, vars,
                                            statements, regcount)?;
                    let id = statements.len() + regcount;
                    statements.push(Statement {
                        op: Operator::Substitute,
                        ret: Some(id),
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
        vars: &HashMap<String, usize>, statements: &mut Vec<Statement>,
        regcount: usize)
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
                let id = expression(child, program, vars, statements,
                                    regcount)?;
                id_vec.push(id);
            }
            let id = statements.len();
            statements.push(Statement {
                op: Operator::Call{name:name.to_string()},
                ret: Some(id),
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

fn if_op(children: &Vec<AST>, program: &Program,
         vars: &mut HashMap<String, usize>, basicblocks: &mut Vec<BasicBlock>,
         regcount: usize) -> bool {
    if children.len() < 2 {
        println!("If need 2 or 3 children");
        return false;
    }
    let id = match expression(&children[0], program, vars,
                              &mut basicblocks.last_mut().unwrap().statements,
                              regcount) {
        Some(id) => id,
        None => {
            println!("Invalid expression");
            return false
        }
    };
    let newregcount = regcount + basicblocks.last().unwrap().statements.len();
    let index = match statement(&children[1], program, vars, newregcount) {
        Some(vb) => {
            basicblocks.last_mut().unwrap().statements.push(Statement {
                op: Operator::If,
                ret: None,
                args: vec![id]
            });
            let offset = basicblocks.len();
            basicblocks.last_mut().unwrap().nexts.push(offset);
            for b in vb {
                let mut nexts = Vec::new();
                for bid in b.nexts {
                    nexts.push(bid + offset);
                }
                basicblocks.push(BasicBlock {
                    statements: b.statements,
                    nexts
                });
            }
            let jump_to = basicblocks.len();
            basicblocks.get_mut(offset-1).unwrap().nexts.push(jump_to);
            basicblocks.len() - 1
        }
        None => {
            println!("Invalid statements");
            println!("{:?}", children[1]);
            return false
        }
    };
    if children.len() == 3 {
        match statement(&children[2], program, vars, newregcount) {
            Some(vb) => {
                basicblocks.last_mut().unwrap().statements.push(Statement {
                    op: Operator::Jump,
                    ret: None,
                    args: Vec::new()
                });
                let offset = basicblocks.len();
                for b in vb {
                    let mut nexts = Vec::new();
                    for bid in b.nexts {
                        nexts.push(bid + offset);
                    }
                    basicblocks.push(BasicBlock {
                        statements: b.statements,
                        nexts
                    });
                }
                let jump_to = basicblocks.len();
                basicblocks.get_mut(offset-1).unwrap().nexts.push(jump_to);
            }
            None => {
                println!("Invalid statements");
                return false
            }
        }
    }
    let jump_to = basicblocks.len();
    basicblocks.last_mut().unwrap().nexts.push(jump_to);
    basicblocks.push(BasicBlock {
        statements: Vec::new(),
        nexts: Vec::new()
    });
    true
}

fn return_op(children: &Vec<AST>, program: &Program,
             vars: &mut HashMap<String, usize>, statements: &mut Vec<Statement>,
             regcount: usize) -> bool {
    let mut vec_id = Vec::new();
    for child in children {
        let id = match expression(child, program, vars,
                                  statements, regcount) {
            Some(id) => id,
            None => {
                println!("Invalid expression");
                return false
            }
        };
        vec_id.push(id);
    }
    statements.push(Statement {
        op: Operator::Return,
        ret: None,
        args: vec_id
    });
    true
}


fn statement_impl(ast: &AST, program: &Program,
                  vars: &mut HashMap<String, usize>,
                  basicblocks: &mut Vec<BasicBlock>, regcount: usize)
        -> bool {
    match ast {
        AST::Node(node) => {
            match node.op {
                Operator::Substitute => {
                    substitute(&node.children, program, vars,
                               &mut basicblocks.last_mut().unwrap().statements,
                               regcount).is_some()
                }
                Operator::Call{ref name} => {
                    call(&name, &node.children, program, vars,
                         &mut basicblocks.last_mut().unwrap().statements,
                         regcount).is_some()
                }
                Operator::If => {
                    if_op(&node.children, program, vars, basicblocks, regcount)
                }
                Operator::Return => {
                    return_op(&node.children, program, vars,
                              &mut basicblocks.last_mut().unwrap().statements,
                              regcount)
                }
                _ => {
                    println!("Unknwon operator: {:?}", node.op);
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

fn statement(ast: &AST, program: &Program,
             vars: &mut HashMap<String, usize>, regcount: usize)
        -> Option<Vec<BasicBlock>> {
    let mut basicblocks = vec![BasicBlock {
        statements: Vec::new(),
        nexts: Vec::new()
    }];
    match ast {
        AST::Node(node) => {
            match node.op {
                Operator::Statement => {
                    for child in &node.children {
                        if !statement_impl(&child, program, vars, &mut basicblocks, regcount) {
                            return None;
                        }
                    }
                    Some(basicblocks)
                }
                _ => {
                    println!("Unexpected operator: {:?}, expected Statement", node.op);
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

fn pre_declare_function(node: &Node, program: &Program) -> Option<Function> {
    match node.op {
        Operator::FunctionDeclare{ref name, ref args, retnum} => {
            Some(Function { name: name.to_string(), args: args.to_vec(), retnum, basicblocks: Vec::new() })
        }
        _ => {
            println!("Unexpected operator: {:?}, expected FunctionDeclare", node.op);
            None
        }
    }
}

fn function(node: &Node, program: &Program) -> Option<Function> {
    match node.op {
        Operator::FunctionDeclare{ref name, ref args, retnum} => {
            let mut vars = HashMap::<String, usize>::new();
            for (i, arg) in args.iter().enumerate() {
                vars.insert(arg.to_string(), i);
            }
            let regcount = args.len();
            match statement(&node.children[0], program, &mut vars, regcount) {
                Some(mut basicblocks) => {
                    if retnum == 0 && name != "main" {
                        basicblocks.last_mut().unwrap().statements.push(Statement {
                            op: Operator::Return,
                            ret: None,
                            args: Vec::new()
                        });
                    }
                    Some(Function { name: name.to_string(), args: args.to_vec(), retnum, basicblocks })
                }
                None => {
                    println!("Invalid statements");
                    None
                }
            }
        }
        _ => {
            println!("Unexpected operator: {:?}, expected FunctionDeclare", node.op);
            None
        }
    }
}

fn pre_declare(ast: &AST, program: &mut Program) -> bool {
    match ast {
        AST::Node(node) => {
            match pre_declare_function(node, program) {
                Some(func) => {
                    program.funcs.insert(func.name.clone(), func);
                    true
                }
                None => {
                    println!("Invalid function declare");
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

fn declare(ast: &AST, program: &mut Program) -> bool {
    match ast {
        AST::Node(node) => {
            match function(node, program) {
                Some(func) => {
                    program.funcs.insert(func.name.clone(), func);
                    true
                }
                None => {
                    println!("Invalid function declare");
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

impl Program {
    fn new() -> Program {
        let getnum = Function {
            name: "getnum".to_string(),
            args: Vec::new(),
            retnum: 1,
            basicblocks: Vec::new()
        };
        let getchar = Function {
            name: "getchar".to_string(),
            args: Vec::new(),
            retnum: 1,
            basicblocks: Vec::new()
        };
        let putnum = Function {
            name: "putnum".to_string(),
            args: vec!["x".to_string()],
            retnum: 0,
            basicblocks: Vec::new()
        };
        let putchar = Function {
            name: "putchar".to_string(),
            args: vec!["x".to_string()],
            retnum: 0,
            basicblocks: Vec::new()
        };
        let halt = Function {
            name: "halt".to_string(),
            args: Vec::new(),
            retnum: 0,
            basicblocks: Vec::new()
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
                Operator::Declare => {
                    for child in &node.children {
                        if !pre_declare(child, &mut program) {
                            return None;
                        }
                    }
                    for child in &node.children {
                        if !declare(child, &mut program) {
                            return None;
                        }
                    }
                    Some(program)
                }
                _ => {
                    println!("Unexpected operator {:?}, expected Declare", node.op);
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

