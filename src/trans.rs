use super::generate::Program;
use super::generate::Function;
use super::generate::Value;
use super::parse::Operator;

use std::io::{BufWriter, Write};

fn pullup<W: Write>(regcount: usize, reg: usize, writer: &mut BufWriter<W>) {
    write!(writer, "PUSH {}\n", regcount - reg);
    write!(writer, "PUSH -1\n");
    write!(writer, "ROLL\n");
    write!(writer, "DUP\n");
    write!(writer, "PUSH {}\n", regcount - reg + 1);
    write!(writer, "PUSH 1\n");
    write!(writer, "ROLL\n");
}

fn substitute<W: Write>(val: &Value, regcount: usize, writer: &mut BufWriter<W>) {
    match val {
        Value::Register(reg) => {
            pullup(regcount, *reg, writer);
        }
        Value::Immediate(imm) => {
            write!(writer, "PUSH {}\n", imm);
        }
    }
}

fn bin_op<W: Write>(args: &[Value], regcount: &mut usize, writer: &mut BufWriter<W>) {
    substitute(&args[0], *regcount, writer);
    *regcount += 1;
    substitute(&args[1], *regcount, writer);
    *regcount += 1;
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
            let mut regcount = func.args.len();
            for inst in &func.statements {
                match inst.op {
                    Operator::Call{ref name} => {
                        write!(writer, "PUSH {}\n", start + count);
                        let mut regcount_tmp = regcount;
                        for arg in &inst.args {
                            substitute(arg, regcount_tmp+1, writer);
                            regcount_tmp += 1;
                        }
                        write!(writer, "JMP func_{}\n", name);
                        write!(writer, "LABEL control_{}\n", start + count);
                        write!(writer, "POP\n");
                        match program.funcs.get(name) {
                            Some(callee) => {
                                regcount += callee.retnum;
                            }
                            None => return None
                        }
                        count += 1;
                    }
                    Operator::Substitute => {
                        substitute(&inst.args[0], regcount, writer);
                        regcount += 1;
                    }
                    Operator::Add => {
                        bin_op(&inst.args, &mut regcount, writer);
                        write!(writer, "ADD\n");
                        regcount -= 1;
                    }
                    Operator::Sub => {
                        bin_op(&inst.args, &mut regcount, writer);
                        write!(writer, "SUB\n");
                        regcount -= 1;
                    }
                    Operator::Multiply => {
                        bin_op(&inst.args, &mut regcount, writer);
                        write!(writer, "MUL\n");
                        regcount -= 1;
                    }
                    Operator::Division => {
                        bin_op(&inst.args, &mut regcount, writer);
                        write!(writer, "DIV\n");
                        regcount -= 1;
                    }
                    Operator::Modulo => {
                        bin_op(&inst.args, &mut regcount, writer);
                        write!(writer, "MOD\n");
                        regcount -= 1;
                    }
                    _ => {
                        println!("Unsupported operator: {:?}", inst.op);
                        return None;
                    }
                }
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
