#[derive(Debug)]
pub struct IRError {
    pub message: String,
}

// impl IRError {
//     pub fn new(message: &str) -> Self {
//         Self {
//             message: message.to_string(),
//         }
//     }
// }

impl std::fmt::Display for IRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IRError: {}", self.message)
    }
}
