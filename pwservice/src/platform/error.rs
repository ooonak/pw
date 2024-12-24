use core::fmt;

/// Module specific custom error, to learn I have not used any macro from one of the error libs.
#[derive(Debug, Clone)]
pub struct MachineError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

/// Must implement fmt.
impl fmt::Display for MachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}:{}", self.message, self.line, self.column)
    }
}

/// Must implement Error, default implementations are fine.
impl std::error::Error for MachineError {}
