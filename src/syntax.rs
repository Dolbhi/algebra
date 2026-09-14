use std::collections::VecDeque;

#[derive(Debug)]
pub enum SyntaxTree {
    Var(String),
    Num(f32),
    Op(Operation, Box<SyntaxTree>, Box<SyntaxTree>),
    Eq(Box<SyntaxTree>, Box<SyntaxTree>)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Operation {
    Add,
    Mult
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
        let tokens = tokanise_string(string).map_err(|err| ParseError::Err(err))?;
        println!("Token stream: {:?}", tokens);
        Self::from_tokens(tokens.into())
    }

    fn from_tokens(mut tokens: VecDeque<Token>) -> Result<Box<Self>, ParseError> {
        let mut result: Option<Box<SyntaxTree>> = None;
        while let Some(token) = tokens.front() {
            match token {
                Token::Op(Operation::Add) => {
                    let token = token.clone();
                    tokens.pop_front();
                    if let Some(prev) = result {
                        return Ok(Box::new(SyntaxTree::Op(Operation::Add, prev, Self::from_tokens(tokens)?)))
                    } else {
                        return Err(ParseError::Err(format!("Invalid token: {:?}", token)));
                    }
                },
                Token::Eq => {
                    let token = token.clone();
                    tokens.pop_front();
                    if let Some(prev) = result {
                        return Ok(Box::new(SyntaxTree::Eq(prev, Self::from_tokens(tokens)?)))
                    } else {
                        return Err(ParseError::Err(format!("Invalid token: {:?}", token)));
                    }
                },
                Token::Op(Operation::Mult) => {
                    let token = token.clone();
                    tokens.pop_front();
                    if let Some(prev) = result {
                        result = Some(Box::new(SyntaxTree::Op(Operation::Add, prev, Self::parse_top(&mut tokens)?)))
                    } else {
                        return Err(ParseError::Err(format!("Invalid token: {:?}", token)));
                    }
                },
                _ => {
                    tokens.pop_front();
                    let next = Self::parse_top(&mut tokens)?;
                    if let Some(prev) = result {
                        result = Some(Box::new(SyntaxTree::Op(Operation::Mult, prev, next)))
                    } else {
                        result = Some(next)
                    }
                }
            }
        }
        result.ok_or(ParseError::Err("Unable to parse anything!".to_owned()))
    }

    /// Parses first valid tree of the token list, returning also the index of the last token parsed
    fn parse_top(tokens: &mut VecDeque<Token>) -> Result<Box<Self>, ParseError> {
        if let Some(first) = tokens.front() {
            match first {
                Token::OpenPeren => {
                    tokens.pop_front();
                    tokens.make_contiguous();
                    let mut temp = tokens.split_off(find_closing_bracket(tokens.as_slices().0, 0)?);
                    temp.pop_front();
                    std::mem::swap(tokens, &mut temp);
                    Self::from_tokens(temp)
                },
                Token::Variable(string) => {
                    let string = string.clone();
                    tokens.pop_front();
                    Ok(Box::new(SyntaxTree::Var(string.clone())))
                },
                Token::Literal(val) => {
                    let val = *val;
                    tokens.pop_front();
                    Ok(Box::new(SyntaxTree::Num(val.into())))
                },
                _ => Err(ParseError::Err(format!("Invalid first token for expression: {:?}", first)))
            }
        } else {
            Err(ParseError::Err("Unable to parse empty token stream".to_owned()))
        }
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
        '*' => {chars.next(); Ok(Token::Op(Operation::Mult))},
        '+' => {chars.next(); Ok(Token::Op(Operation::Add))},
        '=' => {chars.next(); Ok(Token::Eq)},
        _ => Err(format!("Invalid token first char: {}", first).to_owned())
    }
}

/// Assumes `start` is the index of opening bracket, returns index of closing bracket
fn find_closing_bracket(tokens: &[Token], start: usize) -> Result<usize, ParseError> {
    let mut depth = 1;
    let mut i = start + 1;
    while i < tokens.len() {
        let token = &tokens[i];
        if *token == Token::OpenPeren {
            depth += 1;
        } else if *token == Token::ClosePeren {
            depth -= 1;
            if depth == 0 {
                return Ok(i);
            }
        }
        i += 1;
    }
    Err(ParseError::Err(format!("Unable to find closing bracket, final depth: {:?}", depth)))
    // if depth != 0 {
    //     Err(ParseError::Err)
    // } else {
    //     Ok()
    // }
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
        let test = "a xy 1 32 ( ) * +";
        let tokens = tokanise_string(test);
        let expected = vec![
            Token::Variable("a".to_owned()), 
            Token::Variable("xy".to_owned()),
            Token::Literal(1.0.into()),
            Token::Literal(32.0.into()),
            Token::OpenPeren,
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
        let test = " 123 / 3231 a";
        let tokens: Result<Vec<Token>, String> = tokanise_string(test);
        let expected: Result<Vec<Token>, String> = Err("Invalid char: /".to_owned());
        println!("{:?}", tokens);
        assert_eq!(tokens, expected);
    }
}