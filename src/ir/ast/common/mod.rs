pub mod literal;

#[derive(Debug)]
pub struct Identifier {
    pub name: String,
}

impl From<&str> for Identifier {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Identifier(Identifier),
    Literal(literal::LiteralValue),
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
}

impl From<&str> for Label {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
