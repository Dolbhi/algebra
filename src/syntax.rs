use std::ops::Index;

pub enum SyntaxTree {
    Var(usize),
    Num(f32),
    Op(Operation, Box<SyntaxTree>, Box<SyntaxTree>)
}

pub enum Operation {
    Add,
    Mult
}

pub enum ParseError {
    Err
}

enum Token {
    Val(String),
    OpenPeren,
    ClosePeren,
    Op(Operation),
}

impl SyntaxTree {
    pub fn parse(string: &str) -> Result<Self, ParseError> {
        let tokens = convert_to_tokens(string);

        Err(ParseError::Err)
    }
}

fn convert_to_tokens(string: &str) -> Result<Vec<Token>, String> {
    let mut invalid_char = None;
    let result = string.split_whitespace().flat_map(|word| {
        let mut tokens = vec![];
        let mut current_var: Option<String> = None;
        for c in word.chars() {
            if let Some(mut var) = current_var.take() {
                if c.is_alphanumeric() {
                    // extend var name
                    var.push(c);
                    current_var = Some(var);
                } else {
                    // var name end, push as token
                    tokens.push(Token::Val(var));
                }
            } else if c.is_alphabetic() {
                // var start
                current_var = Some(c.to_string());
            }
            match c {
                '(' => tokens.push(Token::OpenPeren),
                ')' => tokens.push(Token::ClosePeren),
                '*' => tokens.push(Token::Op(Operation::Mult)),
                '+' => tokens.push(Token::Op(Operation::Add)),
                _ => {invalid_char = Some(c); break;}
            }
        }

        tokens
    }).collect();
    if let Some(c) = invalid_char {
        Err(format!("Invalid char: {}", c))
    } else {
        Ok(result)
    }
}

// /// Assumes `start` is the index of opening bracket
// fn find_closing_bracket(string: &str, start: usize) -> Result<usize, ParseError> {
//     let mut depth = 1;
//     let mut i = start + 1;
//     while i < string.len() {
//         let char = string.get(i..i+1).unwrap();
//         if char == '(' {
//             depth += 1;
//         } else if char == ')' {
//             depth -= 1;
//             if depth == 0 {
//                 return Ok(i);
//             }
//         }
//         i += 1;
//     }
//     Err(ParseError::Err)
//     // if depth != 0 {
//     //     Err(ParseError::Err)
//     // } else {
//     //     Ok()
//     // }
// }