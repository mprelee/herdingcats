use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Parse,
    Validation,
    Io,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
}

impl Diagnostic {
    pub fn parse(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Parse,
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Validation,
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Io,
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl Error for Diagnostic {}
