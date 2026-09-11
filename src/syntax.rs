pub enum SyntaxTree {
    Var(usize),
    Num(f32),
    Op(Operation, Box<SyntaxTree>, Box<SyntaxTree>)
}

pub enum Operation {
    Add,
    Mult
}