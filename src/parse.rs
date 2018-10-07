use super::tokenize::Token;
use super::tokenize::Punctuator;
use super::tokenize::Keyword;

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Add,
    Sub,
    Multiply,
    Division,
    Modulo,
    Substitute,
    If,
    Call{name: String},
    Do,
    Expression,
    Statement,
    Declare,
    FunctionDeclare{name: String, args: Vec<String>}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    op: Operand,
    children: Vec<AST>
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Node(Node),
    Leaf(Token)
}

fn expression(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut itr = tokens.iter();
    match itr.next() {
        Some(Token::Constant(constant)) => Some((AST::Leaf(Token::Constant(*constant)), 1)),
        Some(Token::Identifier(identifier)) => Some((AST::Leaf(Token::Identifier(identifier.to_string())), 1)),
        _ => None
    }
}

fn expression_loop(tokens: &[Token], res: &mut Vec<AST>) -> Option<usize> {
    if tokens.len() >= 1 {
        if tokens[0] == Token::Punctuator(Punctuator::ParenthesisRight) {
            return Some(0);
        }
    }
    let itr = tokens.iter();
    let mut offset = 0;
    let expression_;
    let seek;
    match expression(tokens.get(offset..).unwrap()) {
        Some((exp, seek_in)) => {
            offset += seek_in;
            expression_ = exp;
            seek = seek_in;
        }
        None => return None
    }
    res.push(expression_);
    let mut itr2 = itr.skip(seek);
    match itr2.next() {
        Some(Token::Punctuator(Punctuator::Comma)) => {
            offset += 1;
            match expression_loop(tokens.get(offset..).unwrap(), res) {
                Some(seek) => Some(offset+seek),
                None => None
            }
        }
        Some(Token::Punctuator(Punctuator::ParenthesisRight)) => {
            Some(offset)
        }
        _ => return None
    }
}

fn expression_list(tokens: &[Token]) -> Option<(Vec<AST>, usize)> {
    let mut res = Vec::<AST>::new();
    match expression_loop(tokens, &mut res) {
        Some(seek) => Some((res, seek)),
        None => None
    }
}

fn statement(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut itr = tokens.iter();
    let left: Token;
    match itr.next() {
        Some(Token::Identifier(identifier)) => {
            left = Token::Identifier(identifier.to_string());
        }
        Some(other) => {
            println!("Unexpected {:?}, expected identifier", other);
            return None
        }
        _ => {
            println!("Unexpected EOF, expected identifier");
            return None
        }
    }
    match itr.next() {
        Some(Token::Punctuator(Punctuator::Equal)) => {
            let right: Token;
            match itr.next() {
                Some(Token::Identifier(identifier)) => {
                    right = Token::Identifier(identifier.to_string());
                }
                Some(Token::Constant(constant)) => {
                    right = Token::Constant(*constant);
                }
                Some(other) => {
                    println!("Unexpected {:?}, expected identifier or constant", other);
                    return None
                }
                _ => return None
            }
            match itr.next() {
                Some(Token::Punctuator(Punctuator::SemiColon)) => {}
                Some(other) => {
                    println!("Unexpected {:?}, expected ;", other);
                    return None
                }
                _ => return None
            }
            Some((AST::Node(Node {
                op: Operand::Substitute,
                children: vec![
                    AST::Leaf(left),
                    AST::Leaf(right)
                ]
            }), 4))
        }
        Some(Token::Punctuator(Punctuator::ParenthesisLeft)) => {
            match expression_list(tokens.get(2..).unwrap()) {
                Some((expressions, seek)) => {
                    let mut itr2 = itr.skip(seek);
                    match itr2.next() {
                        Some(Token::Punctuator(Punctuator::ParenthesisRight)) => {
                            match itr2.next() {
                                Some(Token::Punctuator(Punctuator::SemiColon)) => {
                                    match left {
                                        Token::Identifier(funcname) => {
                                            Some((AST::Node(Node {
                                                op: Operand::Call{name: funcname},
                                                children: expressions
                                            }), 2+seek+2))
                                        }
                                        _ => return None
                                    }
                                }
                                _ => return None
                            }
                        }
                        Some(other) => {
                            println!("Unexpected {:?}, expected )", other);
                            return None
                        }
                        _ => {
                            println!("Unexpected EOF, expected )");
                            return None
                        }
                    }
                }
                None => None
            }
        }
        Some(other) => {
            println!("Unexpected {:?}, expected = or (", other);
            return None
        }
        _ => {
            println!("Unexpected EOF, expected = or (");
            return None
        }
    }
}

fn statements_loop(tokens: &[Token], node: &mut Node) -> Option<usize> {
    if tokens.len() >= 1 {
        if tokens[0] == Token::Punctuator(Punctuator::BraceRight) {
            return Some(0);
        }
    }
    match statement(tokens) {
        Some((stm, seek)) => {
            node.children.push(stm);
            match statements_loop(tokens.get(seek..).unwrap(), node) {
                Some(len) => Some(seek+len),
                None => None
            }
        }
        None => None
    }
}

fn statement_list(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut res = Node {
        op: Operand::Statement,
        children: Vec::new()
    };
    match statements_loop(tokens, &mut res) {
        Some(seek) => {
            Some((AST::Node(res), seek))
        }
        None => None
    }
}

fn argument_list(tokens: &[Token]) -> Option<(Vec<String>, usize)> {
    let mut tokens_itr = tokens.iter();
    match tokens_itr.next() {
        Some(Token::Punctuator(Punctuator::ParenthesisLeft)) => {
            let mut res = Vec::<String>::new();
            let mut len = 1;
            loop {
                match tokens_itr.next() {
                    Some(Token::Identifier(identifier)) => {
                        res.push(identifier.to_string());
                    }
                    Some(Token::Punctuator(Punctuator::ParenthesisRight)) => {
                        return Some((res, len+1))
                    }
                    Some(other) => {
                        println!("Unexpected {:?}, expected identifier or )", other);
                        return None
                    }
                    _ => {
                        println!("Unexpected EOF, expected identifier or )");
                        return None
                    }
                }
                len += 1;
                match tokens_itr.next() {
                    Some(Token::Punctuator(Punctuator::Comma)) => {}
                    Some(Token::Punctuator(Punctuator::ParenthesisRight)) => {
                        return Some((res, len+1))
                    }
                    Some(other) => {
                        println!("Unexpected {:?}, expected , or )", other);
                        return None
                    }
                    _ => {
                        println!("Unexpected EOF, expected , or )");
                        return None
                    }
                }
                len += 1;
            }
        }
        _ => return None
    }
}

fn declaration(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut tokens_itr = tokens.iter();
    match tokens_itr.next() {
        Some(Token::Keyword(Keyword::FN)) => {
            match tokens_itr.next() {
                Some(Token::Punctuator(Punctuator::BracketLeft)) => (),
                Some(other) => {
                    println!("Unexpected {:?}, expected {{", other);
                    return None
                }
                _ => {
                    println!("Unexpected EOF, expected {{");
                    return None
                }
            }
            let retnum;
            match tokens_itr.next() {
                Some(Token::Constant(num)) => {
                    retnum = num;
                }
                _ => return None
            }
            match tokens_itr.next() {
                Some(Token::Punctuator(Punctuator::BracketRight)) => (),
                _ => return None
            }
            let name;
            match tokens_itr.next() {
                Some(Token::Identifier(s)) => name = s,
                _ => return None
            }
            let (args, seek) = argument_list(tokens.get(5..).unwrap())?;
            let mut tokens_itr_2 = tokens_itr.skip(seek);
            match tokens_itr_2.next() {
                Some(Token::Punctuator(Punctuator::BraceLeft)) => (),
                _ => return None
            }
            let (statements, seek2) = statement_list(tokens.get((6+seek)..).unwrap())?;
            let mut tokens_itr_3 = tokens_itr_2.skip(seek2);
            match tokens_itr_3.next() {
                Some(Token::Punctuator(Punctuator::BraceRight)) => (),
                _ => return None
            }
            let res = AST::Node(Node {
                op: Operand::FunctionDeclare{
                    name: name.to_string(), args: args
                },
                children: vec![statements; 1]
            });
            Some((res, 5+seek+2+seek2))
        }
        _ => None
    }
}

fn declarations_loop(tokens: &[Token], res: &mut Node, count: u32) -> bool {
    if tokens.len() == 0 {
        return true;
    }
    match declaration(tokens) {
        Some((ast, seek)) => {
            res.children.push(ast);
            declarations_loop(tokens.get(seek..).unwrap(), res, count+1)
        }
        None => {
            println!("{}", count);
            false
        }
    }
}

pub fn parse(tokens: &[Token]) -> Option<AST> {
    let mut res = Node {
        op: Operand::Declare,
        children: Vec::new()
    };
    if declarations_loop(tokens, &mut res, 0) {
        Some(AST::Node(res))
    } else {
        None
    }
}
