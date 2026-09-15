#[derive(Debug, PartialEq)]
pub enum SyntaxTree {
    Var(String),
    Num(FloatEq),
    Op(Operation, Box<SyntaxTree>, Box<SyntaxTree>),
    Eq(Box<SyntaxTree>, Box<SyntaxTree>)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Operation {
    Add,
    Mult,
    Neg,
    Div,
}

#[derive(Debug)]
pub enum ParseError {
    Err(String)
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Token {
    Variable(String),
    /// literal numbers may end with a decimal ("1.") but not start with one (".1")
    Literal(FloatEq),
    OpenPeren,
    ClosePeren,
    Op(Operation),
    Eq,
}
#[derive(Clone, Copy, Debug)]
pub struct FloatEq(f32);

impl SyntaxTree {
    pub fn parse(string: &str) -> Result<Box<Self>, ParseError> {
        let mut tokens = tokanise_string(string).map_err(|err| ParseError::Err(err))?;
        println!("Token stream: {:?}", tokens);
        insert_impl_mult(&mut tokens);

        let mut depth = 0;
        let tokens_with_depth = tokens.into_iter().map(|token| {
            match token {
                Token::OpenPeren => {depth += 1},
                Token::ClosePeren => {depth -= 1},
                _ => {}
            }
            (token, depth)
        }).collect::<Vec<(Token, usize)>>();

        Self::from_tokens(tokens_with_depth, 0)
    }

    /// parses tokens back to front
    fn from_tokens(tokens: Vec<(Token, usize)>, current_depth: usize) -> Result<Box<Self>, ParseError> {
        tokens.split(|token| *token == (Token::Op(Operation::Neg), current_depth)).try_fold(None, |last, substream| {
            let next = substream.split(|token| *token == (Token::Op(Operation::Add), current_depth)).try_fold(None, |last, substream| {
                let next = substream.split(|token| *token == (Token::Op(Operation::Mult), current_depth)).try_fold(None, |last, substream| {
                    let next = substream.split(|token| *token == (Token::Op(Operation::Div), current_depth)).try_fold(None, |last, substream| {
                        let next = if substream.len() == 1 {
                            let (token, depth) = &substream.first().unwrap();
                            if *depth != current_depth {return Err(ParseError::Err(format!("Expected depth of {}, instead found depth of {} at token {:?}", current_depth, depth, token)));}
                            let tree = match token {
                                Token::Literal(val) => SyntaxTree::Num((*val).into()),
                                Token::Variable(var) => SyntaxTree::Var(var.clone()),
                                _ => return Err(ParseError::Err(format!("Invalid token: {:?}", token)))
                            };
                            Box::new(tree)
                        } else {
                            Self::from_tokens(substream[1..substream.len() - 1].into_iter().map(|t| t.clone()).collect(), current_depth + 1)?
                        };

                        if let Some(last) = last {
                            Ok(Some(Box::new(SyntaxTree::Op(Operation::Div, last, next))))
                        } else {
                            Ok(Some(next))
                        }

                    })?.ok_or(ParseError::Err(format!("Invalid token: {:?}", Token::Op(Operation::Div))))?;
                    if let Some(last) = last {
                        Ok(Some(Box::new(SyntaxTree::Op(Operation::Mult, last, next))))
                    } else {
                        Ok(Some(next))
                    }

                })?.ok_or(ParseError::Err(format!("Invalid token: {:?}", Token::Op(Operation::Mult))))?;
                if let Some(last) = last {
                    Ok(Some(Box::new(SyntaxTree::Op(Operation::Add, last, next))))
                } else {
                    Ok(Some(next))
                }

            })?.ok_or(ParseError::Err(format!("Invalid token: {:?}", Token::Op(Operation::Add))))?;
            if let Some(last) = last {
                Ok(Some(Box::new(SyntaxTree::Op(Operation::Neg, last, next))))
            } else {
                Ok(Some(next))
            }
        })?.ok_or(ParseError::Err(format!("Invalid token: {:?}", Token::Op(Operation::Neg))))
    }
}

pub fn tokanise_string(string: &str) -> Result<Vec<Token>, String> {
    let mut invalid_char = None;
    let result = string.split_whitespace().flat_map(|word| {
        let mut tokens = vec![];
        let mut chars = word.chars().peekable();

        while let Ok(token) = tokanise_top(&mut chars) {
            tokens.push(token);
        }

        // any chars left are invalid
        if invalid_char.is_none() {
            invalid_char = chars.next();
        }

        tokens
    }).collect();
    if let Some(c) = invalid_char {
        Err(format!("Invalid char: {}", c))
    } else {
        Ok(result)
    }
}
fn tokanise_top<I>(chars: &mut core::iter::Peekable<I>) -> Result<Token, String>
where I: Iterator<Item = char> {
    let first = chars.peek().ok_or("Cannot turn empty string into token".to_owned())?;
    match first {
        a if a.is_alphabetic() => {
            // add variable
            let mut var = String::from(chars.next().unwrap());
            while let Some(c) = chars.next_if(|c| c.is_alphanumeric()) {
                var.push(c);
            }
            Ok(Token::Variable(var))
        },
        n if n.is_numeric() => {
            // add literal number
            let mut num = String::from(chars.next().unwrap());
            let mut point_used = false;
            while let Some(c) = chars.next_if(|c| {
                if !point_used && *c == '.' {
                    point_used = true;
                    true
                } else {
                    c.is_numeric()
                }
            }) {
                num.push(c);
            }
            Ok(Token::Literal(num.parse::<f32>().map_err(|e| e.to_string())?.into()))
        },
        '(' => {chars.next(); Ok(Token::OpenPeren)},
        ')' => {chars.next(); Ok(Token::ClosePeren)},
        '+' => {chars.next(); Ok(Token::Op(Operation::Add))},
        '*' => {chars.next(); Ok(Token::Op(Operation::Mult))},
        '-' => {chars.next(); Ok(Token::Op(Operation::Neg))},
        '/' => {chars.next(); Ok(Token::Op(Operation::Div))},
        '=' => {chars.next(); Ok(Token::Eq)},
        _ => Err(format!("Invalid token first char: {}", first).to_owned())
    }
}

fn insert_impl_mult(tokens: &mut Vec<Token>) {
    if tokens.len() < 2 {
        return;
    }

    let mut i = 1;
    let mut prev_expr = match tokens[0] {
        Token::ClosePeren | Token::Literal(_) | Token::Variable(_) => true,
        _ => false
    };
    while i < tokens.len() {
        match tokens[i] {
            Token::Literal(_) | Token::Variable(_) => {
                // expr, impl mult before/after is possible
                if prev_expr {
                    tokens.insert(i, Token::Op(Operation::Mult));
                    i += 1;
                }
                prev_expr = true;
            },
            Token::OpenPeren => {
                // start of expr, impl mult before is possible
                if prev_expr {
                    tokens.insert(i, Token::Op(Operation::Mult));
                    i += 1;
                }
                prev_expr = false;
            },
            Token::ClosePeren => {
                // end of expr, impl mult after is possible
                prev_expr = true;
            },
            _ => {prev_expr = false;}
        };
        i += 1;
    }
}

// /// Assumes `start` is the index of opening bracket, returns index of closing bracket
// fn find_closing_bracket(tokens: &[Token], start: usize) -> Result<usize, ParseError> {
//     let mut depth = 1;
//     let mut i = start + 1;
//     while i < tokens.len() {
//         let token = &tokens[i];
//         if *token == Token::OpenPeren {
//             depth += 1;
//         } else if *token == Token::ClosePeren {
//             depth -= 1;
//             if depth == 0 {
//                 return Ok(i);
//             }
//         }
//         i += 1;
//     }
//     Err(ParseError::Err(format!("Unable to find closing bracket, final depth: {:?}", depth)))
//     // if depth != 0 {
//     //     Err(ParseError::Err)
//     // } else {
//     //     Ok()
//     // }
// }

/// Assumes closing bracket has already been removed
fn find_openning_bracket(tokens: &[Token]) -> Result<usize, ParseError> {
    let mut tokens = tokens.iter().enumerate().rev();
    let mut depth: usize = 1;
    while let Some((i, token)) = tokens.next() {
        match token {
            Token::ClosePeren => depth += 1,
            Token::OpenPeren => {depth -= 1; if depth == 0 {return Ok(i);}},
            _ => {}
        }
    }
    return Err(ParseError::Err("Unable to find corresponding closing bracket".to_owned()));
}

pub fn tokens_to_string<'a>(tokens: impl IntoIterator<Item = &'a Token>) -> String {
    tokens.into_iter().map(|t| t.to_string()).collect::<Vec<String>>().join(" ")
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Variable(s) => s.clone(),
            Token::Literal(FloatEq(num)) => num.to_string(),
            Token::Op(operation) => operation.to_string(),
            Token::OpenPeren => "(".to_string(),
            Token::ClosePeren => ")".to_string(),
            Token::Eq => "=".to_string(),
        }
    }
}
impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Operation::Add => "+",
            Operation::Mult => "*",
            Operation::Neg => "-",
            Operation::Div => "/",
        }.to_string()
    }
}

impl PartialEq for FloatEq {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0) < f32::EPSILON
    }
}
impl Eq for FloatEq {}
impl From<f32> for FloatEq {
    fn from(value: f32) -> Self {
        FloatEq(value)
    }
}
impl Into<f32> for FloatEq {
    fn into(self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn syntax_test() {
        let test = "11 + 22 * 33";
        let tree = SyntaxTree::parse(test);
        let expected = Box::new(
            SyntaxTree::Op(
                Operation::Add,
                Box::new(SyntaxTree::Num(11.0.into())),
                Box::new(SyntaxTree::Op(
                    Operation::Mult,
                    Box::new(SyntaxTree::Num(22.0.into())),
                    Box::new(SyntaxTree::Num(33.0.into())),
                )), 
            )
        );
        assert_eq!(tree.unwrap(), expected);
    }

    #[test]
    fn syntax_parens() {
        let test = "11 + (33 + (4 * 5))";
        let tree = SyntaxTree::parse(test);
        let expected = Box::new(
            SyntaxTree::Op(
                Operation::Add,
                Box::new(SyntaxTree::Num(11.0.into())),
                Box::new(SyntaxTree::Op(
                    Operation::Add,
                    Box::new(SyntaxTree::Num(33.0.into())),
                    Box::new(SyntaxTree::Op(
                        Operation::Mult,
                        Box::new(SyntaxTree::Num(4.0.into())),
                        Box::new(SyntaxTree::Num(5.0.into())),
                    )), 
                )), 
            )
        );
        assert_eq!(tree.unwrap(), expected);
    }

    #[test]
    fn syntax_impl_mult() {
        let test = "11 (a (4 5)) 2 + 1";
        let tree = SyntaxTree::parse(test);
        let expected = Box::new(
            SyntaxTree::Op(
                Operation::Add,
                Box::new(SyntaxTree::Op(
                    Operation::Mult,
                    Box::new(SyntaxTree::Op(
                        Operation::Mult,
                        Box::new(SyntaxTree::Num(11.0.into())),
                        Box::new(SyntaxTree::Op(
                            Operation::Mult,
                            Box::new(SyntaxTree::Var("a".to_owned())),
                            Box::new(SyntaxTree::Op(
                                Operation::Mult,
                                Box::new(SyntaxTree::Num(4.0.into())),
                                Box::new(SyntaxTree::Num(5.0.into())),
                            )), 
                        )), 
                    )),
                    Box::new(SyntaxTree::Num(2.0.into())),
                )), 
                Box::new(SyntaxTree::Num(1.0.into())),
            )
        );
        assert_eq!(tree.unwrap(), expected);
    }

    #[test]
    fn token_decimals() {
        let test = "1.2x + 20.01y + 15.z";
        let tokens = tokanise_string(test);
        let expected = vec![
            Token::Literal(1.2.into()),
            Token::Variable("x".to_owned()), 
            Token::Op(Operation::Add),
            Token::Literal(20.01.into()),
            Token::Variable("y".to_owned()),
            Token::Op(Operation::Add),
            Token::Literal(15.0.into()),
            Token::Variable("z".to_owned()),
        ];
        assert_eq!(tokens, Ok(expected));
    }

    #[test]
    fn token_simple() {
        let test = "a xy 1 32 ( = ) * +";
        let tokens = tokanise_string(test);
        let expected = vec![
            Token::Variable("a".to_owned()), 
            Token::Variable("xy".to_owned()),
            Token::Literal(1.0.into()),
            Token::Literal(32.0.into()),
            Token::OpenPeren,
            Token::Eq,
            Token::ClosePeren,
            Token::Op(Operation::Mult),
            Token::Op(Operation::Add),
        ];
        assert_eq!(tokens, Ok(expected));
    }

    #[test]
    fn token_whitespace() {
        let test = " a b ab  1    abc 123      1 a 1 2";
        let tokens = tokanise_string(test);
        let expected = vec![
            Token::Variable("a".to_owned()), 
            Token::Variable("b".to_owned()),
            Token::Variable("ab".to_owned()),
            Token::Literal(1.0.into()),
            Token::Variable("abc".to_owned()),
            Token::Literal(123.0.into()),
            Token::Literal(1.0.into()),
            Token::Variable("a".to_owned()),
            Token::Literal(1.0.into()),
            Token::Literal(2.0.into()),
        ];
        assert_eq!(tokens, Ok(expected));
    }

    #[test]
    fn token_no_whitespace() {
        let test = "whattheheckisthis123yes*2*5*me+(eea*ee)";
        let tokens = tokanise_string(test);
        let expected = vec![
            Token::Variable("whattheheckisthis123yes".to_owned()), 
            Token::Op(Operation::Mult),
            Token::Literal(2.0.into()),
            Token::Op(Operation::Mult),
            Token::Literal(5.0.into()),
            Token::Op(Operation::Mult),
            Token::Variable("me".to_owned()),
            Token::Op(Operation::Add),
            Token::OpenPeren,
            Token::Variable("eea".to_owned()),
            Token::Op(Operation::Mult),
            Token::Variable("ee".to_owned()),
            Token::ClosePeren,
        ];
        assert_eq!(tokens, Ok(expected));
    }

    #[test]
    fn token_literal_variable() {
        let test = "1a2";
        let tokens: Result<Vec<Token>, String> = tokanise_string(test);
        let expected = vec![
            Token::Literal(1.0.into()),
            Token::Variable("a2".to_owned()),
        ];
        assert_eq!(tokens, Ok(expected));
    }


    #[test]
    fn token_invalid_char() {
        let test = " 123 & 3231 a";
        let tokens: Result<Vec<Token>, String> = tokanise_string(test);
        let expected: Result<Vec<Token>, String> = Err("Invalid char: &".to_owned());
        println!("{:?}", tokens);
        assert_eq!(tokens, expected);
    }
}