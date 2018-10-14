use super::tokenize::Token;
use super::tokenize::TokenType;
use super::tokenize::Punctuator;
use super::tokenize::Keyword;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Multiply,
    Division,
    Modulo,
    Substitute,
    Equal,
    LessThan,
    Greater,
    If,
    Call{name: String},
    Return,
    Jump,
    Do,
    Expression,
    Statement,
    Declare,
    FunctionDeclare{name: String, args: Vec<String>, retnum: usize}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Operator,
    pub children: Vec<AST>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Leaf {
    Identifier(String),
    Constant(i32)
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    Node(Node),
    Leaf(Leaf)
}

fn expression_call(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut itr = tokens.iter();
    match itr.next() {
        Some(Token{token:TokenType::Constant(constant), pos:_}) => Some((AST::Leaf(Leaf::Constant(*constant)), 1)),
        Some(Token{token:TokenType::Identifier(identifier), pos:_}) => {
            match itr.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisLeft), pos:_}) => {
                    match call(tokens, identifier.to_string()) {
                        Some((ast, seek)) => Some((ast, 2+seek)),
                        None => None
                    }
                }
                _ => Some((AST::Leaf(Leaf::Identifier(identifier.to_string())), 1))
            }
        }
        Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisLeft), pos:_}) => {
            match expression(tokens.get(1..).unwrap()) {
                Some((ast, seek)) => {
                    let mut itr2 = itr.skip(seek);
                    match itr2.next() {
                        Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisRight), pos:_}) => {}
                        _ => return None
                    }
                    Some((ast, 1+seek+1))
                }
                None => None
            }
        }
        Some(other) => {
            println!("{:?} is not begin of expression_call", other);
            None
        }
        None => None
    }
}

fn expression_mul(tokens: &[Token]) -> Option<(AST, usize)> {
    let (mut lhs, mut seek) = expression_call(tokens)?;
    let mut itr = tokens.iter().skip(seek);
    loop {
        match itr.next() {
            Some(Token{token:TokenType::Punctuator(Punctuator::Star), pos:_}) => {
                let (rhs, seek2) = expression_call(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Multiply,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            Some(Token{token:TokenType::Punctuator(Punctuator::Slash), pos:_}) => {
                let (rhs, seek2) = expression_call(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Division,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            Some(Token{token:TokenType::Punctuator(Punctuator::Percent), pos:_}) => {
                let (rhs, seek2) = expression_call(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Modulo,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            _ => return Some((lhs, seek))
        }
    }
}

fn expression_add(tokens: &[Token]) -> Option<(AST, usize)> {
    let (mut lhs, mut seek) = expression_mul(tokens)?;
    let mut itr = tokens.iter().skip(seek);
    loop {
        match itr.next() {
            Some(Token{token:TokenType::Punctuator(Punctuator::Plus), pos:_}) => {
                let (rhs, seek2) = expression_mul(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Add,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            Some(Token{token:TokenType::Punctuator(Punctuator::Minus), pos:_}) => {
                let (rhs, seek2) = expression_mul(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Sub,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            _ => return Some((lhs, seek))
        }
    }
}

fn expression_greater(tokens: &[Token]) -> Option<(AST, usize)> {
    let (mut lhs, mut seek) = expression_add(tokens)?;
    let mut itr = tokens.iter().skip(seek);
    loop {
        match itr.next() {
            Some(Token{token:TokenType::Punctuator(Punctuator::LessThan), pos:_}) => {
                let (rhs, seek2) = expression_add(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::LessThan,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            Some(Token{token:TokenType::Punctuator(Punctuator::Greater), pos:_}) => {
                let (rhs, seek2) = expression_add(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Greater,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            _ => return Some((lhs, seek))
        }
    }
}

fn expression(tokens: &[Token]) -> Option<(AST, usize)> {
    let (mut lhs, mut seek) = expression_greater(tokens)?;
    let mut itr = tokens.iter().skip(seek);
    loop {
        match itr.next() {
            Some(Token{token:TokenType::Punctuator(Punctuator::DoubleEqual), pos:_}) => {
                let (rhs, seek2) = expression_greater(tokens.get((seek+1)..).unwrap())?;
                lhs = AST::Node(Node {
                    op: Operator::Equal,
                    children: vec![lhs, rhs]
                });
                seek += 1 + seek2;
            }
            _ => return Some((lhs, seek))
        }
    }
}

fn expression_loop(tokens: &[Token], res: &mut Vec<AST>) -> Option<usize> {
    if tokens.len() >= 1 {
        if tokens[0].token == TokenType::Punctuator(Punctuator::ParenthesisRight) {
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
        Some(Token{token:TokenType::Punctuator(Punctuator::Comma), pos:_}) => {
            offset += 1;
            match expression_loop(tokens.get(offset..).unwrap(), res) {
                Some(seek) => Some(offset+seek),
                None => None
            }
        }
        Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisRight), pos:_}) => {
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

fn call(tokens: &[Token], funcname: String) -> Option<(AST, usize)> {
    let itr = tokens.iter().skip(2);
    match expression_list(tokens.get(2..).unwrap()) {
        Some((expressions, seek)) => {
            let mut itr2 = itr.skip(seek);
            match itr2.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisRight), pos:_}) => {
                    Some((AST::Node(Node {
                        op: Operator::Call{name: funcname},
                        children: expressions
                    }), seek+1))
                }
                Some(other) => {
                    println!("In call, At {:?}: Unexpected {:?}, expected )", other.pos, other.token);
                    return None
                }
                _ => {
                    println!("In call, Unexpected EOF, expected )");
                    return None
                }
            }
        }
        None => None
    }
}

fn if_statement(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut itr = tokens.iter();
    match itr.next() {
        Some(Token{token:TokenType::Keyword(Keyword::IF), pos:_}) => {}
        Some(other) => {
            println!("In if_statement, At {:?}: Unexpected {:?}, expected if", other.pos, other.token);
            return None
        }
        None => {
            println!("In if_statement, Unexpected EOF, expected if");
            return None
        }
    }
    let (cond, seek) = expression(tokens.get(1..).unwrap())?;
    let mut itr2 = itr.skip(seek);
    match itr2.next() {
        Some(Token{token:TokenType::Punctuator(Punctuator::BraceLeft), pos:_}) => {}
        Some(other) => {
            println!("In if_statement, At {:?}: Unexpected {:?}, expected {{", other.pos, other.token);
            return None
        }
        None => {
            println!("In if_statement, Unexpected EOF, expected {{");
            return None
        }
    }
    let (statements, seek2) = statement_list(tokens.get((1+seek+1)..).unwrap())?;
    let mut itr3 = itr2.skip(seek2);
    match itr3.next() {
        Some(Token{token:TokenType::Punctuator(Punctuator::BraceRight), pos:_}) => {}
        Some(other) => {
            println!("In if_statement, At {:?}: Unexpected {:?}, expected }}", other.pos, other.token);
            return None
        }
        None => {
            println!("In if_statement, Unexpected EOF, expected }}");
            return None
        }
    }
    match itr3.next() {
        Some(Token{token:TokenType::Keyword(Keyword::ELSIF), pos:_}) => {
            println!("Unsopported keyword: elsif");
            return None;
        }
        Some(Token{token:TokenType::Keyword(Keyword::ELSE), pos:_}) => {
            match itr3.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::BraceLeft), pos:_}) => {}
                Some(other) => {
                    println!("In if_statement, At {:?}: Unexpected {:?}, expected {{", other.pos, other.token);
                    return None;
                }
                None => {
                    println!("In if_statement, Unexpected EOF, expected {{");
                    return None;
                }
            }
            let (else_statements, seek3) = statement_list(tokens.get((1+seek+1+seek2+3)..).unwrap())?;
            let mut itr4 = itr3.skip(seek3);
            match itr4.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::BraceRight), pos:_}) => {}
                Some(other) => {
                    println!("In if_statement, At {:?}: Unexpected {:?}, expected }}", other.pos, other.token);
                    return None
                }
                None => {
                    println!("In if_statement, Unexpected EOF, expected }}");
                    return None
                }
            }
            Some((AST::Node(Node {
                op: Operator::If,
                children: vec![cond, statements, else_statements]
            }), 1+seek+1+seek2+3+seek3+1))
        }
        _ => {
            Some((AST::Node(Node {
                op: Operator::If,
                children: vec![cond, statements]
            }), 1+seek+1+seek2+3))
        }
    }
}

fn return_statement(tokens: &[Token]) -> Option<(AST, usize)> {
    if tokens.len() < 1 {
        println!("In return_statement, Unexpected EOF, expected return");
        return None;
    }
    if tokens[0].token != TokenType::Keyword(Keyword::RETURN) {
        println!("In return_statement, At {:?}: Unexpected {:?}, expected return", tokens[0].pos, tokens[0].token);
    }
    let (ast, seek) = expression(tokens.get(1..).unwrap())?;
    let mut itr = tokens.iter().skip(1+seek);
    match itr.next() {
        Some(Token{token:TokenType::Punctuator(Punctuator::SemiColon), pos:_}) => {}
        Some(other) => {
            println!("In return_statement, At {:?}: Unexpected {:?}, expected ;", other.pos, other.token);
            return None;
        }
        None => {
            println!("In return_statement, Unexpected EOF, expected ;");
            return None;
        }
    }
    Some((AST::Node(Node {
        op: Operator::Return,
        children: vec![ast]
    }), 1+seek+1))
}

fn statement(tokens: &[Token]) -> Option<(AST, usize)> {
    let mut itr = tokens.iter();
    let left: Leaf;
    match itr.next() {
        Some(Token{token:TokenType::Identifier(identifier), pos:_}) => {
            left = Leaf::Identifier(identifier.to_string());
        }
        Some(Token{token:TokenType::Keyword(Keyword::IF), pos:_}) => {
            return if_statement(tokens);
        }
        Some(Token{token:TokenType::Keyword(Keyword::RETURN), pos:_}) => {
            return return_statement(tokens);
        }
        Some(other) => {
            println!("In statement, At {:?}: Unexpected {:?}, expected identifier", other.pos, other.token);
            return None
        }
        _ => {
            println!("In statement, Unexpected EOF, expected identifier");
            return None
        }
    }
    match itr.next() {
        Some(Token{token:TokenType::Punctuator(Punctuator::Equal), pos:_}) => {
            let (exp, seek) = expression(tokens.get(2..).unwrap())?;
            let mut itr2 = itr.skip(seek);
            match itr2.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::SemiColon), pos:_}) => {}
                Some(other) => {
                    println!("In statement, At {:?}: Unexpected {:?}, expected ;", other.pos, other.token);
                    return None
                }
                _ => return None
            }
            Some((AST::Node(Node {
                op: Operator::Substitute,
                children: vec![
                    AST::Leaf(left),
                    exp
                ]
            }), 2+seek+1))
        }
        Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisLeft), pos:_}) => {
            let funcname = match left {
                Leaf::Identifier(name) => {
                    name
                }
                _ => return None
            };
            match call(tokens, funcname) {
                Some((node, seek)) => {
                    let mut itr2 = itr.skip(seek);
                    match itr2.next() {
                        Some(Token{token:TokenType::Punctuator(Punctuator::SemiColon), pos:_}) => {
                            Some((node, 2+seek+1))
                        }
                        Some(other) => {
                            println!("In statement, At {:?}, Unexpected {:?}, expected ;", other.pos, other.token);
                            None
                        }
                        None => None
                    }
                }
                None => None
            }
        }
        Some(other) => {
            println!("In statement, At {:?}: Unexpected {:?}, expected = or (", other.pos, other.token);
            return None
        }
        _ => {
            println!("In statement, Unexpected EOF, expected = or (");
            return None
        }
    }
}

fn statements_loop(tokens: &[Token], node: &mut Node) -> Option<usize> {
    if tokens.len() >= 1 {
        if tokens[0].token == TokenType::Punctuator(Punctuator::BraceRight) {
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
        op: Operator::Statement,
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
        Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisLeft), pos:_}) => {
            let mut res = Vec::<String>::new();
            let mut len = 1;
            loop {
                match tokens_itr.next() {
                    Some(Token{token:TokenType::Identifier(identifier), pos:_}) => {
                        res.push(identifier.to_string());
                    }
                    Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisRight), pos:_}) => {
                        return Some((res, len+1))
                    }
                    Some(other) => {
                        println!("In argument_list, At {:?}: Unexpected {:?}, expected identifier or )", other.pos, other.token);
                        return None
                    }
                    _ => {
                        println!("In argument_list, Unexpected EOF, expected identifier or )");
                        return None
                    }
                }
                len += 1;
                match tokens_itr.next() {
                    Some(Token{token:TokenType::Punctuator(Punctuator::Comma), pos:_}) => {}
                    Some(Token{token:TokenType::Punctuator(Punctuator::ParenthesisRight), pos:_}) => {
                        return Some((res, len+1))
                    }
                    Some(other) => {
                        println!("In argument_list, At {:?}: Unexpected {:?}, expected , or )", other.pos, other.token);
                        return None
                    }
                    _ => {
                        println!("In argument_list, Unexpected EOF, expected , or )");
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
        Some(Token{token:TokenType::Keyword(Keyword::FN), pos:_}) => {
            match tokens_itr.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::BracketLeft), pos:_}) => (),
                Some(other) => {
                    println!("In declaration, At {:?}:Unexpected {:?}, expected {{", other.pos, other.token);
                    return None
                }
                _ => {
                    println!("In declaration, Unexpected EOF, expected {{");
                    return None
                }
            }
            let retnum;
            match tokens_itr.next() {
                Some(Token{token:TokenType::Constant(num), pos:_}) => {
                    retnum = num;
                }
                _ => return None
            }
            match tokens_itr.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::BracketRight), pos:_}) => (),
                _ => return None
            }
            let name;
            match tokens_itr.next() {
                Some(Token{token:TokenType::Identifier(s), pos:_}) => name = s,
                _ => return None
            }
            let (args, seek) = argument_list(tokens.get(5..).unwrap())?;
            let mut tokens_itr_2 = tokens_itr.skip(seek);
            match tokens_itr_2.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::BraceLeft), pos:_}) => (),
                _ => return None
            }
            let (statements, seek2) = statement_list(tokens.get((6+seek)..).unwrap())?;
            let mut tokens_itr_3 = tokens_itr_2.skip(seek2);
            match tokens_itr_3.next() {
                Some(Token{token:TokenType::Punctuator(Punctuator::BraceRight), pos:_}) => (),
                _ => return None
            }
            let res = AST::Node(Node {
                op: Operator::FunctionDeclare{
                    name: name.to_string(), args: args, retnum: *retnum as usize
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
        op: Operator::Declare,
        children: Vec::new()
    };
    if declarations_loop(tokens, &mut res, 0) {
        Some(AST::Node(res))
    } else {
        None
    }
}
