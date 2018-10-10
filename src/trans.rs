use super::generate::Program;
use super::generate::Function;
use super::generate::Value;
use super::parse::Operand;

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

fn function<W: Write>(func: &Function, start: usize, writer: &mut BufWriter<W>) -> Option<usize> {
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
                    Operand::Call{ref name} => {
                        write!(writer, "PUSH {}\n", start + count);
                        for arg in &inst.args {
                            substitute(arg, regcount+1, writer);
                            regcount += 1;
                        }
                        write!(writer, "JMP func_{}\n", name);
                        write!(writer, "LABEL control_{}\n", start + count);
                        write!(writer, "POP\n");
                        regcount += 1;
                        count += 1;
                    }
                    Operand::Substitute => {
                        substitute(&inst.args[0], regcount, writer);
                        regcount += 1;
                    }
                    Operand::Add => {
                        substitute(&inst.args[0], regcount, writer);
                        regcount += 1;
                        substitute(&inst.args[1], regcount, writer);
                        regcount += 1;
                        write!(writer, "ADD\n");
                    }
                    _ => {
                        println!("Unsupported operand: {:?}", inst.op);
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
        match function(func, start, writer) {
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
