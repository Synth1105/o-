use std::process::Command;

use o_core::{
    engine::JSEngine,
    error::{JSError, JSResult},
};

pub struct BinEngine {
    path: String
}

impl BinEngine {
    pub fn new(path: String) -> Self {
        Self {
            path
        }
    }
}


impl JSEngine for BinEngine {
    fn run(&self, code: &str, filename: &str) -> Result<o_core::error::JSResult, o_core::error::JSError> {
        let result = Command::new(self.path.clone())
            .arg("-c")
            .arg(filename)
            .arg(code)
            .output()
            .map_err(|source| {
                JSError::internal(format!(
                    "failed to execute toolchain binary `{}`: {source}",
                    self.path
                ))
                .with_filename(filename)
                .with_source(code)
            })?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let message = if stderr.trim().is_empty() {
                format!(
                    "toolchain binary `{}` exited with status {}",
                    self.path, result.status
                )
            } else {
                format!(
                    "toolchain binary `{}` failed: {}",
                    self.path,
                    stderr.trim()
                )
            };

            return Err(JSError::runtime(message)
                .with_filename(filename)
                .with_source(code));
        }

        let output = String::from_utf8(result.stdout).map_err(|source| {
            JSError::internal(format!(
                "toolchain binary `{}` returned invalid UTF-8 output: {source}",
                self.path
            ))
            .with_filename(filename)
            .with_source(code)
        })?;
        Ok(JSResult::String(output))
    }
}
