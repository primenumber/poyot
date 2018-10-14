use super::generate::Program;
use super::generate::Function;
use super::generate::Value;
use super::parse::Operator;

use std::io::{BufWriter, Write};

fn pullup<W: Write>(regs: &mut Vec<usize>, reg: usize, writer: &mut BufWriter<W>) {
    let mut index = 0;
    for (i, r) in regs.iter().enumerate() {
        if *r == reg {
            index = i;
            break;
        }
    }
    let regcount = regs.len();
    write!(writer, "PUSH {}\n", regcount - index);
    write!(writer, "PUSH -1\n");
    write!(writer, "ROLL\n");
    write!(writer, "DUP\n");
    write!(writer, "PUSH {}\n", regcount - index + 1);
    write!(writer, "PUSH 1\n");
    write!(writer, "ROLL\n");
}

fn substitute<W: Write>(val: &Value, ret: usize, regs: &mut Vec<usize>, writer: &mut BufWriter<W>) {
    match val {
        Value::Register(reg) => {
            pullup(regs, *reg, writer);
        }
        Value::Immediate(imm) => {
            write!(writer, "PUSH {}\n", imm);
        }
        _ => {}
    }
    regs.push(ret);
}

fn bin_op<W: Write>(args: &[Value], ret: usize, regs: &mut Vec<usize>, writer: &mut BufWriter<W>) {
    substitute(&args[0], 0, regs, writer);
    substitute(&args[1], 0, regs, writer);
    regs.pop();
    regs.pop();
    regs.push(ret);
}

fn function<W: Write>(func: &Function, start: usize, program: &Program, writer: &mut BufWriter<W>) -> Option<usize> {
    write!(writer, "LABEL func_{}\n", func.name);
    let mut count = 0;
    match &*func.name {
        "getnum" => {
            write!(writer, "INN\n");
            write!(writer, "SWAP\n");
            write!(writer, "JMP return\n");
        }
        "getchar" => {
            write!(writer, "INC\n");
            write!(writer, "SWAP\n");
            write!(writer, "JMP return\n");
        }
        "putnum" => {
            write!(writer, "OUTN\n");
            write!(writer, "JMP return\n");
        }
        "putchar" => {
            write!(writer, "OUTC\n");
            write!(writer, "JMP return\n");
        }
        "halt" => {
            write!(writer, "HALT\n");
        }
        _ => {
            eprintln!("{}", func.name);
            let mut regs = Vec::new();
            for i in 0..func.args.len() {
                regs.push(i);
            }
            for (i, block) in func.basicblocks.iter().enumerate() {
                let regs_old = regs.clone();
                write!(writer, "LABEL block_{}_{}\n", func.name, i);
                for inst in &block.statements {
                    eprintln!("{:?}", regs);
                    match inst.op {
                        Operator::Call{ref name} => {
                            write!(writer, "PUSH {}\n", start + count);
                            regs.push(0);
                            for arg in &inst.args {
                                substitute(arg, 0, &mut regs, writer);
                            }
                            write!(writer, "JMP func_{}\n", name);
                            write!(writer, "LABEL control_{}\n", start + count);
                            write!(writer, "POP\n");
                            regs.pop();
                            for _ in &inst.args {
                                regs.pop();
                            }
                            match program.funcs.get(name) {
                                Some(callee) => {
                                    for i in 0..callee.retnum {
                                        regs.push(inst.ret.unwrap())
                                    }
                                }
                                None => return None
                            }
                            count += 1;
                        }
                        Operator::If => {
                            substitute(&inst.args[0], 0, &mut regs, writer);
                            for i in func.args.len()..regs.len()-1 {
                                write!(writer, "SWAP\n");
                                write!(writer, "POP\n");
                            }
                            let jump_to = block.nexts.get(1).unwrap();
                            write!(writer, "JEZ block_{}_{}\n", func.name, jump_to);
                            regs.pop();
                        }
                        Operator::Jump => {
                            let jump_to = block.nexts.get(0).unwrap();
                            write!(writer, "JMP block_{}_{}\n", func.name, jump_to);
                        }
                        Operator::Return => {
                            substitute(&inst.args[0], 0, &mut regs, writer);
                            for i in 1..regs.len() {
                                write!(writer, "SWAP\n");
                                write!(writer, "POP\n");
                            }
                            write!(writer, "SWAP\n");
                            write!(writer, "JMP return\n");
                        }
                        Operator::Substitute => {
                            substitute(&inst.args[0], inst.ret.unwrap(), &mut regs, writer);
                        }
                        Operator::Add => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "ADD\n");
                        }
                        Operator::Sub => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "SUB\n");
                        }
                        Operator::Multiply => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "MUL\n");
                        }
                        Operator::Division => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "DIV\n");
                        }
                        Operator::Modulo => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "MOD\n");
                        }
                        Operator::LessThan => {
                            substitute(&inst.args[1], 0, &mut regs, writer);
                            substitute(&inst.args[0], 0, &mut regs, writer);
                            regs.pop();
                            regs.pop();
                            regs.push(inst.ret.unwrap());
                            write!(writer, "GREATER\n");
                        }
                        Operator::Greater => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "GREATER\n");
                        }
                        Operator::Equal => {
                            bin_op(&inst.args, inst.ret.unwrap(), &mut regs, writer);
                            write!(writer, "SUB\n");
                            write!(writer, "NOT\n");
                        }
                        _ => {
                            println!("Unsupported operator: {:?}", inst.op);
                            return None;
                        }
                    }
                }
                regs = regs_old;
            }
            if func.name == "main" {
                write!(writer, "HALT\n");
            }
        }
    }
    Some(count)
}

pub fn trans<W: Write>(program: &Program, writer: &mut BufWriter<W>) -> Option<()>{
    write!(writer, "JMP func_main\n");
    let mut start = 0;
    for (name, func) in program.funcs.iter() {
        match function(func, start, program, writer) {
            Some(count) => {
                start += count;
            }
            None => return None
        }
    }
    write!(writer, "LABEL return\n");
    for i in 0..start {
        write!(writer, "DUP\n");
        write!(writer, "JEZ control_{}\n", i);
        write!(writer, "PUSH 1\n");
        write!(writer, "SUB\n");
    }
    Some(())
}
