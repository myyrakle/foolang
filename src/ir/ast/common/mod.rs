pub mod literal;

#[derive(Debug)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug)]
pub enum Operand {
    Identifier(Identifier),
    Literal(literal::LiteralValue),
}
