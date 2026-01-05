#[derive(Debug)]
pub struct IRError {
    pub kind: IRErrorKind,
    pub message: String,
}

#[derive(Debug)]
pub enum IRErrorKind {
    VariableNotFound,
    VariableAlreadyDefined,
    LabelNotFound,
    LabelAlreadyDefined,
    AssignmentRequired,
    NotImplemented,
}

impl IRError {
    pub fn new(kind: IRErrorKind, message: &str) -> Self {
        Self {
            kind,
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for IRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IRError: {:?} = {} ", self.kind, self.message)
    }
}
