#[derive(Debug)]
pub enum SyntaxTree {
    Var(String),
    Num(f32),
    Op(Operation, Box<SyntaxTree>, Box<SyntaxTree>)
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
enum Token {
    Var(String),
    OpenPeren,
    ClosePeren,
    Op(Operation),
}

impl SyntaxTree {
    pub fn parse(string: &str) -> Result<Box<Self>, ParseError> {
        let tokens = convert_to_tokens(string).map_err(|err| ParseError::Err(err))?;
        println!("Token stream: {:?}", tokens);
        Self::parse_tokens(&tokens.as_slice())
    }

    fn parse_tokens(tokens: &[Token]) -> Result<Box<Self>, ParseError> {
        let mut result: Option<Box<SyntaxTree>> = None;
        let mut i = 0;
        while i < tokens.len() {
            let next_expr = Self::parse_top(&tokens[i..tokens.len()]);
            if let Some(prev) = result.take() {
                if let Ok((next, end_i)) = next_expr {
                    // implied mult (no operator)
                    result = Some(Box::new(SyntaxTree::Op(Operation::Mult, prev, next)));
                    i += end_i;
                } else if let Token::Op(Operation::Add) = tokens[i] {
                    // addition
                    let next = Self::parse_tokens(&tokens[i+1..tokens.len()])?;
                    result = Some(Box::new(SyntaxTree::Op(Operation::Add, prev, next)));
                    i = tokens.len();
                } else if let Token::Op(Operation::Mult) = tokens[i] {
                    // multiplication
                    let (next, end_i) = Self::parse_top(&tokens[i+1..tokens.len()])?;
                    result = Some(Box::new(SyntaxTree::Op(Operation::Mult, prev, next)));
                    i += 1 + end_i;
                }
            } else {
                // truly first expression
                result = Some(next_expr?.0);
            }
            // println!("Partial result: {:?}", result);
            i += 1;
        }
        result.ok_or(ParseError::Err("Unable to parse anything!".to_owned()))
    }

    /// Parses first valid tree of the token list, returning also the index of the last token parsed
    fn parse_top(tokens: &[Token]) -> Result<(Box<Self>, usize), ParseError> {
        match &tokens[0] {
            Token::OpenPeren => {
                let closing_i = find_closing_bracket(tokens, 0)?;
                Self::parse_tokens(&tokens[1..closing_i]).map(|tree| (tree, closing_i))
            },
            Token::Var(string) => Ok((Box::new(SyntaxTree::Var(string.clone())), 0)),
            _ => Err(ParseError::Err(format!("Invalid first token for expression: {:?}", tokens[0])))
        }
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
                    tokens.push(Token::Var(var));
                    // not a name/value
                    match c {
                    '(' => tokens.push(Token::OpenPeren),
                    ')' => tokens.push(Token::ClosePeren),
                    '*' => tokens.push(Token::Op(Operation::Mult)),
                    '+' => tokens.push(Token::Op(Operation::Add)),
                    _ => {invalid_char = Some(c); break;}
                }
                }
            } else if c.is_alphanumeric() {
                // var start
                current_var = Some(c.to_string());
            }
            else {
                // not a name/value
                match c {
                    '(' => tokens.push(Token::OpenPeren),
                    ')' => tokens.push(Token::ClosePeren),
                    '*' => tokens.push(Token::Op(Operation::Mult)),
                    '+' => tokens.push(Token::Op(Operation::Add)),
                    _ => {invalid_char = Some(c); break;}
                }
            }
        }
        // word end, push cached name as var
        if let Some(var) = current_var {
            tokens.push(Token::Var(var));
        }

        tokens
    }).collect();
    if let Some(c) = invalid_char {
        Err(format!("Invalid char: {}", c))
    } else {
        Ok(result)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn token_simple() {
        let test = "a xy 1 32 ( ) * +";
        let tokens = convert_to_tokens(test);
        let expected = vec![
            Token::Var("a".to_owned()), 
            Token::Var("xy".to_owned()),
            Token::Var("1".to_owned()),
            Token::Var("32".to_owned()),
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
        let tokens = convert_to_tokens(test);
        let expected = vec![
            Token::Var("a".to_owned()), 
            Token::Var("b".to_owned()),
            Token::Var("ab".to_owned()),
            Token::Var("1".to_owned()),
            Token::Var("abc".to_owned()),
            Token::Var("123".to_owned()),
            Token::Var("1".to_owned()),
            Token::Var("a".to_owned()),
            Token::Var("1".to_owned()),
            Token::Var("2".to_owned()),
        ];
        assert_eq!(tokens, Ok(expected));
    }

    #[test]
    fn token_no_whitespace() {
        let test = "whattheheckisthis123yes*2*5*me+(eea*ee)";
        let tokens = convert_to_tokens(test);
        let expected = vec![
            Token::Var("whattheheckisthis123yes".to_owned()), 
            Token::Op(Operation::Mult),
            Token::Var("2".to_owned()),
            Token::Op(Operation::Mult),
            Token::Var("5".to_owned()),
            Token::Op(Operation::Mult),
            Token::Var("me".to_owned()),
            Token::Op(Operation::Add),
            Token::OpenPeren,
            Token::Var("eea".to_owned()),
            Token::Op(Operation::Mult),
            Token::Var("ee".to_owned()),
            Token::ClosePeren,
        ];
        assert_eq!(tokens, Ok(expected));
    }

    // #[test]
    // fn token_invalid_name() {
    //     let test = "1a";
    //     let tokens: Result<Vec<Token>, String> = convert_to_tokens(test);
    //     let expected: Result<Vec<Token>, String> = Err("Invalid var name: 1a".to_owned());
    //     assert_eq!(tokens, expected);
    // }


    #[test]
    fn token_invalid_char() {
        let test = " 123 / 3231 a";
        let tokens: Result<Vec<Token>, String> = convert_to_tokens(test);
        let expected: Result<Vec<Token>, String> = Err("Invalid char: /".to_owned());
        assert_eq!(tokens, expected);
    }
}