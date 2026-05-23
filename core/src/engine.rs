use crate::error::{JSError, JSResult};

pub trait JSEngine {
    fn run(&self, code: &str, filename: &str) -> Result<JSResult, JSError>;
}
