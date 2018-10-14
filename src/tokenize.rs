#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    FN,
    RETURN,
    VAL,
    IF,
    ELSIF,
    ELSE
}

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    BraceLeft,
    BraceRight,
    ParenthesisLeft,
    ParenthesisRight,
    BracketLeft,
    BracketRight,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    DoubleEqual,
    SemiColon,
    LessThan,
    Greater
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Identifier(String),
    Constant(i32),
    Punctuator(Punctuator)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    line: usize,
    block_pos: usize,
    char_pos: usize
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token: TokenType,
    pub pos: Pos
}

fn is_identifier_nondigit(c: char) -> bool {
    return c.is_alphabetic() || c == '_';
}

fn is_identifier_chars(c: char) -> bool {
    return c.is_alphanumeric() || c == '_';
}

fn make_keyword(keyword: &str) -> Option<Keyword> {
    match keyword {
        "fn" => Some(Keyword::FN),
        "return" => Some(Keyword::RETURN),
        "val" => Some(Keyword::VAL),
        "if" => Some(Keyword::IF),
        "elsif" => Some(Keyword::ELSIF),
        "else" => Some(Keyword::ELSE),
        _ => None
    }
}

fn make_punctuator(punctuator: char) -> Option<Punctuator> {
    match punctuator {
        '(' => Some(Punctuator::ParenthesisLeft),
        ')' => Some(Punctuator::ParenthesisRight),
        '[' => Some(Punctuator::BracketLeft),
        ']' => Some(Punctuator::BracketRight),
        '{' => Some(Punctuator::BraceLeft),
        '}' => Some(Punctuator::BraceRight),
        ',' => Some(Punctuator::Comma),
        '+' => Some(Punctuator::Plus),
        '-' => Some(Punctuator::Minus),
        '*' => Some(Punctuator::Star),
        '/' => Some(Punctuator::Slash),
        '%' => Some(Punctuator::Percent),
        '=' => Some(Punctuator::Equal),
        ';' => Some(Punctuator::SemiColon),
        '<' => Some(Punctuator::LessThan),
        '>' => Some(Punctuator::Greater),
        _ => None
    }
}

fn tokenize_impl(code: &str, pos: Pos) -> Option<(Token, usize)> {
    let mut chars = code.chars().peekable();
    match chars.next() {
        Some(c) => {
            if is_identifier_nondigit(c) {
                let mut identifier: String = c.to_string();
                let mut len: usize = 1;
                loop {
                    match chars.next() {
                        Some(d) => {
                            if is_identifier_chars(d) {
                                identifier.push(d);
                            } else {
                                break;
                            }
                        }
                        None => break
                    }
                    len += 1;
                }
                match make_keyword(&identifier) {
                    Some(keyword) => Some((Token{token:TokenType::Keyword(keyword), pos}, len)),
                    None => Some((Token{token:TokenType::Identifier(identifier), pos}, len))
                }
            } else if c.is_digit(10) {
                let mut imm: i32 = c.to_digit(10).unwrap() as i32;
                let mut len: usize = 1;
                loop {
                    match chars.next() {
                        Some(d) => {
                            if d.is_digit(10) {
                                imm *= 10;
                                imm += d.to_digit(10).unwrap() as i32;
                            } else {
                                break;
                            }
                        }
                        None => break
                    }
                    len += 1;
                }
                Some((Token{token:TokenType::Constant(imm), pos}, len))
            } else if c == '\'' {
                match chars.next() {
                    Some('\\') => {
                        match chars.next() {
                            Some('\\') => {
                                if chars.next() == Some('\'') {
                                    Some((Token{token:TokenType::Constant('\\' as i32), pos}, 4))
                                } else {
                                    None
                                }
                            }
                            Some('\'') => {
                                if chars.next() == Some('\'') {
                                    Some((Token{token:TokenType::Constant('\'' as i32), pos}, 4))
                                } else {
                                    None
                                }
                            }
                            _ => None
                        }
                    }
                    Some(d) => {
                        if chars.next() == Some('\'') {
                            Some((Token{token:TokenType::Constant(d as i32), pos}, 3))
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            } else {
                let punc = make_punctuator(c);
                match punc {
                    Some(Punctuator::Equal) => {
                        if chars.peek() == Some(&'=') {
                            Some((Token{token:TokenType::Punctuator(Punctuator::DoubleEqual), pos}, 2))
                        } else {
                            Some((Token{token:TokenType::Punctuator(Punctuator::Equal), pos}, 1))
                        }
                    }
                    Some(punc) => Some((Token{token:TokenType::Punctuator(punc), pos}, 1)),
                    None => None
                }
            }
        }
        None => None
    }
}

fn tokenize_loop(block: &str, pos: Pos, tokens: &mut Vec<Token>) -> bool {
    if block.len() == 0 {
        return true;
    }
    match tokenize_impl(block, pos) {
        Some((token, seek)) => {
            tokens.push(token);
            tokenize_loop(block.get(seek..).unwrap(), Pos {
                line: pos.line,
                block_pos: pos.block_pos,
                char_pos: pos.char_pos + seek
            }, tokens)
        }
        None => false
    }
}

pub fn tokenize(code: &str) -> Option<Vec<Token>> {
    let mut res: Vec<Token> = Vec::new();
    for (i, line) in code.split_terminator('\n').enumerate() {
        for (j, block) in line.split_whitespace().enumerate() {
            if !tokenize_loop(block, Pos {
                        line: i,
                        block_pos: j,
                        char_pos: 0
                    }, &mut res) {
                println!("Failed to tokenize at line {}, block {}: {}", i, j, block);
                return None;
            }
        }
    }
    Some(res)
}
