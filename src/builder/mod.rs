use crate::error::all_error::AllError;

#[derive(Debug, Clone, PartialEq)]
pub struct Builder {
    filenames: Vec<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self { filenames: vec![] }
    }

    pub fn set_filenames(&mut self, filenames: Vec<String>) {
        self.filenames = filenames;
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn build(&mut self) -> Result<String, AllError> {
        todo!()
    }
}
